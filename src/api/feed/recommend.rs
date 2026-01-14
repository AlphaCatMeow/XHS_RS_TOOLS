use crate::api::XhsApiClient;
use crate::models::feed::HomefeedResponse;
use anyhow::Result;

/// 页面-主页发现-推荐
/// 
/// 获取小红书主页推荐内容流
pub async fn get_homefeed_recommend(api: &XhsApiClient) -> Result<HomefeedResponse> {
    let text = api.post("home_feed_recommend").await?;
    let result = serde_json::from_str::<HomefeedResponse>(&text)?;
    Ok(result)
}
