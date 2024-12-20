use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{ExpressionMethods, Insertable, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::result::Error as DieselError;
use tokio::task;
use uuid::Uuid;
use crate::models::{User, UserRole};
use crate::schema::users::dsl::users;
use crate::schema::users::{id as db_id, name as db_name, email as db_email, password as db_password, role as db_role, verification_token as db_token, token_expires_at as db_token_expires_at, created_at, verified};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Clone, Debug)]
pub struct DBClient {
    pool: PgPool,
}

impl DBClient {
    pub fn new(pool: PgPool) -> Self {
        DBClient { pool }
    }
}

#[async_trait]
pub trait UserExt {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<String>,
        email: Option<String>,
        token: Option<String>,
    ) -> Result<Option<User>, DieselError>;

    async fn get_users(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<User>, DieselError>;

    async fn save_user<T: Into<String> + Send + 'static>(
        &self,
        name: T,
        email: T,
        password: T,
        verification_token: T,
        token_expires_at: NaiveDateTime,
    ) -> Result<User, DieselError>;

    async fn get_user_count(&self) -> Result<i64, DieselError>;

    async fn update_user_name(
        &self,
        user_id: Uuid,
        name: String,
    ) -> Result<User, DieselError>;

    async fn update_user_role(
        &self,
        user_id: Uuid,
        role: UserRole,
    ) -> Result<User, DieselError>;

    async fn update_user_password(
        &self,
        user_id: Uuid,
        password: String,
    ) -> Result<User, DieselError>;

    async fn verified_token(
        &self,
        token: &'static str,
    ) -> Result<(), DieselError>;

    async fn add_verified_token(
        &self,
        user_id: Uuid,
        token: &'static str,
        expires_at: NaiveDateTime,
    ) -> Result<(), DieselError>;
}

#[async_trait]
impl UserExt for DBClient {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<String>,
        email: Option<String>,
        token: Option<String>,
    ) -> Result<Option<User>, DieselError> {
        // Clone Pool for thread safety
        let pool = self.pool.clone();

        // Spawn blocking task for Diesel queries
        let result = task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|_| DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                Box::new("Failed to get DB connection".to_string()),
            ))?;

            let mut query = users.into_boxed();

            // Apply filters
            if let Some(user_id) = user_id {
                query = query.filter(db_id.eq(user_id));
            }
            if let Some(name) = name {
                query = query.filter(db_name.eq(name));
            }
            if let Some(email) = email {
                query = query.filter(db_email.eq(email));
            }
            if let Some(token) = token {
                query = query.filter(db_token.eq(token));
            }

            // Execute query
            query.first::<User>(&mut conn).optional()
        })
            .await;

        // Handle async result
        match result {
            Ok(Ok(user)) => Ok(user),              // Query successful
            Ok(Err(err)) => Err(err),             // Diesel returned an error
            Err(err) => Err(DieselError::QueryBuilderError(Box::new(err))), // Tokio task failed
        }
    }

     async fn get_users(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<User>, DieselError> {
        let offset = (page - 1) * limit as u32;
        let pool = self.pool.clone();

        let result = task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|_| DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                Box::new("Failed to get DB connection".to_string()),
            ))?;

            let query = users.into_boxed();

            let u = QueryDsl::offset(QueryDsl::order(query, created_at.desc()), offset.into())
                .limit(limit as i64);

            u.load::<User>(&mut conn)
        }).await;
         match result {
            Ok(Ok(u)) => Ok(u),            // Query erfolgreich
            Ok(Err(err)) => Err(err),             // Diesel-Fehler
            Err(err) => Err(DieselError::QueryBuilderError(Box::new(err))), // Tokio-Task-Fehler
         }
    }

    async fn save_user<T: Into<String> + Send + 'static>(&self, name: T, email: T, password: T, verification_token: T, token_expires_at: NaiveDateTime) -> Result<User, DieselError> {
        let pool = self.pool.clone();
        let result = task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|_| DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                Box::new("Failed to get DB connection".to_string()),
            ))?;

            diesel::insert_into(users)
                .values((
                    db_name.eq(name.into()),
                    db_email.eq(email.into()),
                    db_password.eq(password.into()),
                    db_token.eq(verification_token.into()),
                    db_token_expires_at.eq(token_expires_at),
                    ))
                .get_result::<User>(&mut conn)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    Box::new(e.to_string())
                ))
        }).await;

        match result {
            Ok(Ok(user)) => Ok(user),
            Ok(Err(err)) => Err(err),
            Err(err) => Err(DieselError::QueryBuilderError(Box::new(err)))
        }
    }

    async fn get_user_count(&self) -> Result<i64, DieselError> {
        let pool = self.pool.clone();
        let result = task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|_| DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                Box::new("Failed to get DB connection".to_string()),
            ))?;

            users.select(diesel::dsl::count_star())
                .first::<i64>(&mut conn)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    Box::new(e.to_string())
                ))
        }).await;

        match result {
            Ok(Ok(c)) => Ok(c),
            Ok(Err(err)) => Err(err),
            Err(err) => Err(DieselError::QueryBuilderError(Box::new(err)))
        }
    }

    async fn update_user_name(&self, user_id: Uuid, name: String) -> Result<User, DieselError> {
        let pool = self.pool.clone();
        let result = task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|_| DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                Box::new("Failed to get DB connection".to_string()),
            ))?;

            diesel::update(users.filter(db_id.eq(user_id)))
                .set(db_name.eq(name))
                .get_result::<User>(&mut conn)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    Box::new(e.to_string())
                ))
        }).await;

        match result {
            Ok(Ok(user)) => Ok(user),
            Ok(Err(err)) => Err(err),
            Err(err) => Err(DieselError::QueryBuilderError(Box::new(err)))
        }
    }

    async fn update_user_role(&self, user_id: Uuid, role: UserRole) -> Result<User, DieselError> {
        let pool = self.pool.clone();
        let result = task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|_| DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                Box::new("Failed to get DB connection".to_string()),
            ))?;

            diesel::update(users.filter(db_id.eq(user_id)))
                .set(db_role.eq(role))
                .get_result::<User>(&mut conn)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    Box::new(e.to_string())
                ))
        }).await;

        match result {
            Ok(Ok(user)) => Ok(user),
            Ok(Err(err)) => Err(err),
            Err(err) => Err(DieselError::QueryBuilderError(Box::new(err)))
        }
    }

    async fn update_user_password(&self, user_id: Uuid, password: String) -> Result<User, DieselError> {
        let pool = self.pool.clone();
        let result = task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|_| DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                Box::new("Failed to get DB connection".to_string()),
            ))?;

            diesel::update(users.filter(db_id.eq(user_id)))
                .set(db_password.eq(password))
                .get_result::<User>(&mut conn)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    Box::new(e.to_string())
                ))
        }).await;

        match result {
            Ok(Ok(user)) => Ok(user),
            Ok(Err(err)) => Err(err),
            Err(err) => Err(DieselError::QueryBuilderError(Box::new(err)))
        }
    }

    async fn verified_token(&self, token: &'static str) -> Result<(), DieselError> {
        let pool = self.pool.clone();
        let result = task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|_| DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                Box::new("Failed to get DB connection".to_string()),
            ))?;

            diesel::update(users.filter(db_token.eq(token)))
                .set(verified.eq(true))
                .execute(&mut conn)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    Box::new(e.to_string())
                ))
        }).await;

        match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(err)) => Err(err),
            Err(err) => Err(DieselError::QueryBuilderError(Box::new(err)))
        }
    }

    async fn add_verified_token(&self, user_id: Uuid, token: &'static str, expires_at: NaiveDateTime) -> Result<(), DieselError> {
        let pool = self.pool.clone();
        let result = task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|_| DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                Box::new("Failed to get DB connection".to_string()),
            ))?;

            diesel::update(users.filter(db_id.eq(user_id)))
                .set((db_token.eq(token), db_token_expires_at.eq(expires_at)))
                .execute(&mut conn)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    Box::new(e.to_string())
                ))
        }).await;

        match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(err)) => Err(err),
            Err(err) => Err(DieselError::QueryBuilderError(Box::new(err)))
        }
    }
}