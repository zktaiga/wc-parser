use chrono::Utc;
use chrono::offset::TimeZone;
use wc_parser::parse_string;

const CHAT_EXAMPLE: &str = r#"06/03/2017, 00:45 - Messages to this group are now secured with end-to-end encryption. Tap for more info.
06/03/2017, 00:45 - You created group "ShortChat"
06/03/2017, 00:45 - Sample User: This is a test message
08/05/2017, 01:48 - TestBot: Hey I'm a test too!
09/04/2017, 01:50 - +410123456789: How are you?
Is everything alright?"#;

#[test]
fn test_parse_string_empty() {
    assert_eq!(parse_string("", None).len(), 0);
}

#[test]
fn test_parse_string_count() {
    let messages = parse_string(CHAT_EXAMPLE, None);
    assert_eq!(messages.len(), 5);
}

#[test]
fn test_parse_string_multiline() {
    let messages = parse_string(CHAT_EXAMPLE, None);
    assert_eq!(messages[4].message, "How are you?\nIs everything alright?");
}

#[test]
fn test_issue_237() {
    let messages = parse_string("30/12/2020 13:00 - a: m\n13/1/2021 13:00 - a: m", None);
    assert_eq!(
        messages[0].date,
        Utc.with_ymd_and_hms(2020, 12, 30, 13, 0, 0).unwrap()
    );
    assert_eq!(
        messages[1].date,
        Utc.with_ymd_and_hms(2021, 1, 13, 13, 0, 0).unwrap()
    );
}
