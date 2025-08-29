use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTeamRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
}
