use async_graphql::InputObject;
use services::validation::input_validator::{InputValidator, ValidationErrors, ValidationErrorsExt};
use services::validation::field_validators::FieldValidator;

#[derive(InputObject)]
pub struct SignUpInput {
    pub email: String,
    pub password: String,
}

impl InputValidator for SignUpInput {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        
        FieldValidator::validate_email(&self.email, &mut errors);
        FieldValidator::validate_password_field(&self.password, "password", &mut errors);
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(InputObject)]
pub struct SignInInput {
    pub email: String,
    pub password: String,
}

impl InputValidator for SignInInput {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        
        FieldValidator::validate_required_string(&self.email, "email", &mut errors);
        FieldValidator::validate_required_string(&self.password, "password", &mut errors);
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(InputObject)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
}

impl InputValidator for ChangePasswordInput {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        
        FieldValidator::validate_required_string(&self.current_password, "current_password", &mut errors);
        FieldValidator::validate_password_field(&self.new_password, "new_password", &mut errors);
        FieldValidator::validate_passwords_different(&self.current_password, &self.new_password, &mut errors);
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signup_input_validation() {
        // Valid input
        let valid_input = SignUpInput {
            email: "test@example.com".to_string(),
            password: "SecureP@ssw0rd123!".to_string(),
        };
        assert!(valid_input.validate().is_ok());

        // Invalid email
        let invalid_email = SignUpInput {
            email: "not-an-email".to_string(),
            password: "SecureP@ssw0rd123!".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        // Invalid password
        let invalid_password = SignUpInput {
            email: "test@example.com".to_string(),
            password: "short".to_string(),
        };
        assert!(invalid_password.validate().is_err());
    }

    #[test]
    fn test_signin_input_validation() {
        // Valid input
        let valid_input = SignInInput {
            email: "test@example.com".to_string(),
            password: "anypassword".to_string(),
        };
        assert!(valid_input.validate().is_ok());

        // Empty email
        let empty_email = SignInInput {
            email: "".to_string(),
            password: "anypassword".to_string(),
        };
        assert!(empty_email.validate().is_err());

        // Empty password
        let empty_password = SignInInput {
            email: "test@example.com".to_string(),
            password: "".to_string(),
        };
        assert!(empty_password.validate().is_err());
    }

    #[test]
    fn test_change_password_input_validation() {
        // Valid input
        let valid_input = ChangePasswordInput {
            current_password: "OldP@ssw0rd123!".to_string(),
            new_password: "NewSecureP@ssw0rd456!".to_string(),
        };
        assert!(valid_input.validate().is_ok());

        // Same passwords
        let same_passwords = ChangePasswordInput {
            current_password: "SameP@ssw0rd123!".to_string(),
            new_password: "SameP@ssw0rd123!".to_string(),
        };
        assert!(same_passwords.validate().is_err());

        // Weak new password
        let weak_new_password = ChangePasswordInput {
            current_password: "OldP@ssw0rd123!".to_string(),
            new_password: "weak".to_string(),
        };
        assert!(weak_new_password.validate().is_err());
    }
}
