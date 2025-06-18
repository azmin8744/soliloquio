use models::users;
use sea_orm::ActiveValue;
use crate::validation::active_model_validator::{ActiveModelValidator, ValidationError};

impl ActiveModelValidator for users::ActiveModel {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut error: Option<ValidationError> = None;

        // Validate email format (basic database-level check)
        if let ActiveValue::Set(ref email) = self.email {
            if email.trim().is_empty() {
                let err = ValidationError::new("email", "Email cannot be empty");
                error = Some(match error {
                    Some(e) => e.combine(err),
                    None => err,
                });
            }
            
            // Basic email format validation for database integrity
            if !email.contains('@') {
                let err = ValidationError::new("email", "Email must contain @ symbol");
                error = Some(match error {
                    Some(e) => e.combine(err),
                    None => err,
                });
            }
        }
        
        // Validate that password is properly hashed before database storage
        if let ActiveValue::Set(ref password) = self.password {
            if password.trim().is_empty() {
                let err = ValidationError::new("password", "Password cannot be empty");
                error = Some(match error {
                    Some(e) => e.combine(err),
                    None => err,
                });
            } else {
                // Ensure password is hashed with Argon2 before storing in database
                if !password.starts_with("$argon2") {
                    let err = ValidationError::new(
                        "password", 
                        "Password must be hashed with Argon2 before database storage"
                    );
                    error = Some(match error {
                        Some(e) => e.combine(err),
                        None => err,
                    });
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
    fn test_database_level_validation() {
        // Valid user with hashed password
        let valid_user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set("test@example.com".to_string()),
            password: Set("$argon2id$v=19$m=4096,t=3,p=1$somesalt$hashedpasswordvalue".to_string()),
            created_at: Set(None),
            updated_at: Set(None),
        };
        
        assert!(valid_user.validate().is_ok());
        
        // Invalid: unhashed password should fail database validation
        let unhashed_password = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set("test@example.com".to_string()),
            password: Set("ValidP@ssw0rd123".to_string()), // Raw password, not hashed
            created_at: Set(None),
            updated_at: Set(None),
        };
        
        assert!(unhashed_password.validate().is_err());
        
        // Invalid: empty email
        let invalid_email = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set("".to_string()),
            password: Set("$argon2id$v=19$m=4096,t=3,p=1$somesalt$hashedpasswordvalue".to_string()),
            created_at: Set(None),
            updated_at: Set(None),
        };
        
        assert!(invalid_email.validate().is_err());
        
        // Invalid: email without @ symbol
        let malformed_email = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set("not-an-email".to_string()),
            password: Set("$argon2id$v=19$m=4096,t=3,p=1$somesalt$hashedpasswordvalue".to_string()),
            created_at: Set(None),
            updated_at: Set(None),
        };
        
        assert!(malformed_email.validate().is_err());
    }
    
    #[test]
    fn test_argon2_hash_detection() {
        let test_cases = vec![
            ("$argon2id$v=19$m=4096,t=3,p=1$salt$hash", true),  // Valid Argon2id
            ("$argon2i$v=19$m=4096,t=3,p=1$salt$hash", true),   // Valid Argon2i
            ("$argon2d$v=19$m=4096,t=3,p=1$salt$hash", true),   // Valid Argon2d
            ("$2a$10$salt$hash", false),                         // bcrypt (invalid)
            ("plaintext_password", false),                       // Plain text (invalid)
            ("", false),                                         // Empty (invalid)
        ];
        
        for (password, should_be_valid) in test_cases {
            let user = users::ActiveModel {
                id: Set(Uuid::new_v4()),
                email: Set("test@example.com".to_string()),
                password: Set(password.to_string()),
                created_at: Set(None),
                updated_at: Set(None),
            };
            
            let is_valid = user.validate().is_ok();
            assert_eq!(is_valid, should_be_valid, 
                "Password '{}' validation result should be {}", password, should_be_valid);
        }
    }
}
