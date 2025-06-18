use super::password::validate_password;
use super::input_validator::{ValidationErrors, ValidationErrorsExt};

pub struct FieldValidator;

impl FieldValidator {
    pub fn validate_email(email: &str, errors: &mut ValidationErrors) {
        if email.trim().is_empty() {
            errors.add_error("email", "Email cannot be empty".to_string());
            return;
        }
        
        // Basic email format validation
        if !email.contains('@') || !email.contains('.') {
            errors.add_error("email", "Email format is invalid".to_string());
        }
        
        // Add more sophisticated email validation if needed
    }
    
    pub fn validate_password_field(password: &str, field_name: &str, errors: &mut ValidationErrors) {
        if password.trim().is_empty() {
            errors.add_error(field_name, format!("{} cannot be empty", field_name));
            return;
        }
        
        // Use existing password validation
        if let Err(password_error) = validate_password(password) {
            errors.add_error(field_name, password_error.to_string());
        }
    }
    
    pub fn validate_required_string(value: &str, field_name: &str, errors: &mut ValidationErrors) {
        if value.trim().is_empty() {
            errors.add_error(field_name, format!("{} cannot be empty", field_name));
        }
    }
    
    pub fn validate_passwords_different(current: &str, new: &str, errors: &mut ValidationErrors) {
        // Simple check for now - in practice you'd compare against the hash
        if current == new {
            errors.add_error("new_password", "New password must be different from current password".to_string());
        }
    }
}
