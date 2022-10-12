use super::RaceResult;
use crate::context::ApiContext;
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use auth::Claims;
use db::{
    models::enums::{PoliticalParty, RaceType, State},
    Address, Election, Race,
};
use jsonwebtoken::TokenData;

#[derive(SimpleObject, Clone, Debug)]
#[graphql(complex)]
pub struct ElectionResult {
    id: ID,
    slug: String,
    title: String,
    description: Option<String>,
    state: Option<State>,
    election_date: chrono::NaiveDate,
}

#[ComplexObject]
impl ElectionResult {
    async fn races(&self, ctx: &Context<'_>) -> Result<Vec<RaceResult>> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let records = sqlx::query_as!(
            Race,
            r#"
            SELECT
                id,
                slug,
                title,
                office_id,
                race_type AS "race_type:RaceType",
                party AS "party:PoliticalParty",
                state AS "state:State",
                description,
                ballotpedia_link,
                early_voting_begins_date,
                winner_id,
                total_votes,
                official_website,
                election_id,
                is_special_election,
                num_elect,
                created_at,
                updated_at
            FROM
                race
            WHERE
                election_id = $1
            ORDER BY title DESC
            "#,
            uuid::Uuid::parse_str(&self.id).unwrap()
        )
        .fetch_all(&db_pool)
        .await
        .unwrap();

        let results = records.into_iter().map(RaceResult::from).collect();
        Ok(results)
    }

    /// Show races relevant to the user based on their address
    async fn races_by_user_districts(&self, ctx: &Context<'_>) -> Result<Vec<RaceResult>> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let token = ctx.data::<Option<TokenData<Claims>>>();

        if let Some(token_data) = token.unwrap() {
            let user_address_data = sqlx::query!(
                r#"
                SELECT
                    a.congressional_district,
                    a.state_senate_district,
                    a.state_house_district,
                    a.state AS "state:State",
                    a.county,
                    a.city
                FROM
                    address AS a
                    JOIN user_profile up ON user_id = $1
                WHERE
                    up.user_id = $1 AND 
                    up.address_id = a.id
                "#,
                token_data.claims.sub
            )
            .fetch_one(&db_pool)
            .await?;

            let user_address_extended_mn_data = if user_address_data.state == State::MN {
                let ext = Address::extended_mn_by_user_id(&db_pool, &token_data.claims.sub).await?;
                tracing::info!("extended address MN: {:?}", ext);
                ext
            } else {
                None
            };

            let records = sqlx::query_as!(
                Race,
                r#"
            SELECT
                r.id,
                r.slug,
                r.title,
                r.office_id,
                r.race_type AS "race_type:RaceType",
                r.party AS "party:PoliticalParty",
                r.state AS "state:State",
                r.description,
                r.ballotpedia_link,
                r.early_voting_begins_date,
                r.winner_id,
                r.total_votes,
                r.official_website,
                r.election_id,
                r.is_special_election,
                r.num_elect,
                r.created_at,
                r.updated_at
            FROM
                race r
                JOIN office o ON office_id = o.id
            WHERE
                r.election_id = $1 AND (
                    o.election_scope = 'national'
                    OR (o.state = $2 AND o.election_scope = 'state')
                    OR (o.state = $2 AND (  
                        -- Delete this line when we populate colorado counties from municipalities
                        (o.election_scope = 'county' AND o.municipality = $3) OR
                        (o.election_scope = 'county' AND o.county = $3) OR
                        (o.election_scope = 'city' AND o.municipality = $4) OR
                        (o.election_scope = 'district' AND o.district_type = 'us_congressional' AND o.district = $5) OR
                        (o.election_scope = 'district' AND o.district_type = 'state_senate' AND o.district = $6) OR
                        (o.election_scope = 'district' AND o.district_type = 'state_house' AND o.district = $7) OR
                        (o.election_scope = 'district' AND o.district_type = 'county' AND o.district = $8) 
                    )))
            ORDER BY o.priority ASC, title DESC
                "#,
                uuid::Uuid::parse_str(&self.id).unwrap(),
                user_address_data.state as State,
            user_address_data.county.map(|c| c.to_string().replace(" County", "")),
                user_address_data.city,
                user_address_data.congressional_district,
                user_address_data.state_senate_district,
                user_address_data.state_house_district,
                user_address_extended_mn_data.map(|a| a.county_commissioner_district).unwrap_or(None),
            )
            .fetch_all(&db_pool)
            .await?;
            let results = records.into_iter().map(RaceResult::from).collect();
            Ok(results)
        } else {
            tracing::debug!("No races found with user address data");
            Err("No user address data found".into())
        }
    }

    async fn races_by_voting_guide(
        &self,
        ctx: &Context<'_>,
        voting_guide_id: ID,
    ) -> Result<Vec<RaceResult>> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();

        // TODO: sql trigger that auto deletes voting guide candidate records
        // when is_endorsement = false and note is null

        let records = sqlx::query_as!(
            Race,
            r#"
            SELECT
                DISTINCT id,
                slug,
                title,
                office_id,
                race_type AS "race_type:RaceType",
                party AS "party:PoliticalParty",
                state AS "state:State",
                description,
                ballotpedia_link,
                early_voting_begins_date,
                winner_id,
                total_votes,
                official_website,
                election_id,
                is_special_election,
                num_elect,
                r.created_at,
                r.updated_at
            FROM
                race r
            JOIN race_candidates rc ON rc.race_id = r.id
            JOIN voting_guide_candidates vgc ON vgc.candidate_id = rc.candidate_id
            WHERE
                r.election_id = $1 AND
                vgc.voting_guide_id = $2 AND 
                (vgc.is_endorsement = true OR vgc.note IS NOT NULL)
            ORDER BY title DESC
            "#,
            uuid::Uuid::parse_str(&self.id).unwrap(),
            uuid::Uuid::parse_str(&voting_guide_id).unwrap()
        )
        .fetch_all(&db_pool)
        .await
        .unwrap();

        let results = records.into_iter().map(RaceResult::from).collect();
        Ok(results)
    }
}

impl From<Election> for ElectionResult {
    fn from(e: Election) -> Self {
        Self {
            id: ID::from(e.id),
            slug: e.slug,
            title: e.title,
            description: e.description,
            state: e.state,
            election_date: e.election_date,
        }
    }
}
