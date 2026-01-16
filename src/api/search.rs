use anyhow::Result;
use crate::api::XhsApiClient;
use crate::models::search::*;
use rand::{Rng, distributions::Alphanumeric};
use std::time::{SystemTime, UNIX_EPOCH};

/// 猜你想搜
/// 
/// 获取小红书首页搜索框的热门搜索推荐词
pub async fn query_trending(api: &XhsApiClient) -> Result<QueryTrendingResponse> {
    let text = api.get("search_trending").await?;
    let result = serde_json::from_str::<QueryTrendingResponse>(&text)?;
    Ok(result)
}

/// 搜索推荐 (联想词)
/// 
/// 根据关键词获取搜索建议
pub async fn recommend_search(api: &XhsApiClient, keyword: &str) -> Result<SearchRecommendResponse> {
    let encoded_keyword = urlencoding::encode(keyword);
    let url = format!("https://edith.xiaohongshu.com/api/sns/web/v1/search/recommend?keyword={}", encoded_keyword);
    
    // 使用 get_with_url 处理动态参数并进行纯算法签名
    let text = api.get_with_url("search_recommend", &url).await?;
    let result = serde_json::from_str::<SearchRecommendResponse>(&text)?;
    Ok(result)
}

/// 生成 Search ID (22位随机小写字母数字)
pub fn generate_search_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(22)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}

/// 生成 Request ID (10位随机数-时间戳)
fn generate_request_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let prefix: u64 = rand::thread_rng().gen_range(1_000_000_000..10_000_000_000);
    format!("{}-{}", prefix, now)
}

/// 搜索笔记列表
pub async fn search_notes(api: &XhsApiClient, mut req: SearchNotesRequest) -> Result<SearchNotesResponse> {
    // 补全 search_id
    if req.search_id.is_none() {
        req.search_id = Some(generate_search_id());
    }
    
    let path = "/api/sns/web/v1/search/notes";
    let payload = serde_json::to_value(&req)?;
    
    // 使用 post_algo 进行签名和发送
    let text = api.post_algo(path, payload).await?;
    let result = serde_json::from_str::<SearchNotesResponse>(&text)?;
    Ok(result)
}

/// 搜索 OneBox (聚合结果)
pub async fn search_onebox(api: &XhsApiClient, mut req: SearchOneboxRequest) -> Result<SearchOneboxResponse> {
    // 补全 request_id
    if req.request_id.is_none() {
        req.request_id = Some(generate_request_id());
    }
    
    let path = "/api/sns/web/v1/search/onebox";
    let payload = serde_json::to_value(&req)?;
    
    let text = api.post_algo(path, payload).await?;
    let result = serde_json::from_str::<SearchOneboxResponse>(&text)?;
    Ok(result)
}

/// 搜索筛选器
pub async fn search_filter(api: &XhsApiClient, keyword: &str, search_id: &str) -> Result<SearchFilterResponse> {
    let encoded_kw = urlencoding::encode(keyword);
    let encoded_sid = urlencoding::encode(search_id);
    let url = format!("https://edith.xiaohongshu.com/api/sns/web/v1/search/filter?keyword={}&search_id={}", encoded_kw, encoded_sid);
    
    // get_with_url 适用于任何 edith URL，只要路径正确即可
    let text = api.get_with_url("search_filter", &url).await?;
    let result = serde_json::from_str::<SearchFilterResponse>(&text)?;
    Ok(result)
}

/// 搜索用户列表
pub async fn search_user(api: &XhsApiClient, mut req: SearchUserRequest) -> Result<SearchUserResponse> {
    if req.search_id.is_none() {
        req.search_id = Some(generate_search_id());
    }
    if req.request_id.is_none() {
        req.request_id = Some(generate_request_id());
    }

    let path = "/api/sns/web/v1/search/usersearch";
    
    // 包装请求
    let request_wrapper = SearchUserRequestBody {
        search_user_request: req,
    };
    
    let payload = serde_json::to_value(&request_wrapper)?;
    let text = api.post_algo(path, payload).await?;
    let result = serde_json::from_str::<SearchUserResponse>(&text)?;
    Ok(result)
}
