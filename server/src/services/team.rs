use crate::repo::team::TeamRepo;

pub struct TeamService<R: TeamRepo> {
    repo: R,
}

pub enum TeamServiceError {}

pub struct TeamPayload {
    pub id: String,
    pub name: String,
}

impl<R: TeamRepo> TeamService<R> {
    pub async fn create_team(&self, name: String) -> Result<TeamPayload, TeamServiceError> {
        unimplemented!()
    }
}
