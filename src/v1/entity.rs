use std::fmt::{Display, Formatter, Result as FmtResult};
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub struct Story {
    pub story_id: Uuid,
    pub name: String,
    pub owner: String,
}

#[derive(Debug, PartialEq)]
pub struct Task {
    pub task_id: Uuid,
    pub story_id: Uuid,
    pub name: String,
    pub status: Status,
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Incomplete,
    Complete,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self {
            Self::Incomplete => f.write_str("incomplete"),
            Self::Complete => f.write_str("complete"),
        }
    }
}

impl TryFrom<String> for Status {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim().to_lowercase();
        if value == "incomplete" {
            Ok(Self::Incomplete)
        } else if value == "complete" {
            Ok(Self::Complete)
        } else {
            Err(format!("invalid status string: {}", value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_from_string() {
        let result = Status::try_from("incomplete".to_string()).unwrap();
        assert_eq!(result, Status::Incomplete);
    }

    #[test]
    fn status_from_string_error() {
        let err = Status::try_from("xomplete".to_string()).unwrap_err();
        assert_eq!(err.to_string(), "invalid status string: xomplete");
    }

    #[test]
    fn status_to_string() {
        assert_eq!(Status::Complete.to_string(), "complete");
        assert_eq!(Status::Incomplete.to_string(), "incomplete");
    }
}
