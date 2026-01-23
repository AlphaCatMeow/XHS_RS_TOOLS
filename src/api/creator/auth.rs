//! Creator Center Authentication API
//!
//! Handles the QR code login process specifically for the Creator Center (creator.xiaohongshu.com).
//! Similar to the user login flow but operating in the 'ugc' context.

use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, ORIGIN, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::config::get_agent_url;
use crate::api::login::{
    QrCodeCreateResponse,
    AgentGuestCookiesResponse,
    AgentSignRequest,
    AgentSignResponse,
    QrCodeCreateData,
};

// ============================================================================
// Constants
// ============================================================================

const CREATOR_ORIGIN: &str = "https://creator.xiaohongshu.com";
const CREATOR_REFERER: &str = "https://creator.xiaohongshu.com/";
const XHS_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

// Creator QR Code API is actually the same endpoint on customer.xiaohongshu.com
const QRCODE_CREATE_URL: &str = "https://customer.xiaohongshu.com/api/cas/customer/web/qr-code";

// ============================================================================
// Core Functions
// ============================================================================

/// Fetch guest cookies for Creator Center options
///
/// Calls Agent with `target=creator` to initialize cookies on creator.xiaohongshu.com
pub async fn fetch_creator_guest_cookies() -> Result<HashMap<String, String>> {
    let client = reqwest::Client::new();
    // Use the new target parameter we added to agent_server.py
    let url = format!("{}/guest-cookies?target=creator", get_agent_url());
    
    tracing::info!("Fetching CREATOR guest cookies from Agent...");
    
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(45)) // Little extra buffer
        .send()
        .await
        .map_err(|e| anyhow!("Failed to connect to Agent: {}", e))?;
    
    let result: AgentGuestCookiesResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse Agent response: {}", e))?;
    
    if !result.success {
        return Err(anyhow!("Agent error: {}", result.error.unwrap_or_default()));
    }
    
    result.cookies.ok_or_else(|| anyhow!("No cookies returned"))
}

/// Sign request using Agent (same logic as user, but different endpoint/context)
async fn sign_request(
    cookies: &HashMap<String, String>,
    method: &str,
    uri: &str,
    payload: Option<serde_json::Value>,
) -> Result<(String, String, String)> {
    let client = reqwest::Client::new();
    let url = format!("{}/sign", get_agent_url());
    
    let request = AgentSignRequest {
        method: method.to_string(),
        uri: uri.to_string(),
        cookies: cookies.clone(),
        payload,
    };
    
    let response = client
        .post(&url)
        .json(&request)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| anyhow!("Failed to connect to Agent: {}", e))?;
    
    let result: AgentSignResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse signature response: {}", e))?;
    
    if !result.success {
        return Err(anyhow!("Sign error: {}", result.error.unwrap_or_default()));
    }
    
    Ok((
        result.x_s.unwrap_or_default(),
        result.x_t.unwrap_or_default(),
        result.x_s_common.unwrap_or_default(),
    ))
}

/// Build common headers for Creator API
fn build_creator_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert(ORIGIN, HeaderValue::from_static(CREATOR_ORIGIN));
    headers.insert(REFERER, HeaderValue::from_static(CREATOR_REFERER));
    headers.insert(USER_AGENT, HeaderValue::from_static(XHS_USER_AGENT));
    // CRITICAL: Creator Center / UGC context
    headers.insert("xsecappid", HeaderValue::from_static("ugc"));
    headers
}

fn cookies_to_string(cookies: &HashMap<String, String>) -> String {
    cookies
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Create QR code for Creator Center Login
pub async fn create_creator_qrcode(cookies: &HashMap<String, String>) -> Result<QrCodeCreateResponse> {
    let uri = "/api/cas/customer/web/qr-code";
    let payload = serde_json::json!({"service": "https://creator.xiaohongshu.com"});
    
    // Get signature
    let (x_s, x_t, x_s_common) = 
        sign_request(cookies, "POST", uri, Some(payload.clone())).await?;
    
    // Build request
    let mut headers = build_creator_headers();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json;charset=UTF-8"));
    headers.insert("x-s", HeaderValue::from_str(&x_s)?);
    headers.insert("x-t", HeaderValue::from_str(&x_t)?);
    headers.insert("x-s-common", HeaderValue::from_str(&x_s_common)?);
    headers.insert("cookie", HeaderValue::from_str(&cookies_to_string(cookies))?);
    
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    
    tracing::info!("Creating Creator QR code...");
    
    let response = client
        .post(QRCODE_CREATE_URL)
        .json(&payload)
        .send()
        .await?;
    
    let status = response.status();
    let text = response.text().await?;
    
    tracing::debug!("Creator QR Response [{}]: {}", status, text);
    
    if status.as_u16() >= 400 {
        return Err(anyhow!("API Error ({}): {}", status, text));
    }
    
    #[derive(Debug, Deserialize)]
    struct RawCreatorQrResponse {
        success: bool,
        code: i32,
        msg: Option<String>,
        data: Option<RawCreatorQrData>,
    }
    #[derive(Debug, Deserialize)]
    struct RawCreatorQrData {
        url: String,
        id: String, // Matched from actual API response
    }
    
    let raw: RawCreatorQrResponse = serde_json::from_str(&text)
        .map_err(|e| anyhow!("Parse error: {} - Body: {}", e, text))?;
        
    let data = raw.data.map(|d| QrCodeCreateData {
        url: d.url,
        qr_id: d.id, // Map 'id' to 'qr_id'
        code: "".to_string(),
    });
    
    Ok(QrCodeCreateResponse {
        success: raw.success,
        code: raw.code,
        msg: raw.msg,
        data,
    })
}

/// Check Creator QR Code Status
///
/// Polls the status of the QR code.
/// Status codes: 2 (Waiting), 3 (Scanned), others (Success?)
pub async fn check_creator_qrcode_status(
    qr_id: &str,
    cookies: &HashMap<String, String>
) -> Result<serde_json::Value> {
    let uri = "/api/cas/customer/web/qr-code";
    
    let service = "https://creator.xiaohongshu.com";
    // Construct query component specifically for signature and request
    let query = format!("service={}&qr_code_id={}&source=", urlencoding::encode(service), qr_id);
    let full_uri = format!("{}?{}", uri, query);
    
    // Get signature (GET request, no payload)
    let (x_s, x_t, x_s_common) = 
        sign_request(cookies, "GET", &full_uri, None).await?;
        
    let mut headers = build_creator_headers();
    headers.insert("x-s", HeaderValue::from_str(&x_s)?);
    headers.insert("x-t", HeaderValue::from_str(&x_t)?);
    headers.insert("x-s-common", HeaderValue::from_str(&x_s_common)?);
    headers.insert("cookie", HeaderValue::from_str(&cookies_to_string(cookies))?);
    
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
        
    let url = format!("{}?{}", QRCODE_CREATE_URL, query);
    
    tracing::debug!("Polling Creator QR: {}", url);
    
    let response = client
        .get(&url)
        .send()
        .await?;
        
    let status = response.status();
    let text = response.text().await?;
    
    if status.as_u16() >= 400 {
        return Err(anyhow!("API Error ({}): {}", status, text));
    }
    
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| anyhow!("Parse error: {} - Body: {}", e, text))?;
        
    Ok(json)
}