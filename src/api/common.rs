//! 公共 API 请求模块
//! 
//! 所有 XHS API 接口共享相同的认证层结构（Cookie + Headers + Signature），
//! 但各接口的签名数据是独立存储的。
//! 
//! 此模块抽离了"读取签名 → 构建 Headers → 发送请求"的公共逻辑，
//! 使得每个具体接口只需关注 URL 和 Payload 的构造。

use crate::auth::AuthService;
use crate::auth::credentials::ApiSignature;
use crate::client::XhsClient;
use anyhow::{Result, anyhow};
use std::sync::Arc;

const ORIGIN: &str = "https://www.xiaohongshu.com";
const REFERER: &str = "https://www.xiaohongshu.com/";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

/// XHS API 公共客户端
/// 
/// 封装了所有 API 请求的公共逻辑：
/// - 从 AuthService 获取 Cookie 和签名
/// - 构建标准浏览器 Headers
/// - 添加 XHS 特定签名 Headers
pub struct XhsApiClient {
    http_client: XhsClient,
    auth: Arc<AuthService>,
}

impl XhsApiClient {
    /// 创建新的 API 客户端
    pub fn new(http_client: XhsClient, auth: Arc<AuthService>) -> Self {
        Self { http_client, auth }
    }

    /// 获取认证服务引用
    pub fn auth(&self) -> &Arc<AuthService> {
        &self.auth
    }

    /// 执行 GET 请求
    /// 
    /// 使用捕获的签名和完整 URL 发送请求
    /// 
    /// # Arguments
    /// * `endpoint_key` - 签名存储的 key（如 "search_trending", "notification_mentions"）
    /// 
    /// # Returns
    /// 响应文本内容
    pub async fn get(&self, endpoint_key: &str) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        let signature = self.get_signature(endpoint_key).await?;
        
        let url = signature.request_url.clone()
            .ok_or_else(|| anyhow!("No request_url found for endpoint: {}", endpoint_key))?;
        
        tracing::info!("[XhsApiClient] GET {} using signature: {}", endpoint_key, &url[..url.len().min(80)]);
        
        let response = self.build_get_request(&url, &signature, &credentials.cookie_string())
            .send()
            .await?;
        
        self.handle_response(response, endpoint_key).await
    }

    /// 执行带自定义 URL 的 GET 请求
    /// 
    /// 用于需要动态构造 URL 参数的接口（如 note_comments）
    /// 
    /// # Arguments
    /// * `endpoint_key` - 签名存储的 key
    /// * `url` - 完整的请求 URL（含查询参数）
    pub async fn get_with_url(&self, endpoint_key: &str, url: &str) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        let signature = self.get_signature(endpoint_key).await?;
        
        tracing::info!("[XhsApiClient] GET {} with custom URL: {}", endpoint_key, &url[..url.len().min(80)]);
        
        let response = self.build_get_request(url, &signature, &credentials.cookie_string())
            .send()
            .await?;
        
        self.handle_response(response, endpoint_key).await
    }

    /// 执行 POST 请求
    /// 
    /// 使用捕获的签名和 body 发送请求
    /// 
    /// # Arguments
    /// * `endpoint_key` - 签名存储的 key（如 "home_feed_recommend"）
    pub async fn post(&self, endpoint_key: &str) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        let signature = self.get_signature(endpoint_key).await?;
        
        let url = signature.request_url.clone()
            .unwrap_or_else(|| format!("https://edith.xiaohongshu.com/api/sns/web/v1/{}", endpoint_key));
        
        let body = signature.post_body.clone().unwrap_or_default();
        
        tracing::info!("[XhsApiClient] POST {} body_len: {}", endpoint_key, body.len());
        
        let response = self.build_post_request(&url, &signature, &credentials.cookie_string(), body)
            .send()
            .await?;
        
        self.handle_response(response, endpoint_key).await
    }

    /// 执行带自定义 body 的 POST 请求
    /// 
    /// 用于需要动态构造请求体的接口
    pub async fn post_with_body(&self, endpoint_key: &str, url: &str, body: String) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        let signature = self.get_signature(endpoint_key).await?;
        
        tracing::info!("[XhsApiClient] POST {} with custom body_len: {}", endpoint_key, body.len());
        
        let response = self.build_post_request(url, &signature, &credentials.cookie_string(), body)
            .send()
            .await?;
        
        self.handle_response(response, endpoint_key).await
    }

    // ==================== 私有辅助方法 ====================

    /// 获取指定接口的签名
    async fn get_signature(&self, endpoint_key: &str) -> Result<ApiSignature> {
        self.auth.get_endpoint_signature(endpoint_key).await?
            .ok_or_else(|| anyhow!(
                "No signature found for endpoint: {}. Please login again to capture signatures.", 
                endpoint_key
            ))
    }

    /// 构建 GET 请求（含所有 headers）
    fn build_get_request(&self, url: &str, signature: &ApiSignature, cookie: &str) -> reqwest::RequestBuilder {
        self.http_client.get_client()
            .get(url)
            // Standard browser headers
            .header("accept", "application/json, text/plain, */*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            // Security headers
            .header("sec-ch-ua", r#""Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("user-agent", USER_AGENT)
            // XHS specific headers
            .header("origin", ORIGIN)
            .header("referer", REFERER)
            .header("x-s", &signature.x_s)
            .header("x-t", &signature.x_t)
            .header("x-s-common", &signature.x_s_common)
            .header("x-b3-traceid", &signature.x_b3_traceid)
            .header("x-xray-traceid", &signature.x_xray_traceid)
            .header("cookie", cookie)
    }

    /// 构建 POST 请求（含所有 headers）
    fn build_post_request(&self, url: &str, signature: &ApiSignature, cookie: &str, body: String) -> reqwest::RequestBuilder {
        self.http_client.get_client()
            .post(url)
            // Standard browser headers
            .header("accept", "application/json, text/plain, */*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("content-type", "application/json;charset=UTF-8")
            .header("priority", "u=1, i")
            // Security headers
            .header("sec-ch-ua", r#""Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("user-agent", USER_AGENT)
            // XHS specific headers
            .header("origin", ORIGIN)
            .header("referer", REFERER)
            .header("x-s", &signature.x_s)
            .header("x-t", &signature.x_t)
            .header("x-s-common", &signature.x_s_common)
            .header("x-b3-traceid", &signature.x_b3_traceid)
            .header("x-xray-traceid", &signature.x_xray_traceid)
            .header("xy-direction", "98")
            .header("cookie", cookie)
            .body(body)
    }

    /// 处理响应（日志 + 406 警告）
    async fn handle_response(&self, response: reqwest::Response, endpoint_key: &str) -> Result<String> {
        let status = response.status();
        let text = response.text().await?;
        
        tracing::info!("[XhsApiClient] {} Response [{}]: {} chars", endpoint_key, status, text.len());
        
        if status.as_u16() == 406 {
            tracing::warn!(
                "[XhsApiClient] {} received 406 - signature may be invalid (cookies are still valid)",
                endpoint_key
            );
        }
        
        Ok(text)
    }
}
