#[derive(FromRow, Debug, Clone)]
pub struct CandidateIntake {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
    pub populist_url: Option<String>,
    pub crated_by: uuid::Uuid,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
