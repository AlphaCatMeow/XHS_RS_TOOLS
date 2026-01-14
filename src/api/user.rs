use crate::api::XhsApiClient;
use crate::models::user::UserMeResponse;
use anyhow::Result;

/// 页面-我
/// 
/// 获取当前登录用户的个人信息
/// 
/// 使用 Python 端捕获的 user_me 签名发送请求
pub async fn get_current_user(api: &XhsApiClient) -> Result<UserMeResponse> {
    // 使用公共模块的 get 方法，自动处理签名和 headers
    let text = api.get("user_me").await?;
    
    let result = serde_json::from_str::<UserMeResponse>(&text)?;
    Ok(result)
}
