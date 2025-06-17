use std::collections::HashMap;
use std::fmt;

/// A mapping of field names to their validation error messages
pub type ValidationErrors = HashMap<String, Vec<String>>;

/// A validation error that can be converted into a user-friendly message
#[derive(Debug)]
pub struct ValidationError {
    pub errors: ValidationErrors,
}

impl ValidationError {
    /// Create a new validation error with a single field and message
    pub fn new(field: &str, message: &str) -> Self {
        let mut errors = ValidationErrors::new();
        errors.insert(field.to_string(), vec![message.to_string()]);
        Self { errors }
    }

    /// Combine multiple validation errors into one
    pub fn combine(mut self, other: ValidationError) -> Self {
        for (field, messages) in other.errors {
            self.errors.entry(field).or_insert_with(Vec::new).extend(messages);
        }
        self
    }

    /// Get a comma-separated list of all error messages
    pub fn to_string_list(&self) -> String {
        self.errors
            .values()
            .flatten()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ")
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_list())
    }
}

/// Trait for validating ActiveModel instances
pub trait ActiveModelValidator {
    /// Validate the active model and return any validation errors
    fn validate(&self) -> Result<(), ValidationError>;
    
    /// Check if the model is valid
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}
