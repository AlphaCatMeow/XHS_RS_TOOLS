use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::{self, XhsApiClient},
    auth::AuthService,
    client::XhsClient,
    models::{
        feed::{HomefeedRequest, HomefeedResponse, HomefeedData, HomefeedItem, NoteCard, NoteUser, NoteCover, CoverImageInfo, InteractInfo, NoteVideo, VideoCapa},
        login::{QrCodeSessionResponse, SessionInfoResponse, SessionInfoData, CookieInfo},
        search::{QueryTrendingResponse, QueryTrendingData, TrendingQuery, TrendingHintWord},
        user::{UserMeResponse, UserInfo},
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        query_trending_handler,
        user_me_handler,
        start_login_session_handler,
        get_session_handler,
        api::feed::category::get_category_feed,
        api::note::page::get_note_page,
        mentions_handler,
        connections_handler,
    ),
    components(
        schemas(
            QrCodeSessionResponse, SessionInfoResponse, SessionInfoData, CookieInfo,
            QueryTrendingResponse, QueryTrendingData, TrendingQuery, TrendingHintWord,
            UserMeResponse, UserInfo,
            HomefeedRequest, HomefeedResponse, HomefeedData, HomefeedItem, NoteCard, NoteUser, NoteCover, CoverImageInfo, InteractInfo, NoteVideo, VideoCapa
        )
    ),
    tags(
        (name = "xhs", description = "小红书 API 接口"),
        (name = "auth", description = "认证相关"),
        (name = "Feed", description = "主页发现频道：recommend(推荐)、fashion(穿搭)、food(美食)、cosmetics(彩妆)、movie_and_tv(影视)、career(职场)、love(情感)、household_product(家居)、gaming(游戏)、travel(旅行)、fitness(健身)"),
        (name = "Note", description = "笔记相关接口")
    )
)]
struct ApiDoc;

pub struct AppState {
    pub api: XhsApiClient,
    pub auth: Arc<AuthService>,
}



/// 猜你想搜
/// 
/// 获取小红书首页搜索框的热门搜索推荐词
#[utoipa::path(
    get,
    path = "/api/search/trending",
    tag = "xhs",
    summary = "猜你想搜",
    responses(
        (status = 200, description = "热门搜索词列表", body = QueryTrendingResponse)
    )
)]
async fn query_trending_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::search::query_trending(&state.api).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

/// 页面-我
/// 
/// 获取当前登录用户的个人信息
#[utoipa::path(
    get,
    path = "/api/user/me",
    tag = "xhs",
    summary = "页面-我",
    responses(
        (status = 200, description = "当前用户信息（未登录时返回 Not logged in）", body = UserMeResponse)
    )
)]
async fn user_me_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::user::get_current_user(&state.api).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}


/// 页面-主页发现-推荐
/// 
/// 获取小红书主页推荐内容流
#[utoipa::path(
    post,
    path = "/api/feed/homefeed/recommend",
    tag = "xhs",
    summary = "页面-主页发现-推荐",
    request_body = HomefeedRequest,
    responses(
        (status = 200, description = "主页推荐内容", body = HomefeedResponse)
    )
)]
async fn homefeed_recommend_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::feed::recommend::get_homefeed_recommend(&state.api).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}


/// Start Login Session (Streamed Response)
///
/// Returns a JSON stream. First message contains QR code. Subsequent messages stream login status updates.
#[utoipa::path(
    post,
    path = "/api/auth/login-session",
    tag = "auth",
    responses(
        (status = 200, description = "Login Session Stream (JSON Lines)", body = QrCodeSessionResponse)
    )
)]
async fn start_login_session_handler() -> impl IntoResponse {
    match api::login::start_login_session().await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })).into_response(),
    }
}

/// Get Current Session Info
///
/// View stored cookies and session details. Cookie values are masked for security.
#[utoipa::path(
    get,
    path = "/api/auth/session",
    tag = "auth",
    responses(
        (status = 200, description = "Current Session Information", body = SessionInfoResponse)
    )
)]
async fn get_session_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::login::get_session_info(&state.auth).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

/// 通知页-评论和@
/// 
/// 获取评论和@通知列表
#[utoipa::path(
    get,
    path = "/api/notification/mentions",
    tag = "xhs",
    summary = "通知页-评论和@",
    responses(
        (status = 200, description = "评论和@通知列表")
    )
)]
async fn mentions_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::notification::mentions::get_mentions(&state.api).await {
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
/// 获取新增关注通知列表
#[utoipa::path(
    get,
    path = "/api/notification/connections",
    tag = "xhs",
    summary = "通知页-新增关注",
    responses(
        (status = 200, description = "新增关注通知列表")
    )
)]
async fn connections_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::notification::connections::get_connections(&state.api).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

pub async fn start_server() -> anyhow::Result<()> {
    // Initialize MongoDB connection
    let mongodb_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    
    tracing::info!("Initializing AuthService with MongoDB...");
    let auth = Arc::new(AuthService::new(&mongodb_uri).await?);
    
    let client = XhsClient::new()?;
    let api = XhsApiClient::new(client, auth.clone());
    let state = Arc::new(AppState { api, auth });

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/search/trending", get(query_trending_handler))
        .route("/api/user/me", get(user_me_handler))
        .route("/api/feed/homefeed/recommend", post(homefeed_recommend_handler))
        .route("/api/feed/homefeed/:category", post(api::feed::category::get_category_feed))
        .route("/api/note/page", get(api::note::page::get_note_page))
        .route("/api/notification/mentions", get(mentions_handler))
        .route("/api/notification/connections", get(connections_handler))
        .route("/api/auth/login-session", post(start_login_session_handler))
        .route("/api/auth/session", get(get_session_handler))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    // Get port from environment variable, default to 3000
    let port = std::env::var("PORT")
        .or_else(|_| std::env::var("XHS_API_PORT"))
        .unwrap_or_else(|_| "3000".to_string());
    
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Server running on http://{}/swagger-ui/", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
