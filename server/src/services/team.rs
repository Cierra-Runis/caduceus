use bson::oid::ObjectId;
use derive_more::Display;
use time::OffsetDateTime;

use crate::{
    models::team::{Team, TeamPayload},
    repo::{team::TeamRepo, user::UserRepo},
};

pub struct TeamService<R: TeamRepo, U: UserRepo> {
    pub user_repo: U,
    pub team_repo: R,
}

#[derive(Debug, Display)]
pub enum TeamServiceError {
    #[display("Invalid user ID")]
    InvalidUserId,
    #[display("User not found")]
    UserNotFound,
    #[display("Database error: {_0}")]
    Database(mongodb::error::Error),
}

impl<R: TeamRepo, U: UserRepo> TeamService<R, U> {
    pub async fn create(
        &self,
        creator_id: String,
        name: String,
    ) -> Result<TeamPayload, TeamServiceError> {
        let id = ObjectId::parse_str(&creator_id).map_err(|_| TeamServiceError::InvalidUserId)?;
        match self.user_repo.find_by_id(id).await {
            Ok(Some(_)) => {}
            Ok(None) => return Err(TeamServiceError::UserNotFound),
            Err(e) => return Err(TeamServiceError::Database(e)),
        };

        let team = self
            .team_repo
            .create(Team {
                id: ObjectId::new(),
                name: name.clone(),
                creator_id: id,
                member_ids: vec![id],
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),
            })
            .await
            .map_err(TeamServiceError::Database)?;

        Ok(team.into())
    }


}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::repo::{team::tests::MockTeamRepo, user::tests::MockUserRepo};
    use std::sync::Mutex;

    #[tokio::test]
    async fn test_create_success() {
        let user_repo = MockUserRepo {
            users: Mutex::new(vec![crate::models::user::User {
                id: ObjectId::parse_str("64b64c4f2f9b256e1c8e4d3a").unwrap(),
                username: "test_user".to_string(),
                nickname: "Test User".to_string(),
                password: "hashed_password".to_string(),
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),
            }]),
        };
        let team_repo = MockTeamRepo {
            teams: Mutex::new(vec![]),
        };
        let service = TeamService {
            user_repo,
            team_repo,
        };

        let result = service
            .create(
                "64b64c4f2f9b256e1c8e4d3a".to_string(),
                "Test Team".to_string(),
            )
            .await;
        assert!(result.is_ok());
        let payload = result.unwrap();
        assert_eq!(payload.name, "Test Team");
    }

    #[tokio::test]
    async fn test_create_user_not_found() {
        let user_repo = MockUserRepo {
            users: Mutex::new(vec![]),
        };
        let team_repo = MockTeamRepo {
            teams: Mutex::new(vec![]),
        };
        let service = TeamService {
            user_repo,
            team_repo,
        };

        let result = service
            .create(
                "64b64c4f2f9b256e1c8e4d3a".to_string(),
                "Test Team".to_string(),
            )
            .await;

        assert!(matches!(result, Err(TeamServiceError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_create_invalid_user_id() {
        let user_repo = MockUserRepo {
            users: Mutex::new(vec![]),
        };
        let team_repo = MockTeamRepo {
            teams: Mutex::new(vec![]),
        };
        let service = TeamService {
            user_repo,
            team_repo,
        };

        let result = service
            .create("invalid_object_id".to_string(), "Test Team".to_string())
            .await;
        assert!(matches!(result, Err(TeamServiceError::InvalidUserId)));
    }
}
