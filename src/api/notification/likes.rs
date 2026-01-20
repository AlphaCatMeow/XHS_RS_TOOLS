use crate::api::XhsApiClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Likes request parameters (赞和收藏 请求参数)
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct LikesParams {
    /// 每页数量，固定为 20
    #[serde(default = "default_num")]
    #[schema(default = 20, minimum = 1, maximum = 50)]
    pub num: i32,
    
    /// 分页游标，首次请求为空，后续使用响应中的 cursor 值
    #[serde(default)]
    #[schema(default = "", nullable = true)]
    pub cursor: Option<String>,
}

fn default_num() -> i32 {
    20
}

impl Default for LikesParams {
    fn default() -> Self {
        Self {
            num: 20,
            cursor: None,
        }
    }
}

/// Likes response (赞和收藏 通知)
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LikesResponse {
    pub success: bool,
    pub msg: String,
    pub data: Option<LikesData>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LikesData {
    /// 通知消息列表
    pub message_list: Vec<serde_json::Value>,
    /// 是否有更多数据
    #[serde(default)]
    pub has_more: bool,
    /// 下一页游标 (数值型)
    #[serde(default)]
    pub cursor: i64,
    /// 下一页游标 (字符串型，推荐使用)
    #[serde(default, rename = "strCursor")]
    pub str_cursor: Option<String>,
}

/// 通知页-赞和收藏 (默认参数)
/// 
/// 获取赞和收藏通知列表，使用默认分页参数
pub async fn get_likes(api: &XhsApiClient) -> Result<LikesResponse> {
    get_likes_with_params(api, LikesParams::default()).await
}

/// 通知页-赞和收藏 (自定义参数)
/// 
/// 获取赞和收藏通知列表，支持自定义分页参数
/// 
/// # Arguments
/// * `api` - API 客户端
/// * `params` - 分页参数 (num, cursor)
pub async fn get_likes_with_params(api: &XhsApiClient, params: LikesParams) -> Result<LikesResponse> {
    // 构建 URI
    let cursor = params.cursor.unwrap_or_default();
    let uri = format!("/api/sns/web/v1/you/likes?num={}&cursor={}", params.num, cursor);
    
    let text = api.get_with_query(&uri).await?;
    let result = serde_json::from_str::<LikesResponse>(&text)?;
    Ok(result)
}
