use async_graphql::{ComplexObject, Context, Enum, FieldResult, SimpleObject, ID};
use db::{
    models::{
        enums::{PoliticalParty, State},
        politician::Politician,
    },
    DateTime,
};
use sqlx::{Pool, Postgres};

use super::OrganizationResult;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum OfficeType {
    House,
    Senate,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PoliticianResult {
    id: ID,
    slug: String,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    nickname: Option<String>,
    preferred_name: Option<String>,
    ballot_name: Option<String>,
    description: Option<String>,
    home_state: State,
    website_url: Option<String>,
    twitter_url: Option<String>,
    facebook_url: Option<String>,
    instagram_url: Option<String>,
    office_party: Option<PoliticalParty>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[ComplexObject]
impl PoliticianResult {
    async fn full_name(&self) -> String {
        match &self.middle_name {
            Some(middle_name) => format!(
                "{} {} {}",
                &self.first_name,
                middle_name.to_string(),
                &self.last_name
            ),
            None => format!("{} {}", &self.first_name, &self.last_name),
        }
    }

    async fn endorsements(&self, ctx: &Context<'_>) -> FieldResult<Vec<OrganizationResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records =
            Politician::endorsements(pool, uuid::Uuid::parse_str(&self.id).unwrap()).await?;
        let results = records
            .into_iter()
            .map(|r| OrganizationResult::from(r))
            .collect();
        Ok(results)
    }
}

impl From<Politician> for PoliticianResult {
    fn from(p: Politician) -> Self {
        Self {
            id: ID::from(p.id),
            slug: p.slug,
            first_name: p.first_name,
            middle_name: p.middle_name,
            last_name: p.last_name,
            nickname: p.nickname,
            preferred_name: p.preferred_name,
            ballot_name: p.ballot_name,
            description: p.description,
            home_state: p.home_state,
            website_url: p.website_url,
            twitter_url: p.twitter_url,
            facebook_url: p.facebook_url,
            instagram_url: p.instagram_url,
            office_party: p.office_party,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
