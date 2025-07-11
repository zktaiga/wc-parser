use regex::Regex;
use std::collections::HashMap;

/// Takes an array of numeric dates and tries to understand if the days come
/// before the month or the other way around by checking if numbers go above
/// `12`.
///
/// Output is `true` if days are first, `false` if they are second, or `None` if
/// it failed to understand the order.
pub fn check_above_12(numeric_dates: &[Vec<i32>]) -> Option<bool> {
    if numeric_dates.iter().any(|d| index_above_value(0, 12)(d)) {
        return Some(true);
    }
    if numeric_dates.iter().any(|d| index_above_value(1, 12)(d)) {
        return Some(false);
    }
    None
}

/// Takes an array of numeric dates and tries to understand if the days come
/// before the month or the other way around by checking if a set of numbers
/// during the same year decrease at some point.
///
/// If it does it's probably the days since months can only increase in a given
/// year.
///
/// Output is `true` if days are first, `false` if they are second, or `None` if
/// it failed to understand the order.
pub fn check_decreasing(numeric_dates: &[Vec<i32>]) -> Option<bool> {
    let dates_by_year = group_array_by_value_at_index(numeric_dates, 2);
    let results: Vec<Option<bool>> = dates_by_year
        .iter()
        .map(|dates| {
            let days_first = dates.windows(2).any(|w| is_negative(w[1][0] - w[0][0]));
            if days_first {
                return Some(true);
            }

            let days_second = dates.windows(2).any(|w| is_negative(w[1][1] - w[0][1]));
            if days_second {
                return Some(false);
            }

            None
        })
        .collect();

    if results.iter().any(|&r| r == Some(true)) {
        return Some(true);
    }
    if results.iter().any(|&r| r == Some(false)) {
        return Some(false);
    }

    None
}

/// Takes an array of numeric dates and tries to understand if the days come
/// before the month or the other way around by looking at which number changes
/// more frequently.
///
/// Output is `true` if days are first, `false` if they are second, or `None` if
/// it failed to understand the order.
pub fn change_frequency_analysis(numeric_dates: &[Vec<i32>]) -> Option<bool> {
    let diffs: Vec<Vec<i32>> = numeric_dates
        .windows(2)
        .map(|w| {
            w[1].iter()
                .zip(w[0].iter())
                .map(|(a, b)| (a - b).abs())
                .collect()
        })
        .collect();

    let (first, second) = diffs.iter().fold((0, 0), |(mut acc_f, mut acc_s), diff| {
        acc_f += diff[0];
        acc_s += diff[1];
        (acc_f, acc_s)
    });

    if first > second {
        return Some(true);
    }
    if first < second {
        return Some(false);
    }

    None
}

/// Takes an array of numeric dates and tries to understand if the days come
/// before the month or the other way around by running the dates through various
/// checks.
///
/// Output is `true` if days are first, `false` if they are second, or `None` if
/// it failed to understand the order.
pub fn days_before_months(numeric_dates: &[Vec<i32>]) -> Option<bool> {
    check_above_12(numeric_dates)
        .or_else(|| check_decreasing(numeric_dates))
        .or_else(|| change_frequency_analysis(numeric_dates))
}

/// Takes `year`, `month` and `day` as strings and pads them to `4`, `2`, `2`
/// digits respectively.
pub fn normalize_date(year: &str, month: &str, day: &str) -> (String, String, String) {
    // 2 digit years are assumed to be in the 2000-2099 range
    let normalized_year = if year.len() <= 2 {
        format!("20{:0>2}", year)
    } else {
        year.to_string()
    };

    (
        normalized_year,
        format!("{:0>2}", month),
        format!("{:0>2}", day),
    )
}

/// Pushes the longest number in a date to the end, if there is one. Necessary to
/// ensure the year is the last number.
pub fn order_date_components(date: &str) -> (String, String, String) {
    let parts: Vec<&str> = date
        .split(|c| c == '-' || c == '/' || c == '.')
        .map(|s| s.trim())
        .collect();
    let a = parts[0];
    let b = parts[1];
    let c = parts[2];

    let max_len = a.len().max(b.len()).max(c.len());

    if c.len() == max_len {
        (a.to_string(), b.to_string(), c.to_string())
    } else if b.len() == max_len {
        (a.to_string(), c.to_string(), b.to_string())
    } else {
        (b.to_string(), c.to_string(), a.to_string())
    }
}

/// Converts time from 12 hour format to 24 hour format.
pub fn convert_time_12_to_24(time: &str, ampm: &str) -> String {
    let re = Regex::new(r"[:.]").unwrap();
    let parts: Vec<&str> = re.split(time).collect();

    let mut hours = parts[0].parse::<i32>().unwrap();
    let minutes = parts[1];
    let seconds = if parts.len() > 2 {
        Some(parts[2])
    } else {
        None
    };

    if hours == 12 {
        hours = 0;
    }

    if ampm == "PM" {
        hours += 12;
    }

    if let Some(seconds) = seconds {
        format!("{:02}:{}:{}", hours, minutes, seconds)
    } else {
        format!("{:02}:{}", hours, minutes)
    }
}

/// Normalizes a time string to have the following format: `hh:mm:ss`.
pub fn normalize_time(time: &str) -> String {
    let re = Regex::new(r"[:.]").unwrap();
    let parts: Vec<&str> = re.split(time).collect();

    let hours = parts[0];
    let minutes = parts[1];
    let seconds = if parts.len() > 2 { parts[2] } else { "00" };

    format!("{:0>2}:{}:{}", hours, minutes, seconds)
}

/// Normalizes `am` / `a.m.` / etc. to `AM` (uppercase, no other characters).
pub fn normalize_ampm(ampm: &str) -> String {
    ampm.replace(|c: char| !c.is_alphabetic(), "")
        .to_uppercase()
}

/// Checks that the number at a certain index of an array is greater than a
/// certain value.
pub fn index_above_value(index: usize, value: i32) -> impl Fn(&[i32]) -> bool {
    move |array: &[i32]| array[index] > value
}

/// Returns `true` for a negative number, `false` otherwise.
///
/// `0` and `-0` are considered positive.
pub fn is_negative(number: i32) -> bool {
    number < 0
}

/// Takes an array of arrays and an index and groups the inner arrays by the
/// value at the index provided.
pub fn group_array_by_value_at_index<T: Clone>(array: &[Vec<T>], index: usize) -> Vec<Vec<Vec<T>>>
where
    T: std::cmp::Eq + std::hash::Hash + ToString,
{
    let mut map: HashMap<String, Vec<Vec<T>>> = HashMap::new();

    for item in array {
        let key = format!("_{}", item[index].to_string());
        map.entry(key).or_default().push(item.clone());
    }

    map.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_above_12() {
        let days_first = vec![vec![3, 6, 2017], vec![13, 11, 2017], vec![26, 12, 2017]];
        let months_first = vec![vec![4, 2, 2017], vec![6, 11, 2017], vec![12, 13, 2017]];
        let undetectable = vec![vec![4, 6, 2017], vec![11, 10, 2017], vec![12, 12, 2017]];

        assert_eq!(check_above_12(&days_first), Some(true));
        assert_eq!(check_above_12(&months_first), Some(false));
        assert_eq!(check_above_12(&undetectable), None);
    }

    #[test]
    fn test_check_decreasing() {
        let days_first = vec![vec![8, 3, 2017], vec![10, 5, 2017], vec![6, 9, 2017]];
        let months_first = vec![vec![6, 3, 2017], vec![8, 5, 2017], vec![9, 4, 2017]];
        let undetectable = vec![vec![1, 1, 2017], vec![3, 3, 2017], vec![6, 6, 2017]];
        let different_years1 = vec![vec![8, 3, 2017], vec![7, 5, 2017], vec![6, 9, 2018]];
        let different_years2 = vec![vec![8, 3, 2017], vec![10, 2, 2017], vec![6, 9, 2018]];
        let different_years3 = vec![vec![8, 3, 2017], vec![10, 5, 2017], vec![6, 9, 2018]];

        assert_eq!(check_decreasing(&days_first), Some(true));
        assert_eq!(check_decreasing(&months_first), Some(false));
        assert_eq!(check_decreasing(&undetectable), None);
        assert_eq!(check_decreasing(&different_years1), Some(true));
        assert_eq!(check_decreasing(&different_years2), Some(false));
        assert_eq!(check_decreasing(&different_years3), None);
    }

    #[test]
    fn test_change_frequency_analysis() {
        let days_first = vec![vec![3, 4, 2017], vec![7, 5, 2017], vec![11, 6, 2017]];
        let months_first = vec![vec![1, 1, 2017], vec![1, 3, 2017], vec![2, 7, 2017]];
        let undetectable = vec![vec![6, 3, 2017], vec![8, 5, 2017], vec![9, 4, 2017]];

        assert_eq!(change_frequency_analysis(&days_first), Some(true));
        assert_eq!(change_frequency_analysis(&months_first), Some(false));
        assert_eq!(change_frequency_analysis(&undetectable), None);
    }

    #[test]
    fn test_normalize_date() {
        let expected = ("2011".to_string(), "03".to_string(), "04".to_string());

        assert_eq!(normalize_date("11", "3", "4"), expected);
        assert_eq!(normalize_date("2011", "03", "04"), expected);
    }

    #[test]
    fn test_convert_time_12_to_24() {
        assert_eq!(convert_time_12_to_24("12:00", "PM"), "12:00");
        assert_eq!(convert_time_12_to_24("12:00", "AM"), "00:00");
        assert_eq!(convert_time_12_to_24("05:06", "PM"), "17:06");
        assert_eq!(convert_time_12_to_24("07:19", "AM"), "07:19");
        assert_eq!(convert_time_12_to_24("01:02:34", "PM"), "13:02:34");
        assert_eq!(convert_time_12_to_24("02:04:54", "AM"), "02:04:54");
    }

    #[test]
    fn test_normalize_ampm() {
        assert_eq!(normalize_ampm("am"), "AM");
        assert_eq!(normalize_ampm("pm"), "PM");
        assert_eq!(normalize_ampm("AM"), "AM");
        assert_eq!(normalize_ampm("PM"), "PM");
        assert_eq!(normalize_ampm("a.m."), "AM");
        assert_eq!(normalize_ampm("p.m."), "PM");
        assert_eq!(normalize_ampm("A.M."), "AM");
        assert_eq!(normalize_ampm("P.M."), "PM");
    }

    #[test]
    fn test_normalize_time() {
        assert_eq!(normalize_time("12:34"), "12:34:00");
        assert_eq!(normalize_time("1:23:45"), "01:23:45");
        assert_eq!(normalize_time("12:34:56"), "12:34:56");
    }

    #[test]
    fn test_index_above_value() {
        let array = vec![34, 16];

        assert!(index_above_value(0, 33)(&array));
        assert!(!index_above_value(0, 34)(&array));
        assert!(index_above_value(1, 15)(&array));
        assert!(!index_above_value(1, 16)(&array));
        assert!(!index_above_value(1, 17)(&array));
    }

    #[test]
    fn test_is_negative() {
        assert!(is_negative(-1));
        assert!(is_negative(-15));
        assert!(is_negative(std::i32::MIN));

        assert!(!is_negative(0));
        assert!(!is_negative(1));
        assert!(!is_negative(15));
        assert!(!is_negative(std::i32::MAX));
    }

    #[test]
    fn test_group_array_by_value_at_index() {
        let array = vec![
            vec!["8".to_string(), "30".to_string(), "sample".to_string()],
            vec!["9".to_string(), "50".to_string(), "sample".to_string()],
            vec!["6".to_string(), "30".to_string(), "sample".to_string()],
        ];

        let grouped_by_0 = group_array_by_value_at_index(&array, 0);
        assert_eq!(grouped_by_0.len(), 3);

        let grouped_by_1 = group_array_by_value_at_index(&array, 1);
        assert_eq!(grouped_by_1.len(), 2);

        let grouped_by_2 = group_array_by_value_at_index(&array, 2);
        assert_eq!(grouped_by_2.len(), 1);
    }
}
