use super::RaceResult;
use crate::{context::ApiContext, Error};
use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject, ID};
use auth::AccessTokenClaims;
use db::{
    models::enums::{RaceType, State, VoteType},
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

#[derive(InputObject, Default, Debug)]
pub struct ElectionRaceFilter {
    state: Option<State>,
}

#[ComplexObject]
impl ElectionResult {
    async fn races(
        &self,
        ctx: &Context<'_>,
        filter: Option<ElectionRaceFilter>,
    ) -> Result<Vec<RaceResult>> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let filter = filter.unwrap_or_default();
        let records = sqlx::query_as!(
            Race,
            r#"
            SELECT
                id,
                slug,
                title,
                office_id,
                race_type AS "race_type:RaceType",
                vote_type AS "vote_type:VoteType",
                party_id,
                state AS "state:State",
                description,
                ballotpedia_link,
                early_voting_begins_date,
                winner_ids,
                total_votes,
                num_precincts_reporting,
                total_precincts,
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
                AND ($2::state IS NULL OR state = $2)
            ORDER BY title DESC
            LIMIT 1000
            "#,
            uuid::Uuid::parse_str(&self.id).unwrap(),
            filter.state as Option<State>
        )
        .fetch_all(&db_pool)
        .await
        .unwrap();

        let results = records.into_iter().map(RaceResult::from).collect();
        Ok(results)
    }

    /// Show races relevant to the user based on their address
    async fn races_by_user_districts(&self, ctx: &Context<'_>) -> Result<Vec<RaceResult>, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let token = ctx.data::<Option<TokenData<AccessTokenClaims>>>();

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
                Address::extended_mn_by_user_id(&db_pool, &token_data.claims.sub).await?
            } else {
                None
            };

            let county_commissioner_district = user_address_extended_mn_data
                .clone()
                .map(|a| {
                    a.county_commissioner_district
                        .map(|d| d.as_str().trim_start_matches('0').to_string())
                })
                .unwrap_or(None);

            let school_district = user_address_extended_mn_data
                .clone()
                .map(|a| {
                    a.school_district_number
                        .map(|d| d.as_str().trim_start_matches('0').to_string())
                })
                .unwrap_or(None);

            let school_district_type = user_address_extended_mn_data
                .clone()
                .map(|a| a.school_district_type)
                .unwrap_or(None);

            let school_subdistrict = user_address_extended_mn_data
                .clone()
                .map(|a| {
                    a.school_subdistrict_code
                        .map(|d| d.as_str().trim_start_matches('0').to_string())
                })
                .unwrap_or(None);

            let ward = user_address_extended_mn_data
                .map(|a| {
                    a.ward.map(|d| {
                        // Remove non-numeric prefix and then trim leading zeros
                        if let Some(pos) = d.find('-') {
                            d[(pos + 1)..].trim_start_matches('0').to_string()
                        } else {
                            d.trim_start_matches('0').to_string()
                        }
                    })
                })
                .unwrap_or(None);

            let records = sqlx::query_as!(
                Race,
                r#"
            SELECT
                r.id,
                r.slug,
                r.title,
                r.office_id,
                r.race_type AS "race_type:RaceType",
                r.vote_type AS "vote_type:VoteType",
                r.party_id,
                r.state AS "state:State",
                r.description,
                r.ballotpedia_link,
                r.early_voting_begins_date,
                r.winner_ids,
                r.total_votes,
                r.num_precincts_reporting,
                r.total_precincts,
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
                        (o.election_scope = 'county' AND o.county = $4) OR
                        (o.election_scope = 'district' AND o.district_type = 'us_congressional' AND o.district = $5) OR
                        (o.election_scope = 'district' AND o.district_type = 'state_senate' AND o.district = $6) OR
                        (o.election_scope = 'district' AND o.district_type = 'state_house' AND o.district = $7) OR
                        (o.election_scope = 'district' AND o.district_type = 'county' AND o.county = $4 AND o.district = $8) OR
                        (o.election_scope = 'city' AND o.municipality = $3) OR
                        (o.election_scope = 'district' AND o.district_type = 'city' AND o.municipality = $3 AND o.district = $12) OR
                       (CASE 
                         WHEN $10 = '01' THEN
                          (o.election_scope = 'district' AND o.district_type = 'school' AND REPLACE(o.school_district, 'ISD #', '') = $9) AND
                          (o.election_scope = 'district' AND o.district_type = 'school' AND o.district IS NULL OR o.district = $11)
                         WHEN $10 = '03' THEN
                           (o.election_scope = 'district' AND o.district_type = 'school' AND REPLACE(o.school_district, 'SSD #', '') = $9) AND
                           (o.election_scope = 'district' AND o.district_type = 'school' AND o.district IS NULL OR o.district = $11)
                        END)
                    )))
            ORDER BY o.priority ASC, title DESC
                "#,
                uuid::Uuid::parse_str(&self.id).unwrap(),
                user_address_data.state as State,
                user_address_data.city,
                user_address_data.county.map(|c| c.to_string().replace(" County", "")),
                user_address_data.congressional_district,
                user_address_data.state_senate_district,
                user_address_data.state_house_district,
                county_commissioner_district,
                school_district,
                school_district_type,
                school_subdistrict,
                ward
            )
            .fetch_all(&db_pool)
            .await?;
            let results = records.into_iter().map(RaceResult::from).collect();
            Ok(results)
        } else {
            tracing::debug!("No races found with user address data");
            Err(Error::UserAddressNotFound)
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

        let records = sqlx::query!(
            r#"
            SELECT DISTINCT
                r.id,
                r.slug,
                r.title,
                office_id,
                race_type AS "race_type:RaceType",
                vote_type AS "vote_type:VoteType",
                party_id,
                r.state AS "state:State",
                r.description,
                ballotpedia_link,
                early_voting_begins_date,
                winner_ids,
                total_votes,
                num_precincts_reporting,
                total_precincts,
                official_website,
                election_id,
                is_special_election,
                num_elect,
                r.created_at,
                r.updated_at,
                o.priority 
            FROM
                race r
            JOIN office o ON o.id = r.office_id
            JOIN race_candidates rc ON rc.race_id = r.id
            JOIN voting_guide_candidates vgc ON vgc.candidate_id = rc.candidate_id
            WHERE
                r.election_id = $1 AND
                vgc.voting_guide_id = $2 AND 
                (vgc.is_endorsement = true OR vgc.note IS NOT NULL)
            ORDER BY o.priority ASC, r.title DESC
            "#,
            uuid::Uuid::parse_str(&self.id).unwrap(),
            uuid::Uuid::parse_str(&voting_guide_id).unwrap()
        )
        .fetch_all(&db_pool)
        .await
        .unwrap();

        let results = records
            .into_iter()
            .map(|r| Race {
                id: r.id,
                slug: r.slug,
                title: r.title,
                office_id: r.office_id,
                race_type: r.race_type,
                vote_type: r.vote_type,
                party_id: r.party_id,
                state: r.state,
                description: r.description,
                ballotpedia_link: r.ballotpedia_link,
                early_voting_begins_date: r.early_voting_begins_date,
                winner_ids: r.winner_ids,
                total_votes: r.total_votes,
                num_precincts_reporting: r.num_precincts_reporting,
                total_precincts: r.total_precincts,
                official_website: r.official_website,
                election_id: r.election_id,
                is_special_election: r.is_special_election,
                num_elect: r.num_elect,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .map(RaceResult::from)
            .collect();

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
