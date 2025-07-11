# wc-parser
[![Rust build & test](https://github.com/zktaiga/wc-parser/actions/workflows/rust.yml/badge.svg)](https://github.com/zktaiga/wc-parser/actions/workflows/rust.yml)

A decently fast Rust library for parsing WhatsApp chat exports.

## Features

- Parse WhatsApp chat exports into structured data
- Support for multiple date and time formats
- Automatic detection of date format (day/month vs month/day)
- Optional attachment parsing
- System message detection
- Multiline message support

## Performance & Optimisations

wc-parser is designed to be fast **and** memory-efficient. Key optimisations include:

- **Memory-mapped I/O** — `parse_file` uses `memmap2` so chat exports are read straight from the operating-system page-cache without first copying them into a `String`, keeping peak RSS low even for multi-gigabyte logs.
- **Zero-copy parsing** — When parsing from a `&str`, we split the original slice into `&str` line slices instead of allocating new strings, only allocating when constructing the final `Message` structs.
- **Pre-compiled regular expressions** — All regex patterns are built once at start-up via `lazy_static!`, removing the compile cost from the hot parsing path.
- **Data-parallel message processing** — Heavy-weight work (regex capture extraction, date/time normalisation, etc.) runs in parallel across CPU cores with `rayon` when debug output is disabled.
- **Selective attachment parsing** — Attachment extraction is completely skipped unless `parse_attachments = true`, saving an extra regex run per message in the common case.
- **Configurable debug logging** — Expensive debug printing is off by default. When enabled it switches to single-threaded execution to keep log output ordered.
- **Small-footprint date handling** — Simple heuristics determine whether the log is day-first or month-first in a single pass, avoiding per-message branching once parsing begins.


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
wc-parser = "0.1.1"
```

### Basic Usage

```rust
use wc_parser::parse_string;

fn main() {
    let chat_content = r#"
06/03/2017, 00:45 - Sample User: This is a test message
08/05/2017, 01:48 - TestBot: Hey I'm a test too!
09/04/2017, 01:50 - +410123456789: How are you?
Is everything alright?
"#;

    let messages = parse_string(chat_content, None).unwrap();
    
    for message in messages {
        println!("Date: {}", message.date);
        if let Some(author) = message.author {
            println!("Author: {}", author);
        } else {
            println!("System message");
        }
        println!("Message: {}", message.message);
        println!("---");
    }
}
```

### Advanced Usage with Options

```rust
use wc_parser::{parse_string, models::ParseStringOptions};

let options = ParseStringOptions {
    days_first: Some(true), // Specify date format
    parse_attachments: true, // Parse attachment information
};

let messages = parse_string(chat_content, Some(options)).unwrap();
```

## Message Structure

Each parsed message contains:

```rust
// Located in `src/models.rs`
pub struct Message {
    // Located in `src/models.rs`
    // Located in `src/models.rs`

    pub date: DateTime<Utc>,           // Date and time of the message
    pub author: Option<String>,        // Author name (None for system messages)
    pub message: String,               // Message content
    pub attachment: Option<Attachment>, // Attachment info (if parse_attachments is enabled)
}
```

## Supported Formats

This library supports various WhatsApp chat export formats including:

- Different date formats (DD/MM/YYYY, MM/DD/YYYY, YYYY/MM/DD, etc.)
- 12-hour and 24-hour time formats
- Various separators and punctuation
- Unicode characters and directional marks
- System messages and notifications
