use std::any::Any;
use std::sync::Arc;
use axum::{Extension, Json};
use axum::extract::Query;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use chrono::{Duration, Utc};
use diesel::result::DatabaseErrorKind;
use validator::Validate;
use crate::AppState;
use crate::data::dtos::{LoginUserDto, RegisterUserDto, Response, UserLoginResponseDto, VerifyEmailQueryDto};
use crate::data::UserExt;
use crate::errors::{ErrorMessage, HttpError};
use crate::utils::{password, token};
use crate::mail::mails::send_verification_email;


pub async fn register(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<RegisterUserDto>
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let verification_token = uuid::Uuid::new_v4().to_string();
    let expires_at = (Utc::now() + Duration::hours(24)).naive_utc();

    let hash_password = password::hash(&body.password)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let result = app_state.db_client
        .save_user(body.name.clone(), body.email.clone(), hash_password.clone(), verification_token.clone(), expires_at.clone())
        .await;

    match result {
        Ok(_user) => {
            let send_email_result =
                send_verification_email(
                    &body.email,
                    &body.name,
                    &verification_token
                ).await;
            if let Err(e) = send_email_result {
                eprintln!("Error sending verification email: {}", e);
            }

            Ok((StatusCode::CREATED, Json(Response {
                status: "success",
                message: "Registration successfull Please check your emil to verify your account.".to_string()
            })))
        },
        Err(diesel::result::Error::DatabaseError(db_err, ..)) => {
            if db_err.type_id() == DatabaseErrorKind::UniqueViolation.type_id() {
                Err(HttpError::unique_constraint_violation(
                    ErrorMessage::EmailExists.to_string(),
                ))
            } else {
                Err(HttpError::server_error("Database error".to_string()))
            }
        },
        Err(e) => Err(HttpError::server_error(e.to_string()))
    }
}

pub async fn login(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<LoginUserDto>
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let result = app_state.db_client
        .get_user(None, None, Some(body.email.clone()), None)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = result.ok_or(HttpError::bad_request(ErrorMessage::WrongCredentials.to_string()))?;

    let password_matched = password::compare(&body.password, &user.password)
        .map_err(|_| HttpError::bad_request(ErrorMessage::WrongCredentials.to_string()))?;

    if password_matched {
        let token = token::create_token(
            &user.id.to_string(),
            &app_state.env.jwt_secret.as_bytes(),
            app_state.env.jwt_maxage
        ).map_err(|e| HttpError::server_error(e.to_string()))?;
        let cookie_duration = time::Duration::minutes(app_state.env.jwt_maxage * 60);
        let cookie = Cookie::build(("token", token.clone()))
            .path("/")
            .max_age(cookie_duration)
            .http_only(true)
            .build();

        let response = axum::response::Json(UserLoginResponseDto {
            status: "success".to_string(),
            token,
        });

        let mut headers = HeaderMap::new();

        headers.append(
            header::SET_COOKIE,
            cookie.to_string().parse().unwrap(),
        );

        let mut response = response.into_response();
        response.headers_mut().extend(headers);

        Ok(response)
    } else {
        Err(HttpError::bad_request(ErrorMessage::WrongCredentials.to_string()))
    }
}

pub async fn verify_email(
    Query(query_params): Query<VerifyEmailQueryDto>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse, HttpError> {
    
}