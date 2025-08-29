use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub data: Option<T>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_model() {
        let success_response: Response<String> = Response {
            data: Some("test data".to_string()),
            message: "Success".to_string(),
        };

        assert!(success_response.data.is_some());
        assert!(success_response.message == "Success");

        let error_response: Response<()> = Response {
            data: None,
            message: "Error message".to_string(),
        };

        assert!(error_response.data.is_none());
        assert!(error_response.message == "Error message");
    }
}
