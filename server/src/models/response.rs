use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_model() {
        let success_response: Response<String> = Response {
            success: true,
            data: Some("test data".to_string()),
            message: None,
        };

        assert!(success_response.success);
        assert!(success_response.data.is_some());
        assert!(success_response.message.is_none());

        let error_response: Response<()> = Response {
            success: false,
            data: None,
            message: Some("Error message".to_string()),
        };

        assert!(!error_response.success);
        assert!(error_response.data.is_none());
        assert!(error_response.message.is_some());
    }
}
