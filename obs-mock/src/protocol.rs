use serde::{Deserialize, Serialize};
use serde_json::Value;

// OpCodes
pub const OP_HELLO: u8 = 0;
pub const OP_IDENTIFY: u8 = 1;
pub const OP_IDENTIFIED: u8 = 2;
pub const OP_REIDENTIFY: u8 = 3;
pub const OP_EVENT: u8 = 5;
pub const OP_REQUEST: u8 = 6;
pub const OP_REQUEST_RESPONSE: u8 = 7;
pub const OP_REQUEST_BATCH: u8 = 8;
pub const OP_REQUEST_BATCH_RESPONSE: u8 = 9;

// Request status codes
pub const REQUEST_STATUS_SUCCESS: u16 = 100;
pub const REQUEST_STATUS_UNKNOWN: u16 = 600;

pub const RPC_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub op: u8,
    pub d: Value,
}

#[derive(Debug, Serialize)]
pub struct Hello {
    #[serde(rename = "obsWebSocketVersion")]
    pub obs_web_socket_version: String,
    #[serde(rename = "rpcVersion")]
    pub rpc_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<AuthChallenge>,
}

#[derive(Debug, Serialize)]
pub struct AuthChallenge {
    pub challenge: String,
    pub salt: String,
}

#[derive(Debug, Deserialize)]
pub struct Identify {
    #[serde(rename = "rpcVersion")]
    pub rpc_version: u32,
    pub authentication: Option<String>,
    #[serde(rename = "eventSubscriptions")]
    pub event_subscriptions: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Identified {
    #[serde(rename = "negotiatedRpcVersion")]
    pub negotiated_rpc_version: u32,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    #[serde(rename = "requestType")]
    pub request_type: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "requestData")]
    pub request_data: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct RequestResponse {
    #[serde(rename = "requestType")]
    pub request_type: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "requestStatus")]
    pub request_status: RequestStatus,
    #[serde(rename = "responseData", skip_serializing_if = "Option::is_none")]
    pub response_data: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct RequestStatus {
    pub result: bool,
    pub code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RequestBatch {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "haltOnFailure")]
    pub halt_on_failure: Option<bool>,
    #[allow(dead_code)]
    #[serde(rename = "executionType")]
    pub execution_type: Option<u8>,
    pub requests: Vec<Request>,
}

#[derive(Debug, Serialize)]
pub struct RequestBatchResponse {
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub results: Vec<RequestResponse>,
}

impl Message {
    pub fn new(op: u8, data: impl Serialize) -> Self {
        Self {
            op,
            d: serde_json::to_value(data).unwrap(),
        }
    }
}

impl RequestResponse {
    pub fn success(request_type: String, request_id: String, response_data: Option<Value>) -> Self {
        Self {
            request_type,
            request_id,
            request_status: RequestStatus {
                result: true,
                code: REQUEST_STATUS_SUCCESS,
                comment: None,
            },
            response_data,
        }
    }

    pub fn error(request_type: String, request_id: String, code: u16, comment: String) -> Self {
        Self {
            request_type,
            request_id,
            request_status: RequestStatus {
                result: false,
                code,
                comment: Some(comment),
            },
            response_data: None,
        }
    }
}
