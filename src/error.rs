use std::{fmt, convert::Infallible};
use derive_more::Display;
use actix_web::http::StatusCode;
use div_com::error::DError;
use div_db::sqlx::error::Error as SqlxError;
use actix_web::{ResponseError, HttpResponse};

pub type AResult<T> = Result<T, ApiError>;

#[derive(Debug, Display)]
pub enum ApiError {
    #[display(fmt = "Request error")]
    RequestError(actix_web::Error),
    #[display(fmt = "Response error")]
    ResponseError(actix_web::Error),
    #[display(fmt = "Path error")]
    PathError(actix_web::error::PathError),
    #[display(fmt = "Db error")]
    DbError(div_com::error::DError),
    #[display(fmt = "Sqlx error")]
    SqlxError(div_db::sqlx::Error),
    #[display(fmt = "AWS error")]
    AwsReqError(div_com::error::DError),
    #[display(fmt = "IO error")]
    IoError(div_com::error::DError),
    UserError(UserError),
    #[display(fmt = "Parsing error")]
    Inf(Infallible),
}


#[derive(Debug, Display)]
pub enum UserError {
    #[display(fmt = "Validation error on field: {}", field)]
    ValidationError { field: String },
    #[display(fmt = "No user with {} = {}", field, val)]
    NoSuchUser { field: String, val: String },
}

impl actix_web::error::ResponseError for UserError {

    fn error_response(&self) -> HttpResponse {
        match self {
            Self::ValidationError { field } => HttpResponse::BadRequest()
                .body(format!("No field {}", field)),
            Self::NoSuchUser { field, val } => HttpResponse::NotFound()
                .body(format!("No user with {} {}", field, val))
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            UserError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            UserError::NoSuchUser { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

impl std::error::Error for ApiError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::RequestError(e) => Some(e),
            Self::ResponseError(e) => Some(e),
            Self::DbError(DError::DbError(e)) => Some(e),
            Self::UserError(e) => None,
            Self::SqlxError(sqlx_error) => match sqlx_error {
                SqlxError::Io(e) => Some(e),
                SqlxError::Database(e) => None,
                SqlxError::Decode(e) => None,
                SqlxError::RowNotFound => None,
                SqlxError::PoolTimedOut => None,
                _ => None,
            }
            _ => None,
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::ResponseError(e) => {
                HttpResponse::NotFound()
                    .body(e.to_string())
            },
            _ => HttpResponse::Forbidden().finish()
        }
    }
}


impl From<actix_web::Error> for ApiError {
    fn from(e: actix_web::Error) -> ApiError {
        Self::RequestError(e)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> ApiError {
        Self::IoError(DError::IoError(e))
    }
}

impl From<actix_web::error::PathError> for ApiError {
    fn from(e: actix_web::error::PathError) -> ApiError {
        Self::PathError(e)
    }
}

impl From<SqlxError> for ApiError {
    fn from(e: SqlxError) -> Self {
        Self::SqlxError(e)
    }
}



