use crate::{
    domain::{Profile, ValidationIssue},
    runtime::ChromiumRuntime,
    store::{ProfileStore, StoreError},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub struct AppState {
    pub store: ProfileStore,
    pub runtime: ChromiumRuntime,
    pub token: String,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/app.js", get(script))
        .route("/style.css", get(style))
        .route("/api/v1/profiles", get(list).post(create))
        .route(
            "/api/v1/profiles/{id}",
            get(get_one).put(update).delete(remove),
        )
        .route("/api/v1/profiles/{id}/start", post(start))
        .route("/api/v1/profiles/{id}/stop", post(stop))
        .with_state(state)
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../ui/index.html"))
}
async fn script() -> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/javascript; charset=utf-8",
        )],
        include_str!("../ui/app.js"),
    )
}
async fn style() -> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
    (
        [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../ui/style.css"),
    )
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    issues: Vec<ValidationIssue>,
}
struct ApiError(StatusCode, ErrorBody);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(self.1)).into_response()
    }
}
fn unauthorized() -> ApiError {
    ApiError(
        StatusCode::UNAUTHORIZED,
        ErrorBody {
            code: "unauthorized",
            message: "valid bearer token required".into(),
            issues: vec![],
        },
    )
}
fn authorize(headers: &HeaderMap, state: &AppState) -> Result<(), ApiError> {
    let expected = format!("Bearer {}", state.token);
    if headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        == Some(expected.as_str())
    {
        Ok(())
    } else {
        Err(unauthorized())
    }
}
fn internal(error: impl std::fmt::Display) -> ApiError {
    ApiError(
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorBody {
            code: "internal",
            message: error.to_string(),
            issues: vec![],
        },
    )
}
fn store_error(error: StoreError) -> ApiError {
    match error {
        StoreError::NotFound(_) => ApiError(
            StatusCode::NOT_FOUND,
            ErrorBody {
                code: "not_found",
                message: error.to_string(),
                issues: vec![],
            },
        ),
        StoreError::Conflict { .. } => ApiError(
            StatusCode::CONFLICT,
            ErrorBody {
                code: "revision_conflict",
                message: error.to_string(),
                issues: vec![],
            },
        ),
        _ => internal(error),
    }
}
fn validate(profile: &Profile) -> Result<(), ApiError> {
    let issues = profile.validate();
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorBody {
                code: "invalid_profile",
                message: "profile validation failed".into(),
                issues,
            },
        ))
    }
}

async fn list(
    State(s): State<Arc<AppState>>,
    h: HeaderMap,
) -> Result<Json<Vec<Profile>>, ApiError> {
    authorize(&h, &s)?;
    Ok(Json(s.store.list().map_err(store_error)?))
}
async fn get_one(
    State(s): State<Arc<AppState>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Profile>, ApiError> {
    authorize(&h, &s)?;
    Ok(Json(s.store.get(id).map_err(store_error)?))
}
async fn create(
    State(s): State<Arc<AppState>>,
    h: HeaderMap,
    Json(p): Json<Profile>,
) -> Result<(StatusCode, Json<Profile>), ApiError> {
    authorize(&h, &s)?;
    validate(&p)?;
    Ok((
        StatusCode::CREATED,
        Json(s.store.create(p).map_err(store_error)?),
    ))
}
async fn update(
    State(s): State<Arc<AppState>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
    Json(mut p): Json<Profile>,
) -> Result<Json<Profile>, ApiError> {
    authorize(&h, &s)?;
    if p.id != id {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            ErrorBody {
                code: "id_mismatch",
                message: "path and profile IDs differ".into(),
                issues: vec![],
            },
        ));
    }
    let expected = p.revision;
    validate(&p)?;
    p.revision = 0;
    Ok(Json(s.store.update(p, expected).map_err(store_error)?))
}
#[derive(Deserialize)]
struct Revision {
    revision: u64,
}
async fn remove(
    State(s): State<Arc<AppState>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
    Json(r): Json<Revision>,
) -> Result<StatusCode, ApiError> {
    authorize(&h, &s)?;
    s.store.delete(id, r.revision).map_err(store_error)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn start(
    State(s): State<Arc<AppState>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    authorize(&h, &s)?;
    let p = s.store.get(id).map_err(store_error)?;
    validate(&p)?;
    s.runtime.start(&p).await.map_err(internal)?;
    Ok(StatusCode::ACCEPTED)
}
async fn stop(
    State(s): State<Arc<AppState>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    authorize(&h, &s)?;
    if s.runtime.stop(id).await.map_err(internal)? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}
