use super::{SecurityError, Result};

pub trait Validator<T> {
    fn validate(&self, input: &T) -> Result<()>;
}

pub struct StringValidator {
    max_length: usize,
    allowed_chars: Option<String>,
}

impl StringValidator {
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            allowed_chars: None,
        }
    }

    pub fn with_allowed_chars(mut self, chars: impl Into<String>) -> Self {
        self.allowed_chars = Some(chars.into());
        self
    }
}

impl Validator<String> for StringValidator {
    fn validate(&self, input: &String) -> Result<()> {
        if input.len() > self.max_length {
            return Err(SecurityError::ValidationFailed(format!(
                "Input exceeds maximum length of {}",
                self.max_length
            )));
        }

        if let Some(allowed) = &self.allowed_chars {
            for c in input.chars() {
                if !allowed.contains(c) {
                    return Err(SecurityError::ValidationFailed(format!(
                        "Invalid character '{}' in input",
                        c
                    )));
                }
            }
        }

        Ok(())
    }
}

pub struct PathValidator {
    allow_absolute: bool,
    allow_parent: bool,
}

impl PathValidator {
    pub fn new() -> Self {
        Self {
            allow_absolute: false,
            allow_parent: false,
        }
    }

    pub fn allow_absolute(mut self, allow: bool) -> Self {
        self.allow_absolute = allow;
        self
    }

    pub fn allow_parent(mut self, allow: bool) -> Self {
        self.allow_parent = allow;
        self
    }
}

impl Default for PathValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator<String> for PathValidator {
    fn validate(&self, input: &String) -> Result<()> {
        if !self.allow_absolute && input.starts_with('/') {
            return Err(SecurityError::ValidationFailed(
                "Absolute paths are not allowed".to_string(),
            ));
        }

        if !self.allow_parent && input.contains("..") {
            return Err(SecurityError::ValidationFailed(
                "Parent directory references are not allowed".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_validation() {
        let validator = StringValidator::new(10);
        
        assert!(validator.validate(&"short".to_string()).is_ok());
        assert!(validator.validate(&"this is too long".to_string()).is_err());
    }

    #[test]
    fn test_path_validation() {
        let validator = PathValidator::new();
        
        assert!(validator.validate(&"relative/path".to_string()).is_ok());
        assert!(validator.validate(&"/absolute/path".to_string()).is_err());
        assert!(validator.validate(&"../parent".to_string()).is_err());
    }
}
