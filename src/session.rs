// Session and related data types for Claude Code session parsing

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// A discovered Claude Code session with metadata.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub project_path: String,
    pub project_name: String,
    pub git_branch: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub first_message: String,
    pub cwd: String,
    pub project_exists: bool,
}

impl Session {
    /// Build the shell command to resume this session.
    pub fn resume_command(&self) -> String {
        format!("cd {} && claude -r {}", self.cwd, self.id)
    }
}

/// A single line from a session JSONL file.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SessionFileEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    #[serde(rename = "gitBranch")]
    pub git_branch: Option<String>,
    pub timestamp: Option<String>,
    pub version: Option<String>,
    pub message: Option<MessageContent>,
    pub uuid: Option<String>,
}

/// The message payload inside a JSONL entry.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MessageContent {
    pub role: String,
    pub content: StringOrArray,
}

/// Content can be a plain string or an array of content blocks.
#[derive(Debug, Clone)]
pub enum StringOrArray {
    Str(String),
    Array(Vec<ContentBlock>),
}

impl StringOrArray {
    /// Extract the text content, concatenating text blocks if needed.
    pub fn text(&self) -> String {
        match self {
            StringOrArray::Str(s) => s.clone(),
            StringOrArray::Array(blocks) => blocks
                .iter()
                .filter(|b| b.block_type == "text")
                .map(|b| b.text.as_str())
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}

/// A single content block within an array-style message.
#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(default)]
    pub text: String,
}

impl<'de> Deserialize<'de> for StringOrArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        struct StringOrArrayVisitor;

        impl<'de> de::Visitor<'de> for StringOrArrayVisitor {
            type Value = StringOrArray;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or an array of content blocks")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StringOrArray::Str(v.to_string()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StringOrArray::Str(v))
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let blocks: Vec<ContentBlock> =
                    Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))?;
                Ok(StringOrArray::Array(blocks))
            }
        }

        deserializer.deserialize_any(StringOrArrayVisitor)
    }
}

/// A single user prompt extracted from a session JSONL file.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserPrompt {
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub uuid: Option<String>,
}

/// An entry for history display.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HistoryEntry {
    pub display: String,
    pub timestamp: i64,
    pub project: String,
}
