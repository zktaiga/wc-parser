use std::collections::HashMap;
use std::env;
use std::fs;
use wc_parser::parse_string;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let content = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    let messages = parse_string(&content, None).unwrap();

    let mut user_counts = HashMap::new();
    for message in messages {
        if let Some(author) = message.author {
            *user_counts.entry(author).or_insert(0) += 1;
        }
    }

    let mut sorted_users: Vec<_> = user_counts.into_iter().collect();
    sorted_users.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Users by message count:");
    for (user, count) in sorted_users {
        println!("{}: {}", user, count);
    }
}
