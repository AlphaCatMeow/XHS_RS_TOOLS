use crate::api::XhsApiClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Mentions request parameters (评论和@ 请求参数)
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct MentionsParams {
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

impl Default for MentionsParams {
    fn default() -> Self {
        Self {
            num: 20,
            cursor: None,
        }
    }
}

/// Mentions response (评论和@ 通知)
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MentionsResponse {
    pub code: Option<i32>,
    pub success: bool,
    pub msg: String,
    pub data: Option<MentionsData>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MentionsData {
    /// 下一页游标 (数值型)
    pub cursor: Option<i64>,
    /// 下一页游标 (字符串型，推荐使用)
    #[serde(rename = "strCursor")]
    pub str_cursor: Option<String>,
    /// 是否有更多数据
    #[serde(default)]
    pub has_more: bool,
    /// 通知消息列表
    pub message_list: Vec<serde_json::Value>,
}

/// 通知页-评论和@ (默认参数)
/// 
/// 获取评论和@通知列表，使用默认分页参数
pub async fn get_mentions(api: &XhsApiClient) -> Result<MentionsResponse> {
    get_mentions_with_params(api, MentionsParams::default()).await
}

/// 通知页-评论和@ (自定义参数)
/// 
/// 获取评论和@通知列表，支持自定义分页参数
/// 
/// # Arguments
/// * `api` - API 客户端
/// * `params` - 分页参数 (num, cursor)
pub async fn get_mentions_with_params(api: &XhsApiClient, params: MentionsParams) -> Result<MentionsResponse> {
    // 构建 URI
    let cursor = params.cursor.unwrap_or_default();
    let uri = format!("/api/sns/web/v1/you/mentions?num={}&cursor={}", params.num, cursor);
    
    let text = api.get_with_query(&uri).await?;
    let result = serde_json::from_str::<MentionsResponse>(&text)?;
    Ok(result)
}

