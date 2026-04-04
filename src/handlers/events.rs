use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::handlers::{check_user_access, AuthenticatedUser};
use crate::models::{CreateEvent, Event, QueryEvents, UpdateEvent};
use crate::state::AppState;

#[derive(Serialize)]
pub struct EventResponse {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub recurrence_rule: Option<String>,
    pub recurrence_until: Option<String>,
    pub reminder_minutes: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub status: String,
    pub created_at: String,
}

impl TryFrom<Event> for EventResponse {
    type Error = serde_json::Error;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        Ok(Self {
            id: event.id,
            user_id: event.user_id,
            title: event.title,
            description: event.description,
            location: event.location,
            start_time: event.start_time,
            end_time: event.end_time,
            recurrence_rule: event.recurrence_rule,
            recurrence_until: event.recurrence_until,
            reminder_minutes: event.reminder_minutes,
            tags: match &event.tags {
                Some(t) => Some(serde_json::from_str(t)?),
                None => None,
            },
            status: event.status,
            created_at: event.created_at,
        })
    }
}

#[derive(Deserialize)]
pub struct CreateEventRequest {
    pub user_id: String,
    pub event: CreateEvent,
}

#[derive(Deserialize)]
pub struct EventQuery {
    pub user_id: Option<String>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub keyword: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn create_event(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Json(req): Json<CreateEventRequest>,
) -> AppResult<(StatusCode, Json<EventResponse>)> {
    check_user_access(&auth, &req.user_id)?;

    let event = state.event_repo.create(req.user_id.clone(), req.event).await?;
    let response = EventResponse::try_from(event)?;
    let user_id = req.user_id;

    // Fire webhook notification in background
    let webhook_service = state.webhook_service.clone();
    let webhook_payload = serde_json::to_value(&response).unwrap_or_default();
    tokio::spawn(async move {
        if let Err(e) = webhook_service.send_event_webhook(
            &user_id,
            "event.created",
            webhook_payload,
        ).await {
            tracing::warn!(error = %e, "Failed to send event.created webhook");
        }
    });

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_events(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Query(query): Query<EventQuery>,
) -> AppResult<Json<Vec<EventResponse>>> {
    let user_id = query
        .user_id
        .clone()
        .unwrap_or_else(|| auth.user_id.clone());

    check_user_access(&auth, &user_id)?;

    let query = QueryEvents {
        user_id: Some(user_id.clone()),
        status: query.status.or_else(|| Some("active".to_string())),
        from: query.from,
        to: query.to,
        keyword: query.keyword,
    };

    let events = state.event_repo.find_by_user(&user_id, query).await?;
    let response: Vec<EventResponse> = events
        .into_iter()
        .map(EventResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(response))
}

pub async fn get_event(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<Json<EventResponse>> {
    let event = state.event_repo.find_by_id(&id).await?;
    check_user_access(&auth, &event.user_id)?;

    Ok(Json(EventResponse::try_from(event)?))
}

#[tracing::instrument(skip_all)]
pub async fn update_event(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
    Json(input): Json<UpdateEvent>,
) -> AppResult<Json<EventResponse>> {
    let event = state.event_repo.find_by_id(&id).await?;
    check_user_access(&auth, &event.user_id)?;

    let event = state.event_repo.update(&id, input).await?;
    let response = EventResponse::try_from(event)?;
    let user_id = response.user_id.clone();
    let resp_clone = serde_json::to_value(&response).unwrap_or_default();

    // Fire webhook notification in background
    let webhook_service = state.webhook_service.clone();
    tokio::spawn(async move {
        if let Err(e) = webhook_service.send_event_webhook(
            &user_id,
            "event.updated",
            resp_clone,
        ).await {
            tracing::warn!(error = %e, "Failed to send event.updated webhook");
        }
    });

    Ok(Json(response))
}

#[tracing::instrument(skip_all)]
pub async fn delete_event(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let event = state.event_repo.find_by_id(&id).await?;
    check_user_access(&auth, &event.user_id)?;
    let user_id = event.user_id.clone();

    state.event_repo.delete(&id).await?;

    // Fire webhook notification in background
    let webhook_service = state.webhook_service.clone();
    tokio::spawn(async move {
        if let Err(e) = webhook_service.send_event_webhook(
            &user_id,
            "event.deleted",
            serde_json::json!({"id": id}),
        ).await {
            tracing::warn!(error = %e, "Failed to send event.deleted webhook");
        }
    });

    Ok(StatusCode::NO_CONTENT)
}
