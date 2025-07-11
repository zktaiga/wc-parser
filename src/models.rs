use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq)]
pub struct RawMessage {
    pub system: bool,
    pub msg: String,
}

#[derive(Debug, PartialEq)]
pub struct Attachment {
    /// The filename of the attachment, including the extension.
    pub file_name: String,
}

#[derive(Debug, PartialEq)]
pub struct Message {
    /// The date of the message.
    pub date: DateTime<Utc>,
    /// The author of the message. Will be None for messages without an author (system messages).
    pub author: Option<String>,
    /// The message itself.
    pub message: String,
    /// Available for messages containing attachments when setting the option
    /// `parse_attachments` to `true`.
    pub attachment: Option<Attachment>,
}

#[derive(Debug, Default)]
pub struct ParseStringOptions {
    /// Specify if the dates in your log file start with a day (`true`) or a month
    /// (`false`).
    ///
    /// Manually specifying this may improve performance.
    pub days_first: Option<bool>,
    /// Specify if attachments should be parsed.
    ///
    /// If set to `true`, messages containing attachments will include an
    /// `attachment` property.
    pub parse_attachments: bool,
    /// Enable debug output during parsing.
    ///
    /// If set to `true`, detailed information about the parsing process will be
    /// printed to stdout, including regex matches, message processing steps, and
    /// statistics.
    pub debug: bool,
}
