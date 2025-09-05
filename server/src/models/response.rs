use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T> {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(message: &str, payload: T) -> Self {
        Self {
            message: message.to_string(),
            payload: Some(payload),
        }
    }
}

impl ApiResponse<()> {
    pub fn success_no_payload(message: &str) -> Self {
        Self {
            message: message.to_string(),
            payload: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            message: message.to_string(),
            payload: None,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_success_response_with_payload() {
        let response = ApiResponse::success("Success", "test_payload");
        assert_eq!(response.message, "Success");
        assert_eq!(response.payload, Some("test_payload"));
    }

    #[test]
    fn test_success_response_no_payload() {
        let response = ApiResponse::success_no_payload("Success");
        assert_eq!(response.message, "Success");
        assert_eq!(response.payload, None);
    }

    #[test]
    fn test_error_response() {
        let response = ApiResponse::error("Error occurred");
        assert_eq!(response.message, "Error occurred");
        assert_eq!(response.payload, None);
    }
}
