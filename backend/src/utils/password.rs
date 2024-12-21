use argon2::{
    password_hash::{
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
    },
    Argon2,
};
use argon2::password_hash::Salt;
use crate::errors::ErrorMessage;

const MAX_PASSWORD_LENGTH: usize = 128;
const SALT_STR: &str = "öasldgjfAFGÄLÖJAdfgadfgasdfö";

pub fn hash(password: impl Into<String>) -> Result<String, ErrorMessage> {
    let pwd = password.into();

    if pwd.is_empty() {
        return Err(ErrorMessage::EmptyPassword)
    }

    if pwd.len() > MAX_PASSWORD_LENGTH {
        return  Err(ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD_LENGTH))
    }

    let salt: Salt = SALT_STR.try_into().unwrap();
    let hashed_pwd = Argon2::default()
        .hash_password(pwd.as_bytes(), salt)
        .map_err(|_| ErrorMessage::HashingError)?
        .to_string();
    Ok(hashed_pwd)
}

pub fn compare(password: &str, hashed_pwd: &str) -> Result<bool, ErrorMessage> {
    if password.is_empty() {
        return Err(ErrorMessage::EmptyPassword)
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return  Err(ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD_LENGTH))
    }

    let parsed_hash = PasswordHash::new(hashed_pwd)
        .map_err(|_| ErrorMessage::InvalidHashFormat)?;

    let password_match = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_or(false, |_| true);

    Ok(password_match)
}