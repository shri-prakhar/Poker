use std::error;

use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use log::{error, warn};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Bad Request : {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Not Found;{0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("ValidationError: {0}")]
    ValidationError(String),
    #[error("Service Unavailable")]
    ServiceUnavailable,
    #[error("DataBase Error: ")]
    DataBaseError(String),
    #[error("Internal Server Error")]
    InternalServerError,
    #[error(transparent)]
    ExternalError(#[from] anyhow::Error),
}

#[derive(Debug, Serialize)]

pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

impl ServiceError {
    fn details(&self) -> Option<String> {
        match self {
            ServiceError::BadRequest(msg) => Some(format!("Invalid Input : {}", msg)),
            ServiceError::Unauthorized(msg) => Some(format!("Unauthorized: {}", msg)),
            ServiceError::Forbidden(msg) => Some(format!("Forbidden: {}", msg)),
            ServiceError::NotFound(msg) => Some(format!("NotFound: {}", msg)),
            ServiceError::Conflict(msg) => Some(format!("Conflict: {}", msg)),
            _ => None,
        }
    }
}

impl ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServiceError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ServiceError::Forbidden(_) => StatusCode::FORBIDDEN,
            ServiceError::NotFound(_) => StatusCode::NOT_FOUND,
            ServiceError::Conflict(_) => StatusCode::CONFLICT,
            ServiceError::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ServiceError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ServiceError::DataBaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::ExternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

fn error_response(&self) -> HttpResponse {
    match self {
        ServiceError::InternalServerError => {
            error!("Internal server error occurred:{}", self);
        }
        ServiceError::DataBaseError(_) => {
            error!("DataBase error: {}", self)
        }
        ServiceError::ExternalError(_) => {
            error!("External Dependency Error: {}", self)
        }
        ServiceError::ServiceUnavailable => {
            error!("Service temporarily Error:{}", self)
        }
        _ => {
            log::debug!("Client Error: {}", self)
        }
    }

    let body = ErrorResponse {
        error: self.to_string(),
        details: self.details(),
    };

    HttpResponse::build(self.status_code()).json(body)
}

impl From<anyhow::Error> for ServiceError {
    fn from(error: anyhow::Error) -> Self {
        ServiceError::ExternalError(error)
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(error: sqlx::Error) -> Self {
        ServiceError::DataBaseError(error)
    }
}
