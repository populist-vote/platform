use super::{BallotMeasureResult, RaceResult};
use crate::{
    context::ApiContext,
    relay::{Base64Cursor, ConnectionFields, ConnectionResult},
    Error,
};
use async_graphql::{
    connection::{Connection, CursorType, Edge},
    ComplexObject, Context, InputObject, Result, SimpleObject, ID,
};
use auth::AccessTokenClaims;
use db::{
    filters::mn::apply_mn_filters,
    models::{
        ballot_measure::BallotMeasure,
        enums::{BallotMeasureStatus, RaceType, State, VoteType},
    },
    Address, AddressInput, Election, ElectionScope, Race,
};
use geocodio::GeocodioProxy;
use jsonwebtoken::TokenData;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

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
    query: Option<String>,
}

pub async fn process_address_with_geocodio(
    db_pool: &sqlx::PgPool,
    address: AddressInput,
) -> Result<Uuid, Error> {
    let address_clone = address.clone();
    let geocodio = GeocodioProxy::new().unwrap();

    let existing_address = sqlx::query!(
        r#"
        SELECT
            id
        FROM
            address
        WHERE
            line_1 = $1 AND
            line_2 = $2 AND
            city = $3 AND
            state = $4 AND
            country = $5 AND
            postal_code = $6
        "#,
        address.line_1,
        address.line_2,
        address.city,
        address.state.to_string(),
        address.country,
        address.postal_code
    )
    .fetch_optional(db_pool)
    .await?;

    if let Some(address) = existing_address {
        return Ok(address.id);
    }

    // Process address with geocodio
    let geocode_result = geocodio
        .geocode(
            geocodio::AddressParams::AddressInput(geocodio::AddressInput {
                line_1: address.line_1,
                line_2: address.line_2,
                city: address.city,
                state: address.state.to_string(),
                country: address.country,
                postal_code: address.postal_code,
            }),
            Some(&["cd118", "stateleg-next"]),
        )
        .await;

    if let Ok(geocodio_data) = geocode_result {
        let city = geocodio_data.results[0]
            .address_components
            .city
            .clone()
            .unwrap_or(address_clone.city);
        let coordinates = geocodio_data.results[0].location.clone();
        let county = geocodio_data.results[0].address_components.county.clone();
        let primary_result = geocodio_data.results[0].fields.as_ref().unwrap();
        let congressional_district =
            &primary_result.congressional_districts.as_ref().unwrap()[0].district_number;
        let state_legislative_districts =
            primary_result.state_legislative_districts.as_ref().unwrap();
        let state_house_district = &state_legislative_districts.house[0].district_number;
        let state_senate_district = &state_legislative_districts.senate[0].district_number;

        let temp_address_record = sqlx::query!(r#"
                    INSERT INTO address (line_1, line_2, city, state, county, country, postal_code, lon, lat, geog, geom, congressional_district, state_senate_district, state_house_district)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, ST_SetSRID(ST_MakePoint($8, $9), 4326), ST_GeomFromText($10, 4326), $11, $12, $13)
                    ON CONFLICT (line_1, line_2, city, state, country, postal_code) -- adjust the conflict target columns as per your unique constraint
                    DO UPDATE SET
                        lon = EXCLUDED.lon,
                        lat = EXCLUDED.lat,
                        geog = EXCLUDED.geog,
                        geom = EXCLUDED.geom,
                        congressional_district = EXCLUDED.congressional_district,
                        state_senate_district = EXCLUDED.state_senate_district,
                        state_house_district = EXCLUDED.state_house_district
                    RETURNING id

            "#, 
            address_clone.line_1,
            address_clone.line_2,
            city,
            address_clone.state.to_string(),
            county,
            address_clone.country,
            address_clone.postal_code,
            coordinates.longitude,
            coordinates.latitude,
            format!("POINT({} {})", coordinates.longitude, coordinates.latitude), // A string we pass into ST_GeomFromText function
            &congressional_district.to_string(),
            state_senate_district,
            state_house_district
            ).fetch_one(db_pool).await?;

        let address_id = temp_address_record.id;

        // TODO - Clean up and delete temp address record in separate thread
        // Need to determine if new address was created or existing address was updated so we
        // don't delete an address that is still in use
        // tokio::spawn(async move {
        //     if let Err(err) = sqlx::query!(
        //         r#"
        //         DELETE FROM address
        //         WHERE id = $1
        //         "#,
        //         address_id
        //     )
        //     .execute(&db_pool)
        //     .await
        //     {
        //         tracing::error!("Failed to delete address: {:?}", err);
        //     }
        // });

        Ok(address_id)
    } else {
        Err(Error::BadInput {
            field: "address".to_string(),
            message: "Invalid address".to_string(),
        })
    }
}

async fn get_races_by_address_id(
    db_pool: &sqlx::PgPool,
    election_id: &Uuid,
    address_id: &Uuid,
) -> Result<Vec<RaceResult>, Error> {
    // 1. Fetch base address info
    let user_address_data = sqlx::query!(
        r#"
        SELECT
            a.congressional_district,
            a.state_senate_district,
            a.state_house_district,
            a.state AS "state:State",
            a.postal_code,
            a.county,
            a.city
        FROM address AS a
        WHERE a.id = $1
        "#,
        address_id
    )
    .fetch_one(db_pool)
    .await?;

    // 2. Fetch extended state info if applicable
    let extended_address = if user_address_data.state == State::MN {
        Address::extended_mn_by_address_id(db_pool, address_id).await?
    } else {
        None
    };

    // 3. Normalize extended / fallback fields
    // For MN, use helper methods on AddressExtendedMN
    let (
        county_commissioner_district,
        judicial_district,
        parsed_soil_and_water_district,
        hospital_district,
        school_district,
        school_district_type,
        school_subdistrict,
        ward,
        city,
    ) = match &extended_address {
        Some(ext) if user_address_data.state == State::MN => (
            ext.county_commissioner_district_norm(),
            ext.judicial_district_norm(),
            ext.parsed_soil_and_water_district(),
            ext.hospital_district_norm(),
            ext.school_district_norm(),
            ext.school_district_type_norm(),
            ext.school_subdistrict_norm(),
            ext.ward_norm(),
            ext.city_norm(&user_address_data.city),
        ),
        _ => (
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            user_address_data.city.clone(),
        ),
    };

    // 4. Build query
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT
            r.id,
            r.slug,
            r.title,
            r.office_id,
            r.race_type,
            r.vote_type,
            r.party_id,
            r.state,
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
        FROM race r
        JOIN office o ON office_id = o.id
        WHERE r.election_id =
        "#,
    );

    builder.push_bind(election_id);

    // Always include national + state scope
    builder.push(" AND (o.election_scope = 'national'");
    builder.push(" OR (o.state = ");
    builder.push_bind(user_address_data.state);
    builder.push(" AND o.election_scope = 'state')");

    // Add Minnesota‑specific filters
    if user_address_data.state == State::MN {
        apply_mn_filters(
            &mut builder,
            user_address_data.state.clone(),
            user_address_data.county.as_deref(),
            city.clone(),
            user_address_data.congressional_district.clone(),
            user_address_data.state_senate_district.clone(),
            user_address_data.state_house_district.clone(),
            county_commissioner_district.clone(),
            judicial_district.clone(),
            school_district.clone(),
            school_district_type.clone(),
            school_subdistrict.clone(),
            ward.clone(),
            parsed_soil_and_water_district.clone(),
            hospital_district.clone(),
        );
    }

    builder.push(") ORDER BY o.priority ASC, title DESC");

    // 5. Run query
    let query = builder.build_query_as::<Race>();
    let records = query.fetch_all(db_pool).await?;

    Ok(records.into_iter().map(RaceResult::from).collect())
}

#[ComplexObject]
impl ElectionResult {
    async fn races(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
        filter: Option<ElectionRaceFilter>,
    ) -> ConnectionResult<RaceResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let filter = filter.unwrap_or_default();
        let default_page_size = 10;

        // Decode cursors
        let after_offset = after
            .as_ref()
            .and_then(|a| Base64Cursor::decode_cursor(a).ok())
            .map(|c| usize::from(c) + 1)
            .unwrap_or(0);

        let before_offset = before
            .as_ref()
            .and_then(|b| Base64Cursor::decode_cursor(b).ok())
            .map(|c| usize::from(c));

        // Determine range
        let limit = first.or(last).unwrap_or(default_page_size);
        let mut offset = after_offset;

        if let Some(before_idx) = before_offset {
            if let Some(last_n) = last {
                if before_idx > last_n as usize {
                    offset = before_idx - last_n as usize;
                } else {
                    offset = 0;
                }
            }
        }

        // Prepare normalized filters
        let state = filter.state;
        let query_like = filter
            .query
            .as_ref()
            .map(|q| format!("%{}%", q.trim().to_lowercase()));

        // --- 1️⃣ Count total ------------------------------------------
        let total_count: i64 = sqlx::query_scalar!(
            r#"
        SELECT COUNT(*)
        FROM race
        WHERE election_id = $1
          AND ($2::state IS NULL OR state = $2)
          AND ($3::TEXT IS NULL OR LOWER(title) LIKE $3)
        "#,
            uuid::Uuid::parse_str(&self.id)?,
            state as Option<State>,
            query_like,
        )
        .fetch_one(&db_pool)
        .await?
        .unwrap_or(0);

        // --- 2️⃣ Fetch paginated slice --------------------------------
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
        FROM race r
        JOIN office o ON o.id = r.office_id
        WHERE r.election_id = $1
          AND ($2::state IS NULL OR r.state = $2)
          AND ($3::TEXT IS NULL OR LOWER(r.title) LIKE $3)
        ORDER BY o.priority ASC NULLS LAST, r.title DESC, r.id ASC
        LIMIT $4 OFFSET $5
        "#,
            uuid::Uuid::parse_str(&self.id)?,
            state as Option<State>,
            query_like,
            limit as i64 + 1,
            offset as i64
        )
        .fetch_all(&db_pool)
        .await?;

        // --- 3️⃣ Pagination metadata -----------------------------------
        let has_next_page = records.len() as i64 > limit as i64;
        let has_previous_page = offset > 0;
        let slice = records.into_iter().take(limit as usize);

        // --- 4️⃣ Build Relay connection -------------------------------
        let mut connection = Connection::with_additional_fields(
            has_previous_page,
            has_next_page,
            ConnectionFields {
                total_count: total_count as usize,
            },
        );

        connection.edges.extend(
            slice.enumerate().map(|(idx, race)| {
                Edge::new(Base64Cursor::new(offset + idx), RaceResult::from(race))
            }),
        );

        Ok(connection)
    }

    /// Show races based on an anonymous user with an address
    async fn races_by_address(
        &self,
        ctx: &Context<'_>,
        address: AddressInput,
    ) -> Result<Vec<RaceResult>> {
        let election_id = uuid::Uuid::parse_str(&self.id)?;
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let address_id = process_address_with_geocodio(&db_pool, address).await?;
        let races = get_races_by_address_id(&db_pool, &election_id, &address_id).await?;
        Ok(races)
    }

    /// Show races relevant to the user based on their address
    async fn races_by_user_districts(&self, ctx: &Context<'_>) -> Result<Vec<RaceResult>, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let token = ctx.data::<Option<TokenData<AccessTokenClaims>>>();

        if let Some(token_data) = token.unwrap() {
            let election_id = uuid::Uuid::parse_str(&self.id)?;
            let address_id = Address::find_by_user_id(&db_pool, &token_data.claims.sub)
                .await?
                .map(|a| a.id);
            if let Some(address_id) = address_id {
                println!("address_id = {:?}", address_id);
                let results = get_races_by_address_id(&db_pool, &election_id, &address_id).await?;

                Ok(results)
            } else {
                tracing::debug!("No address found with user address data");
                Err(Error::UserAddressNotFound)
            }
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

    async fn ballot_measures_by_address(
        &self,
        ctx: &Context<'_>,
        address: AddressInput,
    ) -> Result<Vec<BallotMeasureResult>> {
        let election_id = uuid::Uuid::parse_str(&self.id)?;
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let address_id = process_address_with_geocodio(&db_pool, address).await?;
        let user_address_data = sqlx::query!(
            r#"
            SELECT
                a.congressional_district,
                a.state_senate_district,
                a.state_house_district,
                a.state AS "state:State",
                a.postal_code,
                a.county,
                a.city
            FROM
                address AS a
            WHERE
                
                a.id = $1
            "#,
            address_id
        )
        .fetch_one(&db_pool)
        .await?;

        let user_address_extended_mn =
            Address::extended_mn_by_address_id(&db_pool, &address_id).await?;

        let county_fips = user_address_extended_mn
            .clone()
            .and_then(|a| a.county_fips.clone());

        let municipality_fips = user_address_extended_mn
            .clone()
            .map(|a| {
                a.municipality_fips
                    .map(|d| d.as_str().trim_start_matches('0').to_string())
            })
            .unwrap_or(None);

        let school_district = user_address_extended_mn
            .clone()
            .map(|a| {
                a.school_district_number
                    .map(|d| d.as_str().trim_start_matches('0').to_string())
            })
            .unwrap_or(None);

        println!("county_fips = {:?}", county_fips);
        println!("municipality_fips = {:?}", municipality_fips);
        println!("school_district = {:?}", school_district);

        // Only handling statewide ballot measures for now
        let records = sqlx::query_as!(
            BallotMeasure,
            r#"
            SELECT
                bm.id,
                bm.slug,
                bm.title,
                bm.description,
                bm.status AS "status:BallotMeasureStatus",
                bm.ballot_measure_code,
                bm.measure_type,
                bm.definitions,
                bm.official_summary,
                bm.populist_summary,
                bm.full_text_url,
                bm.election_id,
                bm.state AS "state:State",
                bm.county,
                bm.municipality, 
                bm.school_district,
                bm.county_fips,
                bm.municipality_fips,
                bm.yes_votes,
                bm.no_votes,
                bm.num_precincts_reporting,
                bm.total_precincts,
                bm.election_scope AS "election_scope:ElectionScope",
                bm.created_at,
                bm.updated_at
            FROM
                ballot_measure bm
            WHERE
                bm.election_id = $1
                AND bm.state = $2::state
                AND (bm.county_fips IS NULL OR bm.county_fips = $3)
                AND (bm.municipality_fips IS NULL OR bm.municipality_fips = $4)
                AND (bm.school_district IS NULL OR bm.school_district = $5)
            "#,
            &election_id,
            user_address_data.state as State,
            county_fips,
            municipality_fips,
            school_district
        )
        .fetch_all(&db_pool)
        .await?;
        Ok(records.into_iter().map(BallotMeasureResult::from).collect())
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
