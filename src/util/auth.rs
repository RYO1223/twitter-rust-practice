use jwt_simple::prelude::*;
use std::env;

// Authentication utility
pub struct Authentication;

impl Authentication {
    // Function to create a new JWT token
    pub fn create_token(user_id: i32) -> Result<String, String> {
        let key = get_jwt_key();

        let claims = Claims::create(Duration::from_days(7)).with_subject(user_id.to_string());

        key.authenticate(claims)
            .map_err(|e| format!("Error creating token: {}", e))
    }

    // Function to verify and extract claims from token
    pub fn verify_token(token: &str) -> Result<JWTClaims<NoCustomClaims>, String> {
        let key = get_jwt_key();

        key.verify_token::<NoCustomClaims>(token, None)
            .map_err(|e| format!("Error verifying token: {}", e))
    }
}

// Helper function to get JWT key
fn get_jwt_key() -> HS256Key {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");
    HS256Key::from_bytes(secret.as_bytes())
}
