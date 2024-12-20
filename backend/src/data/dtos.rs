use core::str;
use chrono::NaiveDateTime;
use serde::{ Serialize, Deserialize };
use validator::Validate;

use crate::models::{User, UserRole};

#[derive(Debug, Deserialize, Serialize, Validate, Clone, Default)]
pub struct RegisterUserDto {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Email address required")
    )]
    pub email: String,
    #[validate(length(min = 8, message = "Password must contain 8 characters"))]
    pub password: String,
    #[validate(
        length(min = 8, message = "Password confirmation must contain 8 characters"),
        must_match(other = "password", message = "Passwords do not match")
    )]
    pub confirm_password: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone, Default)]
pub struct LoginUserDto {
    #[validate(length(min = 1, message = "Email is required"), email(message = "Email is invalid"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must contain 8 characters"))]
    pub password: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct RequestQueryDto {
    #[validate(range(min = 1))]
    pub page: Option<usize>,
    #[validate(range(min = 1, max = 50))]
    pub limit: Option<usize>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FilterUserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub verified: bool,
    #[serde(rename = "createAt")]
    pub created_at: NaiveDateTime,
    #[serde(rename = "updateAt")]
    pub updated_at: NaiveDateTime,
}

impl FilterUserDto {
    pub fn filter_user(user: &User) -> Self {
        FilterUserDto {
            id: user.id.to_string(),
            name: user.name.to_string(),
            email: user.email.to_string(),
            verified: user.verified,
            role: user.role.to_str().to_string(),
            created_at: user.created_at.unwrap(),
            updated_at: user.updated_at.unwrap()
        }
    }

    pub fn filter_users(user: &[User]) -> Vec<Self> {
        user.iter().map(FilterUserDto::filter_user).collect()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserData {
    pub user: FilterUserDto,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct UserResponseDto {
    pub status: String,
    pub data: UserData,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct UserListResponseDto {
    pub status: String,
    pub users: Vec<FilterUserDto>,
    pub results: i64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct UserLoginResponseDto {
    pub status: String,
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct Response {
    pub status: &'static str,
    pub message: String
}

#[derive(Deserialize, Serialize, Validate, Default, Clone, Debug)]
pub struct NameUpdateDto {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
}

#[derive(Deserialize, Serialize, Validate, Clone, Debug)]
pub struct RoleUpdateDto {
    #[validate(custom(function = "validate_user_role"))]
    pub role: UserRole,
}

fn validate_user_role(role: &UserRole) -> Result<(), validator::ValidationError> {
    match role {
        UserRole::Admin | UserRole::User => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_role")),
    }
}

#[derive(Deserialize, Serialize, Validate, Clone, Debug, Default)]
pub struct UserPasswordUpdateDto {
    #[validate(length(min = 8, message = "Password must contain 8 characters"))]
    pub password: String,
    #[validate(length(min = 8, message = "Password confirm must contain 8 characters"))]
    pub confirm_password: String,
    #[validate(length(min = 8, message = "Old password must contain 8 characters"))]
    pub old_password: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct VerifyEmailQueryDto {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
}

#[derive(Deserialize, Serialize, Validate, Clone, Debug)]
pub struct ForgotPasswordRequestDto {
    #[validate(length(min = 1, message = "Email is required"), email(message = "Email is invalid"))]
    pub email: String,
}

#[derive(Deserialize, Serialize, Validate, Clone, Debug)]
pub struct ResetPasswordRequestDto {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
    #[validate(length(min = 8, message = "Password must contain 8 characters"))]
    pub password: String,
    #[validate(length(min = 8, message = "Password confirm must contain 8 characters"))]
    pub confirm_password: String,
}