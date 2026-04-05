use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
};
use crate::{
    state::AppState,
    ical::ICalGenerator,
};

pub async fn subscribe_calendar(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> crate::error::AppResult<Response> {
    let user = state.user_repo.find_by_id(&user_id).await?;

    // 查询活跃日程和6个月内过期的日程
    let events = state.event_repo.find_active_and_recent_expired(&user_id, 6).await?;

    let ical_content =
        ICalGenerator::generate(&events, &user.username);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/calendar; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(ical_content.into())
        .map_err(|e| crate::error::AppError::Io(std::io::Error::other(e)))
}
