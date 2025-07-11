use crate::datetime::{days_before_months, normalize_date, order_date_components, convert_time_12_to_24, normalize_ampm, normalize_time};
use crate::models::{Attachment, Message, ParseStringOptions, RawMessage};
use lazy_static::lazy_static;
use regex::Regex;
use rayon::prelude::*;

lazy_static! {
    static ref SHARED_REGEX: Regex = Regex::new(r"^(?:\u{200E}|\u{200F})*\[?(\d{1,4}[-/.]\s?\d{1,4}[-/.]\s?\d{1,4})[,.]?\s\D*?(\d{1,2}[.:]\d{1,2}(?:[.:]\d{1,2})?)(?:(?:\s|\u{202F})([AaPp](?:\.\s?|\s?)[Mm]\.?))?\]?(?:\s-|:)?\s").unwrap();
    static ref AUTHOR_AND_MESSAGE_REGEX: Regex = Regex::new(r"(?s)(.+?):\s(.*)").unwrap();
    static ref MESSAGE_REGEX: Regex = Regex::new(r"(?s)(.*)").unwrap();
    static ref REGEX_ATTACHMENT: Regex = Regex::new(r"^(?:\u{200E}|\u{200F})*(?:<.+:(.+)>|([\w-]+\.\w+)\s[(<].+[)>])").unwrap();
    // Precompiled full regexes to avoid runtime compilation cost on each function call
    static ref REGEX_USER: Regex = Regex::new(&format!("{}{}", SHARED_REGEX.as_str(), AUTHOR_AND_MESSAGE_REGEX.as_str())).unwrap();
    static ref REGEX_SYSTEM: Regex = Regex::new(&format!("{}{}", SHARED_REGEX.as_str(), MESSAGE_REGEX.as_str())).unwrap();
}

#[allow(dead_code)]
fn get_full_regex(is_system: bool) -> Regex {
    let pattern = if is_system {
        format!("{}{}", SHARED_REGEX.as_str(), MESSAGE_REGEX.as_str())
    } else {
        format!(
            "{}{}",
            SHARED_REGEX.as_str(),
            AUTHOR_AND_MESSAGE_REGEX.as_str()
        )
    };
    Regex::new(&pattern).unwrap()
}

/// Takes an array of lines and detects the lines that are part of a previous
/// message (multiline messages) and merges them.
///
/// It also labels messages without an author as system messages.
pub fn make_array_of_messages(lines: &[&str]) -> Vec<RawMessage> {
    make_array_of_messages_with_debug(lines, false)
}

/// Takes an array of lines and detects the lines that are part of a previous
/// message (multiline messages) and merges them with optional debug output.
///
/// It also labels messages without an author as system messages.
pub fn make_array_of_messages_with_debug(lines: &[&str], debug: bool) -> Vec<RawMessage> {
    let mut acc: Vec<RawMessage> = Vec::new();
    let regex_parser = &*REGEX_USER;
    let regex_parser_system = &*REGEX_SYSTEM;

    if debug {
        println!("🔍 DEBUG: Starting message aggregation with {} lines", lines.len());
        println!("🔍 DEBUG: User message regex: {}", regex_parser.as_str());
        println!("🔍 DEBUG: System message regex: {}", regex_parser_system.as_str());
        println!("🔍 DEBUG: =====================================");
    }

    for (line_idx, line) in lines.iter().enumerate() {
        if debug {
            println!("🔍 DEBUG: Processing line {}: '{}'", line_idx + 1, line);
        }
        
        if !regex_parser.is_match(line) {
            if regex_parser_system.is_match(line) {
                if debug {
                    println!("🔍 DEBUG: ✓ Detected system message");
                }
                acc.push(RawMessage {
                    system: true,
                    msg: line.to_string(),
                });
            } else if let Some(prev_message) = acc.last_mut() {
                if debug {
                    println!("🔍 DEBUG: ↪ Appending to previous message (multiline)");
                }
                prev_message.msg.push('\n');
                prev_message.msg.push_str(line);
            } else {
                if debug {
                    println!("🔍 DEBUG: ⚠ Line doesn't match any pattern and no previous message exists");
                }
            }
        } else {
            if debug {
                println!("🔍 DEBUG: ✓ Detected user message");
            }
            acc.push(RawMessage {
                system: false,
                msg: line.to_string(),
            });
        }
    }

    if debug {
        println!("🔍 DEBUG: =====================================");
        println!("🔍 DEBUG: Message aggregation complete!");
        println!("🔍 DEBUG: Total messages found: {}", acc.len());
        let system_count = acc.iter().filter(|m| m.system).count();
        let user_count = acc.len() - system_count;
        println!("🔍 DEBUG: - User messages: {}", user_count);
        println!("🔍 DEBUG: - System messages: {}", system_count);
        println!("🔍 DEBUG: =====================================");
    }

    acc
}

/// Parses a message extracting the attachment if it's present.
fn parse_message_attachment(message: &str) -> Option<Attachment> {
    REGEX_ATTACHMENT.captures(message).map(|caps| Attachment {
        file_name: caps
            .get(1)
            .or_else(|| caps.get(2))
            .map_or(String::new(), |m| m.as_str().trim().to_string()),
    })
}

/// Parses and array of raw messages into an array of structured objects.
pub fn parse_messages(messages: &[RawMessage], options: &ParseStringOptions) -> Vec<Message> {
    let mut days_first = options.days_first;
    let parse_attachments = options.parse_attachments;
    let debug = options.debug;

    if debug {
        println!("🔍 DEBUG: Starting message parsing with {} messages", messages.len());
        println!("🔍 DEBUG: Options - days_first: {:?}, parse_attachments: {}", days_first, parse_attachments);
        println!("🔍 DEBUG: =====================================");
    }

    // Precompiled regexes for user and system messages (static)
    let regex_user = &*REGEX_USER;
    let regex_system = &*REGEX_SYSTEM;

    // Use parallel iterator for faster processing when debug is disabled
    let parsed: Vec<_> = if debug {
        messages
            .iter()
            .enumerate()
            .map(|(msg_idx, obj)| {
                // existing sequential logic
                let (system, msg) = (&obj.system, &obj.msg);
                let regex = if *system { regex_system } else { regex_user };
                if debug {
                    println!("🔍 DEBUG: Processing message {}: {} message", msg_idx + 1, if *system { "system" } else { "user" });
                    println!("🔍 DEBUG: Raw message: '{}'", msg);
                    println!("🔍 DEBUG: Using regex: {}", regex.as_str());
                }
                let caps = regex.captures(msg.as_ref()).unwrap();
                let date = caps.get(1).map_or("", |m| m.as_str()).to_string();
                let time = caps.get(2).map_or("", |m| m.as_str()).to_string();
                let ampm = caps.get(3).map(|m| m.as_str().to_string());
                let (author, message) = if *system {
                    (None, caps.get(4).map_or("", |m| m.as_str()).to_string())
                } else {
                    (
                        caps.get(4).map(|m| m.as_str().to_string()),
                        caps.get(5).map_or("", |m| m.as_str()).to_string(),
                    )
                };
                if debug {
                    println!("🔍 DEBUG: Extracted components:\n - Date: '{}'\n - Time: '{}'\n - AM/PM: '{:?}'\n - Author: '{:?}'\n - Message (before cleanup): '{}'", date, time, ampm, author, message);
                }
                let message = message.replace('\u{200E}', "").replace('\u{200F}', "").trim().to_string();
                (date, time, ampm, author, message)
            })
            .collect()
    } else {
        messages
            .par_iter()
            .map(|obj| {
                let (system, msg) = (&obj.system, &obj.msg);
                let regex = if *system { regex_system } else { regex_user };
                let caps = regex.captures(msg.as_ref()).unwrap();
                let date = caps.get(1).map_or("", |m| m.as_str()).to_string();
                let time = caps.get(2).map_or("", |m| m.as_str()).to_string();
                let ampm = caps.get(3).map(|m| m.as_str().to_string());
                let (author, message) = if *system {
                    (None, caps.get(4).map_or("", |m| m.as_str()).to_string())
                } else {
                    (
                        caps.get(4).map(|m| m.as_str().to_string()),
                        caps.get(5).map_or("", |m| m.as_str()).to_string(),
                    )
                };
                let message = message.replace('\u{200E}', "").replace('\u{200F}', "").trim().to_string();
                (date, time, ampm, author, message)
            })
            .collect()
    };

    if days_first.is_none() {
        if debug {
            println!("🔍 DEBUG: Date format not specified, attempting auto-detection...");
        }
        let numeric_dates: Vec<Vec<i32>> = parsed
            .iter()
            .map(|(date, _, _, _, _)| {
                let (d, m, y) = order_date_components(date);
                vec![d.parse().unwrap(), m.parse().unwrap(), y.parse().unwrap()]
            })
            .collect();
        days_first = days_before_months(&numeric_dates);
        if debug {
            println!("🔍 DEBUG: Date format auto-detection result: days_first = {:?}", days_first);
        }
    }

    let final_messages: Vec<Message> = if debug {
        parsed
            .into_iter()
            .enumerate()
            .map(|(msg_idx, (date, time, ampm, author, message))| {
                if debug {
                    println!("🔍 DEBUG: Creating final message object {}", msg_idx + 1);
                }
                // existing logic here (same as before)
                let (day, month, year) = {
                    let (d, m, y) = order_date_components(&date);
                    if days_first == Some(false) {
                        (m, d, y)
                    } else {
                        (d, m, y)
                    }
                };
                let (year, month, day) = normalize_date(&year, &month, &day);
                let time_normalized = if let Some(ampm_val) = ampm {
                    normalize_time(&convert_time_12_to_24(&time, &normalize_ampm(&ampm_val)))
                } else {
                    normalize_time(&time)
                };
                if debug {
                    println!("🔍 DEBUG: Date components: day={}, month={}, year={}", day, month, year);
                    println!("🔍 DEBUG: Time normalized: {}", time_normalized);
                }
                let final_date = {
                    let day_u: u32 = day.parse().unwrap_or(1);
                    let month_u: u32 = month.parse().unwrap_or(1);
                    let year_i: i32 = year.parse().unwrap_or(1970);
                    let mut time_split = time_normalized.split(':');
                    let hour_u: u32 = time_split.next().unwrap_or("0").parse().unwrap_or(0);
                    let minute_u: u32 = time_split.next().unwrap_or("0").parse().unwrap_or(0);
                    let second_u: u32 = time_split.next().unwrap_or("0").parse().unwrap_or(0);
                    let date = chrono::NaiveDate::from_ymd_opt(year_i, month_u, day_u).unwrap();
                    let time = chrono::NaiveTime::from_hms_opt(hour_u, minute_u, second_u).unwrap();
                    let naive_dt = date.and_time(time);
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive_dt, chrono::Utc)
                };
                let mut final_object = Message {
                    date: final_date,
                    author: author.clone(),
                    message: message.clone(),
                    attachment: None,
                };
                if parse_attachments {
                    final_object.attachment = parse_message_attachment(&message);
                }
                final_object
            })
            .collect()
    } else {
        parsed
            .into_par_iter()
            .map(|(date, time, ampm, author, message)| {
                let (day, month, year) = {
                    let (d, m, y) = order_date_components(&date);
                    if days_first == Some(false) {
                        (m, d, y)
                    } else {
                        (d, m, y)
                    }
                };
                let (year, month, day) = normalize_date(&year, &month, &day);
                let time_normalized = if let Some(ampm_val) = ampm {
                    normalize_time(&convert_time_12_to_24(&time, &normalize_ampm(&ampm_val)))
                } else {
                    normalize_time(&time)
                };
                let day_u: u32 = day.parse().unwrap_or(1);
                let month_u: u32 = month.parse().unwrap_or(1);
                let year_i: i32 = year.parse().unwrap_or(1970);
                let mut time_split = time_normalized.split(':');
                let hour_u: u32 = time_split.next().unwrap_or("0").parse().unwrap_or(0);
                let minute_u: u32 = time_split.next().unwrap_or("0").parse().unwrap_or(0);
                let second_u: u32 = time_split.next().unwrap_or("0").parse().unwrap_or(0);
                let date = chrono::NaiveDate::from_ymd_opt(year_i, month_u, day_u).unwrap();
                let time = chrono::NaiveTime::from_hms_opt(hour_u, minute_u, second_u).unwrap();
                let naive_dt = date.and_time(time);
                let final_date = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive_dt, chrono::Utc);
                let mut final_object = Message {
                    date: final_date,
                    author: author.clone(),
                    message: message.clone(),
                    attachment: None,
                };
                if parse_attachments {
                    final_object.attachment = parse_message_attachment(&message);
                }
                final_object
            })
            .collect()
    };

    if debug {
        println!("🔍 DEBUG: Message parsing complete!");
        println!("🔍 DEBUG: Total messages processed: {}", final_messages.len());
        let authors: std::collections::HashSet<_> = final_messages.iter()
            .filter_map(|m| m.author.as_ref())
            .collect();
        println!("🔍 DEBUG: Unique authors: {}", authors.len());
        let with_attachments = final_messages.iter().filter(|m| m.attachment.is_some()).count();
        println!("🔍 DEBUG: Messages with attachments: {}", with_attachments);
        println!("🔍 DEBUG: =====================================");
    }

    final_messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RawMessage;
    use chrono::{Datelike, TimeZone, Timelike, Utc};

    #[test]
    fn test_make_array_of_messages_multiline() {
        let multiline_message = vec!["23/06/2018, 01:55 p.m. - Loris: one", "two"];
        assert_eq!(
            make_array_of_messages(&multiline_message)[0].msg,
            "23/06/2018, 01:55 p.m. - Loris: one\ntwo"
        );
    }

    #[test]
    fn test_make_array_of_messages_system_flag() {
        let multiline_message = vec!["23/06/2018, 01:55 p.m. - Loris: one", "two"];
        let system_message = vec!["06/03/2017, 00:45 - You created group \"Test\""];
        let empty_message = vec!["03/02/17, 18:42 - Luke: "];
        let multiline_system_message = vec![
            "06/03/2017, 00:45 - You created group \"Test\"",
            "This is another line",
        ];

        assert!(!make_array_of_messages(&multiline_message)[0].system);
        assert!(!make_array_of_messages(&empty_message)[0].system);
        assert!(make_array_of_messages(&multiline_system_message)[0].system);
        assert!(make_array_of_messages(&system_message)[0].system);
    }

    #[test]
    fn test_make_array_of_messages_datetime_in_multiline() {
        let multiline_message = vec![
            "23/06/2018, 01:55 p.m. - Loris: one",
            "two",
            "2016-04-29 10:30:00",
        ];
        assert_eq!(
            make_array_of_messages(&multiline_message)[0].msg,
            "23/06/2018, 01:55 p.m. - Loris: one\ntwo\n2016-04-29 10:30:00"
        );
    }

    #[test]
    fn test_parse_messages_normal() {
        let messages = vec![RawMessage {
            system: false,
            msg: "23/06/2018, 01:55 a.m. - Luke: Hey!".to_string(),
        }];
        let parsed = parse_messages(&messages, &ParseStringOptions::default());

        assert_eq!(parsed[0].date.year(), 2018);
        assert_eq!(parsed[0].date.month(), 6);
        assert_eq!(parsed[0].date.day(), 23);
        assert_eq!(parsed[0].date.hour(), 1);
        assert_eq!(parsed[0].date.minute(), 55);
        assert_eq!(parsed[0].date.second(), 0);
        assert_eq!(parsed[0].author, Some("Luke".to_string()));
        assert_eq!(parsed[0].message, "Hey!".to_string());
    }

    #[test]
    fn test_parse_messages_system() {
        let messages = vec![RawMessage {
            system: true,
            msg: "06/03/2017, 00:45 - You created group \"Test\"".to_string(),
        }];
        let parsed = parse_messages(&messages, &ParseStringOptions::default());

        assert_eq!(parsed[0].date.year(), 2017);
        assert_eq!(parsed[0].date.month(), 3);
        assert_eq!(parsed[0].date.day(), 6);
        assert_eq!(parsed[0].date.hour(), 0);
        assert_eq!(parsed[0].date.minute(), 45);
        assert_eq!(parsed[0].date.second(), 0);
        assert_eq!(parsed[0].author, None);
        assert_eq!(parsed[0].message, "You created group \"Test\"".to_string());
    }

    #[test]
    fn test_parse_messages_formats() {
        let format1 = RawMessage {
            system: false,
            msg: "3/6/18, 1:55 p.m. - a: m".to_string(),
        };
        let format2 = RawMessage {
            system: false,
            msg: "03-06-2018, 01.55 PM - a: m".to_string(),
        };
        let format3 = RawMessage {
            system: false,
            msg: "13.06.18 21.25.15: a: m".to_string(),
        };
        let format4 = RawMessage {
            system: false,
            msg: "[06.13.18 21:25:15] a: m".to_string(),
        };
        let format5 = RawMessage {
            system: false,
            msg: "13.6.2018 klo 21.25.15 - a: m".to_string(),
        };
        let format6 = RawMessage {
            system: false,
            msg: "13. 6. 2018. 21:25:15 a: m".to_string(),
        };
        let format7 = RawMessage {
            system: false,
            msg: "[3/6/18 1:55:00 p. m.] a: m".to_string(),
        };
        let format8 = RawMessage {
            system: false,
            msg: "\u{200E}[3/6/18 1:55:00 p. m.] a: m".to_string(),
        };
        let format9 = RawMessage {
            system: false,
            msg: "[2018/06/13, 21:25:15] a: m".to_string(),
        };
        let format10 = RawMessage {
            system: false,
            msg: "[06/2018/13, 21:25:15] a: m".to_string(),
        };
        let format11 = RawMessage {
            system: false,
            msg: "3/6/2018 1:55 p. m. - a: m".to_string(),
        };
        let format12 = RawMessage {
            system: false,
            msg: "3/6/18, 1:55\u{202F}PM - a: m".to_string(),
        };

        let parsed1 = parse_messages(&vec![format1], &ParseStringOptions::default());
        let parsed2 = parse_messages(&vec![format2], &ParseStringOptions::default());
        let parsed3 = parse_messages(&vec![format3], &ParseStringOptions::default());
        let parsed4 = parse_messages(&vec![format4], &ParseStringOptions::default());
        let parsed5 = parse_messages(&vec![format5], &ParseStringOptions::default());
        let parsed6 = parse_messages(&vec![format6], &ParseStringOptions::default());
        let parsed7 = parse_messages(&vec![format7], &ParseStringOptions::default());
        let parsed8 = parse_messages(&vec![format8], &ParseStringOptions::default());
        let parsed9 = parse_messages(&vec![format9], &ParseStringOptions::default());
        let parsed10 = parse_messages(&vec![format10], &ParseStringOptions::default());
        let parsed11 = parse_messages(&vec![format11], &ParseStringOptions::default());
        let parsed12 = parse_messages(&vec![format12], &ParseStringOptions::default());

        let expected1 = Utc.with_ymd_and_hms(2018, 6, 3, 13, 55, 0).unwrap();
        let expected2 = Utc.with_ymd_and_hms(2018, 6, 13, 21, 25, 15).unwrap();

        assert_eq!(parsed1[0].date, expected1);
        assert_eq!(parsed2[0].date, expected1);
        assert_eq!(parsed3[0].date, expected2);
        assert_eq!(parsed4[0].date, expected2);
        assert_eq!(parsed5[0].date, expected2);
        assert_eq!(parsed6[0].date, expected2);
        assert_eq!(parsed7[0].date, expected1);
        assert_eq!(parsed8[0].date, expected1);
        assert_eq!(parsed9[0].date, expected2);
        assert_eq!(parsed10[0].date, expected2);
        assert_eq!(parsed11[0].date, expected1);
        assert_eq!(parsed12[0].date, expected1);
    }

    #[test]
    fn test_parse_messages_days_first_option() {
        let messages = vec![RawMessage {
            system: false,
            msg: "3/6/18, 1:55 p.m. - a: m".to_string(),
        }];
        let parsed_day_first = parse_messages(
            &messages,
            &ParseStringOptions {
                days_first: Some(true),
                ..Default::default()
            },
        );
        let parsed_month_first = parse_messages(
            &messages,
            &ParseStringOptions {
                days_first: Some(false),
                ..Default::default()
            },
        );

        assert_eq!(parsed_day_first[0].date.day(), 3);
        assert_eq!(parsed_day_first[0].date.month(), 6);
        assert_eq!(parsed_month_first[0].date.day(), 6);
        assert_eq!(parsed_month_first[0].date.month(), 3);
    }

    #[test]
    fn test_parse_messages_attachments() {
        let format1 = "3/6/18, 1:55 p.m. - a: < attached: 00000042-PHOTO-2020-06-07-15-13-20.jpg >";
        let format2 = "3/6/18, 1:55 p.m. - a: IMG-20210428-WA0001.jpg (file attached)";
        let format3 = "3/6/18, 1:55 p.m. - a: 2015-08-04-PHOTO-00004762.jpg <\u{200E}attached>";
        let format4 = "3/6/18, 1:55 p.m. - a: \u{200E}4f2680f1db95a8454775cc2eefc95bfc.jpg (Datei angehängt)\nDir auch frohe Ostern.";
        let messages = vec![
            RawMessage {
                system: false,
                msg: format1.to_string(),
            },
            RawMessage {
                system: false,
                msg: "3/6/18, 1:55 p.m. - a: m".to_string(),
            },
            RawMessage {
                system: false,
                msg: format2.to_string(),
            },
            RawMessage {
                system: false,
                msg: format3.to_string(),
            },
            RawMessage {
                system: false,
                msg: format4.to_string(),
            },
        ];

        let parsed_without_attachments = parse_messages(
            &messages,
            &ParseStringOptions {
                parse_attachments: false,
                ..Default::default()
            },
        );
        let parsed_with_attachments = parse_messages(
            &messages,
            &ParseStringOptions {
                parse_attachments: true,
                ..Default::default()
            },
        );

        assert_eq!(
            parsed_with_attachments[0]
                .attachment
                .as_ref()
                .unwrap()
                .file_name,
            "00000042-PHOTO-2020-06-07-15-13-20.jpg"
        );
        assert!(parsed_without_attachments[0].attachment.is_none());
        assert!(parsed_with_attachments[1].attachment.is_none());
        assert_eq!(
            parsed_with_attachments[2]
                .attachment
                .as_ref()
                .unwrap()
                .file_name,
            "IMG-20210428-WA0001.jpg"
        );
        assert_eq!(
            parsed_with_attachments[3]
                .attachment
                .as_ref()
                .unwrap()
                .file_name,
            "2015-08-04-PHOTO-00004762.jpg"
        );
        assert_eq!(
            parsed_with_attachments[4]
                .attachment
                .as_ref()
                .unwrap()
                .file_name,
            "4f2680f1db95a8454775cc2eefc95bfc.jpg"
        );
    }

    #[test]
    fn test_parse_messages_sticker_with_u200e() {
        // This simulates a sticker message with U+200E both at the beginning and before "sticker omitted"
        let sticker_message = "\u{200E}[23/10/21, 18:44:02] Iago: \u{200E}sticker omitted".to_string();
        let messages = vec![RawMessage {
            system: false,
            msg: sticker_message,
        }];
        let parsed = parse_messages(&messages, &ParseStringOptions::default());

        assert_eq!(parsed[0].date.year(), 2021);
        assert_eq!(parsed[0].date.month(), 10);
        assert_eq!(parsed[0].date.day(), 23);
        assert_eq!(parsed[0].date.hour(), 18);
        assert_eq!(parsed[0].date.minute(), 44);
        assert_eq!(parsed[0].date.second(), 2);
        assert_eq!(parsed[0].author, Some("Iago".to_string()));
        // The message should NOT contain the U+200E character
        assert_eq!(parsed[0].message, "sticker omitted");
    }
}
