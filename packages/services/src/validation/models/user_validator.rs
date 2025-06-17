use models::users;
use sea_orm::ActiveValue;
use crate::validation::{
    active_model_validator::{ActiveModelValidator, ValidationError},
    password::validate_password,
};

impl ActiveModelValidator for users::ActiveModel {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut error: Option<ValidationError> = None;

        // Validate email if it's set
        if let ActiveValue::Set(ref email) = self.email {
            if email.trim().is_empty() {
                let err = ValidationError::new("email", "Email cannot be empty");
                error = Some(match error {
                    Some(e) => e.combine(err),
                    None => err,
                });
            }
            
            // Check for valid email format
            if !email.contains('@') || !email.contains('.') {
                let err = ValidationError::new("email", "Email format is invalid");
                error = Some(match error {
                    Some(e) => e.combine(err),
                    None => err,
                });
            }
        }
        
        // Validate password if it's set
        if let ActiveValue::Set(ref password) = self.password {
            if password.trim().is_empty() {
                let err = ValidationError::new("password", "Password cannot be empty");
                error = Some(match error {
                    Some(e) => e.combine(err),
                    None => err,
                });
            } else {
                // Only validate if it's not already hashed (check for common hashing prefixes)
                if !password.starts_with("$argon2") {
                    if let Err(password_err) = validate_password(password) {
                        let err = ValidationError::new("password", &password_err.to_string());
                        error = Some(match error {
                            Some(e) => e.combine(err),
                            None => err,
                        });
                    }
                }
            }
        }
        
        match error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Set;
    use uuid::Uuid;

    #[test]
    fn test_user_validation() {
        // Valid user
        let valid_user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set("test@example.com".to_string()),
            password: Set("ValidP@ssw0rd123".to_string()),
            created_at: Set(None),
            updated_at: Set(None),
        };
        
        assert!(valid_user.validate().is_ok());
        
        // Invalid email
        let invalid_email = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set("not-an-email".to_string()),
            password: Set("ValidP@ssw0rd123".to_string()),
            created_at: Set(None),
            updated_at: Set(None),
        };
        
        assert!(invalid_email.validate().is_err());
        
        // Invalid password
        let invalid_password = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set("test@example.com".to_string()),
            password: Set("short".to_string()),
            created_at: Set(None),
            updated_at: Set(None),
        };
        
        assert!(invalid_password.validate().is_err());
        
        // Already hashed password should pass validation
        let hashed_password = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set("test@example.com".to_string()),
            password: Set("$argon2id$v=19$m=4096,t=3,p=1$somesalt$hashedpasswordvalue".to_string()),
            created_at: Set(None),
            updated_at: Set(None),
        };
        
        assert!(hashed_password.validate().is_ok());
    }
}
