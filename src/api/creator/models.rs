//! Creator Center API Models
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Request body for creating Creator QR Code
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatorQrcodeCreateRequest {
    /// Guest cookies obtained from /api/creator/auth/guest-init
    #[schema(example = json!({"web_session": "xxxxx", "xsecappid": "ugc"}))]
    pub cookies: HashMap<String, String>,
}

/// Request body for polling Creator QR Code status
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatorQrcodeStatusRequest {
    /// QR Code ID returned from creation step
    #[schema(example = "68c517598657858235023360")]
    pub qr_id: String,
    
    /// Guest cookies
    #[schema(example = json!({"web_session": "xxxxx", "xsecappid": "ugc"}))]
    pub cookies: HashMap<String, String>,
}
