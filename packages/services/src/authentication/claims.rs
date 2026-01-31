use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_serializes_to_json() {
        let claims = Claims {
            iss: "localhost".to_string(),
            sub: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            exp: 1700000000,
            iat: 1699996400,
            jti: "unique-token-id".to_string(),
        };

        let json = serde_json::to_string(&claims).unwrap();

        assert!(json.contains("\"iss\":\"localhost\""));
        assert!(json.contains("\"sub\":\"550e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"exp\":1700000000"));
        assert!(json.contains("\"iat\":1699996400"));
        assert!(json.contains("\"jti\":\"unique-token-id\""));
    }

    #[test]
    fn test_claims_deserializes_from_json() {
        let json = r#"{
            "iss": "localhost",
            "sub": "550e8400-e29b-41d4-a716-446655440000",
            "exp": 1700000000,
            "iat": 1699996400,
            "jti": "unique-token-id"
        }"#;

        let claims: Claims = serde_json::from_str(json).unwrap();

        assert_eq!(claims.iss, "localhost");
        assert_eq!(claims.sub, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(claims.exp, 1700000000);
        assert_eq!(claims.iat, 1699996400);
        assert_eq!(claims.jti, "unique-token-id");
    }
}
