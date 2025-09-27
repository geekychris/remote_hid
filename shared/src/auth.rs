use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use bcrypt::{hash, verify, DEFAULT_COST};
use thiserror::Error;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,           // Subject (username)
    pub client_type: String,   // Client type (HidClient or Commander)
    pub client_id: Option<String>, // Optional client identifier
    pub exp: i64,              // Expiration timestamp
    pub iat: i64,              // Issued at timestamp
    pub jti: String,           // JWT ID (unique identifier)
}

/// Authentication errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,
    #[error("JWT encoding error: {0}")]
    JwtEncoding(#[from] jsonwebtoken::errors::Error),
    #[error("Password hashing error: {0}")]
    PasswordHashing(#[from] bcrypt::BcryptError),
}

/// Authentication manager for handling JWT tokens and password verification
pub struct AuthManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    token_expiry_hours: i64,
}

impl AuthManager {
    /// Create a new authentication manager with a secret key
    pub fn new(secret: &str, token_expiry_hours: i64) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        
        Self {
            encoding_key,
            decoding_key,
            validation,
            token_expiry_hours,
        }
    }
    
    /// Generate a JWT token for authenticated user
    pub fn generate_token(
        &self,
        username: &str,
        client_type: &str,
        client_id: Option<String>,
    ) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.token_expiry_hours);
        
        let claims = Claims {
            sub: username.to_string(),
            client_type: client_type.to_string(),
            client_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
        };
        
        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }
    
    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)?;
        
        // Check if token is expired
        let now = Utc::now().timestamp();
        if token_data.claims.exp < now {
            return Err(AuthError::TokenExpired);
        }
        
        Ok(token_data.claims)
    }
    
    /// Hash a password using bcrypt
    pub fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        let hashed = hash(password, DEFAULT_COST)?;
        Ok(hashed)
    }
    
    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        let valid = verify(password, hash)?;
        Ok(valid)
    }
}

/// User information for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub active: bool,
}

impl User {
    /// Create a new user with hashed password
    pub fn new(username: String, password: &str, auth_manager: &AuthManager) -> Result<Self, AuthError> {
        let password_hash = auth_manager.hash_password(password)?;
        
        Ok(Self {
            username,
            password_hash,
            created_at: Utc::now(),
            last_login: None,
            active: true,
        })
    }
    
    /// Verify password for this user
    pub fn verify_password(&self, password: &str, auth_manager: &AuthManager) -> Result<bool, AuthError> {
        if !self.active {
            return Ok(false);
        }
        
        auth_manager.verify_password(password, &self.password_hash)
    }
    
    /// Update last login timestamp
    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
    }
}

/// Simple in-memory user store (for demonstration)
/// In production, this would be backed by a database
#[derive(Debug, Default)]
pub struct UserStore {
    users: std::collections::HashMap<String, User>,
}

impl UserStore {
    /// Create a new empty user store
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a user to the store
    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.username.clone(), user);
    }
    
    /// Get a user by username
    pub fn get_user(&self, username: &str) -> Option<&User> {
        self.users.get(username)
    }
    
    /// Get a mutable reference to a user
    pub fn get_user_mut(&mut self, username: &str) -> Option<&mut User> {
        self.users.get_mut(username)
    }
    
    /// Authenticate a user with username/password
    pub fn authenticate(
        &mut self,
        username: &str,
        password: &str,
        auth_manager: &AuthManager,
    ) -> Result<bool, AuthError> {
        if let Some(user) = self.get_user_mut(username) {
            let valid = user.verify_password(password, auth_manager)?;
            if valid {
                user.update_last_login();
            }
            Ok(valid)
        } else {
            Ok(false)
        }
    }
    
    /// Create default admin user for testing
    pub fn create_default_admin(&mut self, auth_manager: &AuthManager) -> Result<(), AuthError> {
        let admin_user = User::new("admin".to_string(), "admin123", auth_manager)?;
        self.add_user(admin_user);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_auth_manager() {
        let auth_manager = AuthManager::new("test_secret", 24);
        
        // Test token generation and validation
        let token = auth_manager
            .generate_token("testuser", "Commander", Some("client123".to_string()))
            .unwrap();
        
        let claims = auth_manager.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "testuser");
        assert_eq!(claims.client_type, "Commander");
        assert_eq!(claims.client_id, Some("client123".to_string()));
    }
    
    #[test]
    fn test_password_hashing() {
        let auth_manager = AuthManager::new("test_secret", 24);
        let password = "test_password_123";
        
        let hash = auth_manager.hash_password(password).unwrap();
        assert!(auth_manager.verify_password(password, &hash).unwrap());
        assert!(!auth_manager.verify_password("wrong_password", &hash).unwrap());
    }
    
    #[test]
    fn test_user_creation() {
        let auth_manager = AuthManager::new("test_secret", 24);
        let user = User::new("testuser".to_string(), "password123", &auth_manager).unwrap();
        
        assert_eq!(user.username, "testuser");
        assert!(user.verify_password("password123", &auth_manager).unwrap());
        assert!(!user.verify_password("wrong_password", &auth_manager).unwrap());
    }
    
    #[test]
    fn test_user_store() {
        let auth_manager = AuthManager::new("test_secret", 24);
        let mut store = UserStore::new();
        
        let user = User::new("testuser".to_string(), "password123", &auth_manager).unwrap();
        store.add_user(user);
        
        assert!(store.authenticate("testuser", "password123", &auth_manager).unwrap());
        assert!(!store.authenticate("testuser", "wrong_password", &auth_manager).unwrap());
        assert!(!store.authenticate("nonexistent", "password123", &auth_manager).unwrap());
    }
}