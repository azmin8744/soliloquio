use std::fmt;

/// Various types of password validation errors
#[derive(Debug)]
pub enum PasswordValidationError {
    /// Password is too short
    TooShort,
    /// Password is missing an uppercase letter
    MissingUppercase,
    /// Password is missing a lowercase letter
    MissingLowercase,
    /// Password is missing a digit
    MissingDigit,
    /// Password is missing a special character
    MissingSpecialChar,
    /// Password is a commonly used password
    CommonPassword,
}

impl fmt::Display for PasswordValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PasswordValidationError::TooShort => 
                write!(f, "Password must be at least 12 characters long"),
            PasswordValidationError::MissingUppercase => 
                write!(f, "Password must contain at least one uppercase letter"),
            PasswordValidationError::MissingLowercase => 
                write!(f, "Password must contain at least one lowercase letter"),
            PasswordValidationError::MissingDigit => 
                write!(f, "Password must contain at least one digit"),
            PasswordValidationError::MissingSpecialChar => 
                write!(f, "Password must contain at least one special character"),
            PasswordValidationError::CommonPassword => 
                write!(f, "Password is too common and easily guessable"),
        }
    }
}

/// Validates a password against security requirements
/// 
/// # Arguments
/// * `password` - The password to validate
/// 
/// # Returns
/// * `Ok(())` if the password is valid
/// * `Err(PasswordValidationError)` if the password is invalid
pub fn validate_password(password: &str) -> Result<(), PasswordValidationError> {
    // Check length
    if password.len() < 12 {
        return Err(PasswordValidationError::TooShort);
    }
    
    // Check for uppercase
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(PasswordValidationError::MissingUppercase);
    }
    
    // Check for lowercase
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(PasswordValidationError::MissingLowercase);
    }
    
    // Check for digits
    if !password.chars().any(|c| c.is_numeric()) {
        return Err(PasswordValidationError::MissingDigit);
    }
    
    // Check for special characters
    let has_only_alphanumeric = password.chars().all(|c| c.is_alphanumeric());
    if has_only_alphanumeric {
        return Err(PasswordValidationError::MissingSpecialChar);
    }
    
    // Check against common passwords (could use an actual list in production)
    let common_passwords = ["Password123!", "Qwerty123!", "Admin123!"];
    if common_passwords.contains(&password) {
        return Err(PasswordValidationError::CommonPassword);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_password_length() {
        // Too short
        assert!(matches!(validate_password("Short1!"), Err(PasswordValidationError::TooShort)));
        
        // Just right
        let password = "LongEnough123!";
        assert!(validate_password(password).is_ok());
    }
    
    #[test]
    fn test_password_uppercase() {
        assert!(matches!(validate_password("longpassword123!"), Err(PasswordValidationError::MissingUppercase)));
        assert!(validate_password("LongPassword123!").is_ok());
    }
    
    #[test]
    fn test_password_lowercase() {
        assert!(matches!(validate_password("PASSWORD123!"), Err(PasswordValidationError::MissingLowercase)));
        assert!(validate_password("MyUniqueP@ssw0rd").is_ok());
    }
    
    #[test]
    fn test_password_digits() {
        assert!(matches!(validate_password("PasswordNoDigit!"), Err(PasswordValidationError::MissingDigit)));
        assert!(validate_password("MyUniqueP@ssw0rd").is_ok());
    }
    
    #[test]
    fn test_password_special_chars() {
        // This password is long enough and has uppercase, lowercase, and digits but no special chars
        assert!(matches!(validate_password("PasswordWithoutSpecialChars123456"), Err(PasswordValidationError::MissingSpecialChar)));
        assert!(validate_password("MyUniqueP@ssw0rd").is_ok());
    }
    
    #[test]
    fn test_common_password() {
        assert!(matches!(validate_password("Password123!"), Err(PasswordValidationError::CommonPassword)));
        assert!(validate_password("MyUniqueP@ssw0rd").is_ok());
    }
}
