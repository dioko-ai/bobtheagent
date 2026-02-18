use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::capabilities::CapabilityId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RequestMetadata {
    #[serde(default)]
    pub transport: Option<String>,
    #[serde(default)]
    pub actor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestEnvelope<T> {
    #[serde(default)]
    pub request_id: Option<String>,
    pub capability: CapabilityId,
    #[serde(default)]
    pub metadata: RequestMetadata,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseEnvelope<T> {
    #[serde(default)]
    pub request_id: Option<String>,
    pub capability: CapabilityId,
    pub result: ApiResultEnvelope<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ApiResultEnvelope<T> {
    Ok { data: T },
    Err { error: ApiErrorEnvelope },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApiErrorCode {
    InvalidRequest,
    ValidationFailed,
    NotFound,
    Conflict,
    IoFailure,
    ExternalFailure,
    Unsupported,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiErrorEnvelope {
    pub code: ApiErrorCode,
    pub message: String,
    #[serde(default)]
    pub retryable: bool,
    #[serde(default)]
    pub details: Option<Value>,
}
