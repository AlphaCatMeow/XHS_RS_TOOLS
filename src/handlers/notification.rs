//! Notification HTTP Handlers
//! 
//! Handles: mentions, connections, likes

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::api;
use crate::server::AppState;

// ============================================================================
// Handlers
// ============================================================================

/// 通知页-评论和@
/// 
/// 获取评论和@通知列表，支持分页
#[utoipa::path(
    get,
    path = "/api/notification/mentions",
    tag = "xhs",
    summary = "通知页-评论和@",
    params(
        ("num" = Option<i32>, Query, description = "每页数量，固定为 20", example = 20),
        ("cursor" = Option<String>, Query, description = "分页游标，首次请求为空，后续使用响应中的 cursor/strCursor 值", example = "")
    ),
    responses(
        (status = 200, description = "评论和@通知列表", body = api::notification::mentions::MentionsResponse)
    )
)]
pub async fn mentions_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<api::notification::mentions::MentionsParams>,
) -> impl IntoResponse {
    match api::notification::mentions::get_mentions_with_params(&state.api, params).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

/// 通知页-新增关注
/// 
/// 获取新增关注通知列表，支持分页
#[utoipa::path(
    get,
    path = "/api/notification/connections",
    tag = "xhs",
    summary = "通知页-新增关注",
    params(
        ("num" = Option<i32>, Query, description = "每页数量，固定为 20", example = 20),
        ("cursor" = Option<String>, Query, description = "分页游标，首次请求为空，后续使用响应中的 cursor/strCursor 值", example = "")
    ),
    responses(
        (status = 200, description = "新增关注通知列表", body = api::notification::connections::ConnectionsResponse)
    )
)]
pub async fn connections_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<api::notification::connections::ConnectionsParams>,
) -> impl IntoResponse {
    match api::notification::connections::get_connections_with_params(&state.api, params).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

/// 通知页-赞和收藏
/// 
/// 获取赞和收藏通知列表，支持分页
#[utoipa::path(
    get,
    path = "/api/notification/likes",
    tag = "xhs",
    summary = "通知页-赞和收藏",
    params(
        ("num" = Option<i32>, Query, description = "每页数量，固定为 20", example = 20),
        ("cursor" = Option<String>, Query, description = "分页游标，首次请求为空，后续使用响应中的 cursor/strCursor 值", example = "")
    ),
    responses(
        (status = 200, description = "赞和收藏通知列表", body = api::notification::likes::LikesResponse)
    )
)]
pub async fn likes_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<api::notification::likes::LikesParams>,
) -> impl IntoResponse {
    match api::notification::likes::get_likes_with_params(&state.api, params).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}
