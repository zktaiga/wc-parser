
pub mod datetime;
pub mod parser;
pub mod models;

use crate::parser::{parse_messages};
use crate::models::{Message, ParseStringOptions};

use std::fs::File;
use std::io::Result as IoResult;
use std::path::Path;
use memmap2::Mmap;

pub fn parse_string(s: &str, options: Option<ParseStringOptions>) -> Result<Vec<Message>, String> {
    let lines: Vec<&str> = s.split('\n').collect();
    let opts = options.unwrap_or_default();
    let debug = opts.debug;
    
    if debug {
        println!("🔍 DEBUG: parse_string called with {} characters", s.len());
        println!("🔍 DEBUG: Split into {} lines", lines.len());
        println!("🔍 DEBUG: Options: {:?}", opts);
        println!("🔍 DEBUG: =====================================");
    }
    
    Ok(parse_messages(&parser::make_array_of_messages_with_debug(&lines, debug), &opts))
}

/// Convenience helper that memory-maps a chat export file and parses it without
/// copying its contents into an intermediate `String`.
///
/// This keeps peak memory low (the OS brings pages in on demand) and can be
/// noticeably faster on very large exports.
pub fn parse_file<P: AsRef<Path>>(path: P, options: Option<ParseStringOptions>) -> IoResult<Vec<Message>> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let text: &str = std::str::from_utf8(&mmap).expect("Chat file is not valid UTF-8");
    parse_string(text, options).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
