use std::collections::HashMap;

pub type ValidationErrors = HashMap<String, Vec<String>>;

pub trait InputValidator {
    fn validate(&self) -> Result<(), ValidationErrors>;

    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

// Helper trait for combining validation errors
pub trait ValidationErrorsExt {
    fn add_error(&mut self, field: &str, message: String);
    fn merge(&mut self, other: ValidationErrors);
}

impl ValidationErrorsExt for ValidationErrors {
    fn add_error(&mut self, field: &str, message: String) {
        self.entry(field.to_string())
            .or_insert_with(Vec::new)
            .push(message);
    }

    fn merge(&mut self, other: ValidationErrors) {
        for (field, mut errors) in other {
            self.entry(field)
                .or_insert_with(Vec::new)
                .append(&mut errors);
        }
    }
}
