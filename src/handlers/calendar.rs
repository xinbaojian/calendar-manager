use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
};
use crate::{
    models::QueryEvents,
    state::AppState,
    ical::ICalGenerator,
};

pub async fn subscribe_calendar(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> crate::error::AppResult<Response> {
    let user = state.user_repo.find_by_id(&user_id).await?;

    let query = QueryEvents {
        user_id: Some(user_id.clone()),
        status: Some("active".to_string()),
        from: None,
        to: None,
        keyword: None,
    };

    let events = state.event_repo.find_by_user(&user_id, query).await?;

    let ical_content =
        ICalGenerator::generate(&events, &format!("{}的日程", user.username));

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/calendar; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(ical_content.into())
        .unwrap())
}
