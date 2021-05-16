use crate::traits::HookData;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum Hook {
    Update,
    Invalid,
}

impl From<&str> for Hook {
    fn from(path: &str) -> Self {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() == 0 {
            return Self::Invalid;
        }

        match *parts.last().unwrap() {
            "update" => Self::Update,
            _ => Self::Invalid,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    TitleFormat(String),
    TitleMaxLength(u8),
    BodyRequired,
    BodyMaxLineLength(u8),
    EnforceSquashMerge,
    RegexCompilation(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let common_message_part = "Validation failed: ";
        match self {
            Self::TitleFormat(pattern) => write!(
                f,
                "{}The format of the commit title is wrong. Please follow the following regex pattern: {}",
                common_message_part, pattern
            ),
            Self::BodyMaxLineLength(body_max_line_length) => write!(
                f,
                "{}The body line length is larger than the allowed {} characters",
                common_message_part, body_max_line_length
            ),
            Self::TitleMaxLength(title_max_length) => write!(
                f,
                "{}The title length is larger than the allowed {} characters",
                common_message_part, title_max_length
            ),
            Self::EnforceSquashMerge => write!(
                f,
                "{}Make sure to squash before trying to merge.",
                common_message_part
            ),
            Self::BodyRequired => write!(
                f,
                "{}Body missing from commit(s). Ensure that all commits contain a commit body.",
                common_message_part
            ),
            Self::RegexCompilation(regex) => write!(
                f,
                "Regex \"{}\" failed to be created",
                regex
            ),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct UpdateRules {
    pub branches: Option<Vec<String>>,
    pub title_max_length: u8,
    pub title_format: String,
    pub body_required: Option<bool>,
    pub body_max_line_length: Option<u8>,
    pub enforce_squash_merge: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct Rules {
    pub update: UpdateRules,
}

impl Rules {
    pub fn new() -> Self {
        return Self {
            update: UpdateRules {
                branches: None,
                title_max_length: 80,
                title_format: "".to_owned(),
                body_required: None,
                body_max_line_length: None,
                enforce_squash_merge: None,
            },
        };
    }
}

// Structs and implementations related to the Git hooks.
#[derive(Deserialize, Debug)]
pub struct UpdateHookData {
    pub branch: String,
    pub new_commit: String,
    pub old_commit: String,
}

impl HookData for UpdateHookData {
    fn get_data(input: &str) -> Self {
        let parts: Vec<&str> = input.split(',').collect();
        let branch = parts[0].replace("refs/heads/", "");
        let old_commit = parts[1].to_owned();
        let new_commit = parts[2].to_owned();
        return Self {
            branch,
            new_commit,
            old_commit,
        };
    }
}
