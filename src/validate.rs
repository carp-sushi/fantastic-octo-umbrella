use crate::{Error, Result};
use uuid::Uuid;

pub struct Validate {}

impl Validate {
    /// Ensure that a given string is non-empty.
    pub fn non_empty(value: &str, param: &str) -> Result<String> {
        let value = value.trim().to_string();
        if value.is_empty() {
            return Err(Error::InvalidArgument {
                message: format!("empty string: {}", param),
            });
        }
        Ok(value)
    }

    /// Ensure a uuid value can be created from a string
    pub fn validate_uuid(value: &str) -> Result<Uuid> {
        let value = value.trim().to_lowercase();
        let uuid = Uuid::parse_str(&value).map_err(|err| Error::InvalidArgument {
            message: err.to_string(),
        })?;
        Ok(uuid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_success() {
        let result = Validate::non_empty(" test ", "").unwrap();
        assert_eq!(result, "test");
    }

    #[test]
    fn non_empty_fail() {
        let error = Validate::non_empty("  ", "2spaces").unwrap_err();
        assert_eq!(error.to_string(), "invalid argument: empty string: 2spaces");
    }

    #[test]
    fn validate_uuid_success() {
        let input = "  4ac0160a-b132-440e-9cdf-135d7a91d6dc  ";
        let result = Validate::validate_uuid(input).unwrap();
        assert_eq!(result.to_string(), input.trim());
    }

    #[test]
    fn validate_uuid_fail() {
        let error = Validate::validate_uuid("4ac0160a").unwrap_err();
        assert!(error.to_string().starts_with("invalid argument"));
    }
}
