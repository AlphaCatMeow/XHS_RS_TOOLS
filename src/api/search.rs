use crate::api::XhsApiClient;
use crate::models::search::QueryTrendingResponse;
use anyhow::Result;

/// 猜你想搜
/// 
/// 获取小红书首页搜索框的热门搜索推荐词
pub async fn query_trending(api: &XhsApiClient) -> Result<QueryTrendingResponse> {
    let text = api.get("search_trending").await?;
    let result = serde_json::from_str::<QueryTrendingResponse>(&text)?;
    Ok(result)
}
