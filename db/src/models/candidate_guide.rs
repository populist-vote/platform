use uuid::Uuid;

#[derive(FromRow, Debug, Clone)]
pub struct CandidateGuide {
    pub id: Uuid,
    pub race_id: Uuid,
    pub candidate_question_set_id: Uuid,
    pub organization_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
