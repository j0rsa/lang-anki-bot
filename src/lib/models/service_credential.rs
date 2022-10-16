use crate::lib::curr_millis;

#[derive(Debug, Clone)]
pub struct ServiceCredential {
    pub username: String,
    pub password: String,
    pub token: Option<Token>,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub value: String,
    pub expires_ms: u128,
}

impl AsRef<ServiceCredential> for ServiceCredential {
    fn as_ref(&self) -> &ServiceCredential {
        self
    }
}

impl ServiceCredential {
    pub fn no_token_new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
            token: None,
        }
    }
    pub fn new(username: String, password: String, token: String, expires_ms: u128) -> Self {
        Self {
            username,
            password,
            token: Some(Token {
                value: token,
                expires_ms,
            }),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new_token(self, token: String, expires_at_ms: u128) -> Self {
        Self {
            username: self.username.clone(),
            password: self.password.clone(),
            token: Some(Token {
                value: token,
                expires_ms: expires_at_ms,
            }),
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.token {
            Some(ref token) => curr_millis() > token.expires_ms,
            None => true,
        }
    }
}

trait Expirable {
    fn is_expired(&self) -> bool;
}

impl Expirable for Option<ServiceCredential> {
    fn is_expired(&self) -> bool {
        match self {
            Some(token) => token.is_expired(),
            None => true,
        }
    }
}
