use super::enums::State;
use async_graphql::InputObject;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct Address {
    pub id: uuid::Uuid,
    pub line_1: String,
    pub line_2: Option<String>,
    pub city: String,
    pub state: State,
    pub county: Option<String>,
    pub country: String,
    pub postal_code: String,
    pub congressional_district: Option<String>,
    pub state_senate_district: Option<String>,
    pub state_house_district: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, InputObject, Debug)]
pub struct AddressInput {
    pub line_1: String,
    pub line_2: Option<String>,
    pub city: String,
    pub county: Option<String>,
    pub state: State,
    pub country: String,
    pub postal_code: String,
    pub coordinates: Option<Coordinates>,
    pub congressional_district: Option<String>,
    pub state_senate_district: Option<String>,
    pub state_house_district: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, InputObject, Debug)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

/// Input for creating a new address. Used by scrapers and other code that does not geocode.
#[derive(Debug, Clone)]
pub struct InsertAddressInput {
    pub line_1: String,
    pub line_2: Option<String>,
    pub city: String,
    pub state: State,
    pub country: String,
    pub postal_code: String,
    pub county: Option<String>,
    pub congressional_district: Option<String>,
    pub state_senate_district: Option<String>,
    pub state_house_district: Option<String>,
    pub lon: Option<f64>,
    pub lat: Option<f64>,
}

/// Input for updating an existing address. All fields optional except id.
#[derive(Debug, Clone)]
pub struct UpdateAddressInput {
    pub id: uuid::Uuid,
    pub line_1: Option<String>,
    pub line_2: Option<String>,
    pub city: Option<String>,
    pub state: Option<State>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub county: Option<String>,
    pub congressional_district: Option<String>,
    pub state_senate_district: Option<String>,
    pub state_house_district: Option<String>,
    pub lon: Option<f64>,
    pub lat: Option<f64>,
}

/// Parameters for searching addresses. All fields optional; combined with AND.
#[derive(Debug, Clone, Default)]
pub struct SearchAddressParams {
    pub line_1_contains: Option<String>,
    pub city: Option<String>,
    pub state: Option<State>,
    pub country: Option<String>,
    pub postal_code_prefix: Option<String>,
    pub limit: Option<u32>,
}

#[derive(FromRow, Debug, Clone)]
pub struct AddressExtendedMN {
    pub gid: i32,
    pub voting_tabulation_district_id: Option<String>,
    pub county_code: Option<String>,
    pub county_name: Option<String>,
    pub county_fips: Option<String>,
    pub municipality_fips: Option<String>,
    pub precinct_code: Option<String>,
    pub precinct_name: Option<String>,
    pub municipality_name: Option<String>,
    pub county_commissioner_district: Option<String>,
    pub hospital_district: Option<String>,
    pub judicial_district: Option<String>,
    pub soil_and_water_district: Option<String>,
    pub school_district_number: Option<String>,
    pub school_district_name: Option<String>,
    pub school_subdistrict_code: Option<String>,
    pub school_subdistrict_name: Option<String>,
    pub school_district_type: Option<String>,
    pub ward: Option<String>,
}

pub enum SchoolDistrictTypeMN {
    /// Independent Districts and Schools
    ISD = 1,
    /// Includes only 2 Common Districts, Franconia-0323 and Prinsburg-0815
    Common = 2,
    /// Includes only 2 Special Districts, Minneapolis-0001 and South St. Paul-0006
    SSD = 3,
}

impl Address {
    pub async fn find_by_user_id(
        pool: &PgPool,
        user_id: &uuid::Uuid,
    ) -> Result<Option<Address>, sqlx::Error> {
        let address = sqlx::query_as!(
            Address,
            r#"
        SELECT
            a.id,
            a.line_1,
            a.line_2,
            a.city,
            a.county,
            a.state AS "state:State",
            a.postal_code,
            a.country,
            a.congressional_district,
            a.state_senate_district,
            a.state_house_district
        FROM
            address AS a
            JOIN user_profile up ON user_id = $1
            JOIN address ON up.address_id = a.id
        "#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(address)
    }

    /// Fetch an address by id.
    pub async fn find_by_id(
        pool: &PgPool,
        id: &uuid::Uuid,
    ) -> Result<Option<Address>, sqlx::Error> {
        let address = sqlx::query_as!(
            Address,
            r#"
            SELECT
                id,
                line_1,
                line_2,
                city,
                county,
                state AS "state:State",
                postal_code,
                country,
                congressional_district,
                state_senate_district,
                state_house_district
            FROM address
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(address)
    }

    /// Find an address by the unique key (line_1, line_2, city, state, country, postal_code).
    pub async fn find_by_unique_key(
        pool: &PgPool,
        line_1: &str,
        line_2: Option<&str>,
        city: &str,
        state: &State,
        country: &str,
        postal_code: &str,
    ) -> Result<Option<Address>, sqlx::Error> {
        let address = sqlx::query_as!(
            Address,
            r#"
            SELECT
                id,
                line_1,
                line_2,
                city,
                county,
                state AS "state:State",
                postal_code,
                country,
                congressional_district,
                state_senate_district,
                state_house_district
            FROM address
            WHERE line_1 = $1
              AND line_2 IS NOT DISTINCT FROM $2
              AND city = $3
              AND state = $4
              AND country = $5
              AND postal_code = $6
            "#,
            line_1,
            line_2,
            city,
            state.to_string(),
            country,
            postal_code
        )
        .fetch_optional(pool)
        .await?;
        Ok(address)
    }

    /// Upsert by (line_1, line_2, city, state, country, postal_code). Inserts if new; if conflict,
    /// updates lon/lat and district fields and returns the existing row.
    pub async fn upsert(
        pool: &PgPool,
        input: &InsertAddressInput,
    ) -> Result<Address, sqlx::Error> {
        let (lon, lat) = (input.lon, input.lat);
        let has_coords = lon.is_some() && lat.is_some();
        let row = if has_coords {
            let lon = lon.unwrap();
            let lat = lat.unwrap();
            let wkt = format!("POINT({} {})", lon, lat);
            sqlx::query_as!(
                Address,
                r#"
                INSERT INTO address (line_1, line_2, city, state, county, country, postal_code,
                    lon, lat, geog, geom, congressional_district, state_senate_district, state_house_district)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    ST_SetSRID(ST_MakePoint($8, $9), 4326), ST_GeomFromText($10, 4326),
                    $11, $12, $13)
                ON CONFLICT (line_1, line_2, city, state, country, postal_code)
                DO UPDATE SET
                    lon = EXCLUDED.lon,
                    lat = EXCLUDED.lat,
                    geog = EXCLUDED.geog,
                    geom = EXCLUDED.geom,
                    county = COALESCE(EXCLUDED.county, address.county),
                    congressional_district = COALESCE(EXCLUDED.congressional_district, address.congressional_district),
                    state_senate_district = COALESCE(EXCLUDED.state_senate_district, address.state_senate_district),
                    state_house_district = COALESCE(EXCLUDED.state_house_district, address.state_house_district)
                RETURNING
                    id,
                    line_1,
                    line_2,
                    city,
                    county,
                    state AS "state:State",
                    postal_code,
                    country,
                    congressional_district,
                    state_senate_district,
                    state_house_district
                "#,
                input.line_1,
                input.line_2,
                input.city,
                input.state.to_string(),
                input.county,
                input.country,
                input.postal_code,
                lon,
                lat,
                wkt,
                input.congressional_district,
                input.state_senate_district,
                input.state_house_district
            )
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_as!(
                Address,
                r#"
                INSERT INTO address (line_1, line_2, city, state, county, country, postal_code,
                    congressional_district, state_senate_district, state_house_district)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (line_1, line_2, city, state, country, postal_code)
                DO UPDATE SET
                    county = COALESCE(EXCLUDED.county, address.county),
                    congressional_district = COALESCE(EXCLUDED.congressional_district, address.congressional_district),
                    state_senate_district = COALESCE(EXCLUDED.state_senate_district, address.state_senate_district),
                    state_house_district = COALESCE(EXCLUDED.state_house_district, address.state_house_district)
                RETURNING
                    id,
                    line_1,
                    line_2,
                    city,
                    county,
                    state AS "state:State",
                    postal_code,
                    country,
                    congressional_district,
                    state_senate_district,
                    state_house_district
                "#,
                input.line_1,
                input.line_2,
                input.city,
                input.state.to_string(),
                input.county,
                input.country,
                input.postal_code,
                input.congressional_district,
                input.state_senate_district,
                input.state_house_district
            )
            .fetch_one(pool)
            .await?
        };
        Ok(row)
    }

    /// Find an address by unique key, or insert it if not found.
    pub async fn find_or_create(
        pool: &PgPool,
        input: &InsertAddressInput,
    ) -> Result<Address, sqlx::Error> {
        if let Some(addr) = Self::find_by_unique_key(
            pool,
            &input.line_1,
            input.line_2.as_deref(),
            &input.city,
            &input.state,
            &input.country,
            &input.postal_code,
        )
        .await?
        {
            return Ok(addr);
        }
        Self::upsert(pool, input).await
    }

    /// Update an existing address by id. Only provided fields are updated.
    pub async fn update(
        pool: &PgPool,
        input: &UpdateAddressInput,
    ) -> Result<Address, sqlx::Error> {
        let has_coords = input.lon.is_some() && input.lat.is_some();
        let row = if has_coords {
            let lon = input.lon.unwrap();
            let lat = input.lat.unwrap();
            let wkt = format!("POINT({} {})", lon, lat);
            sqlx::query_as!(
                Address,
                r#"
                UPDATE address
                SET
                    line_1 = COALESCE($2, line_1),
                    line_2 = COALESCE($3, line_2),
                    city = COALESCE($4, city),
                    state = COALESCE($5, state),
                    county = COALESCE($6, county),
                    country = COALESCE($7, country),
                    postal_code = COALESCE($8, postal_code),
                    lon = COALESCE($9, lon),
                    lat = COALESCE($10, lat),
                    geog = COALESCE(ST_SetSRID(ST_MakePoint($9, $10), 4326), geog),
                    geom = COALESCE(ST_GeomFromText($11, 4326), geom),
                    congressional_district = COALESCE($12, congressional_district),
                    state_senate_district = COALESCE($13, state_senate_district),
                    state_house_district = COALESCE($14, state_house_district)
                WHERE id = $1
                RETURNING
                    id,
                    line_1,
                    line_2,
                    city,
                    county,
                    state AS "state:State",
                    postal_code,
                    country,
                    congressional_district,
                    state_senate_district,
                    state_house_district
                "#,
                input.id,
                input.line_1,
                input.line_2,
                input.city,
                input.state.as_ref().map(|s| s.to_string()),
                input.county,
                input.country,
                input.postal_code,
                lon,
                lat,
                wkt,
                input.congressional_district,
                input.state_senate_district,
                input.state_house_district
            )
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_as!(
                Address,
                r#"
                UPDATE address
                SET
                    line_1 = COALESCE($2, line_1),
                    line_2 = COALESCE($3, line_2),
                    city = COALESCE($4, city),
                    state = COALESCE($5, state),
                    county = COALESCE($6, county),
                    country = COALESCE($7, country),
                    postal_code = COALESCE($8, postal_code),
                    congressional_district = COALESCE($9, congressional_district),
                    state_senate_district = COALESCE($10, state_senate_district),
                    state_house_district = COALESCE($11, state_house_district)
                WHERE id = $1
                RETURNING
                    id,
                    line_1,
                    line_2,
                    city,
                    county,
                    state AS "state:State",
                    postal_code,
                    country,
                    congressional_district,
                    state_senate_district,
                    state_house_district
                "#,
                input.id,
                input.line_1,
                input.line_2,
                input.city,
                input.state.as_ref().map(|s| s.to_string()),
                input.county,
                input.country,
                input.postal_code,
                input.congressional_district,
                input.state_senate_district,
                input.state_house_district
            )
            .fetch_one(pool)
            .await?
        };
        Ok(row)
    }

    /// Search addresses by optional filters. Conditions are ANDed. Limit defaults to 100.
    pub async fn search(
        pool: &PgPool,
        params: &SearchAddressParams,
    ) -> Result<Vec<Address>, sqlx::Error> {
        let limit = params.limit.unwrap_or(100).min(500) as i64;
        let line_1_contains = params
            .line_1_contains
            .as_deref()
            .map(|s| format!("%{}%", s));
        let postal_code_prefix = params.postal_code_prefix.as_deref();

        let rows = sqlx::query_as!(
            Address,
            r#"
            SELECT
                id,
                line_1,
                line_2,
                city,
                county,
                state AS "state:State",
                postal_code,
                country,
                congressional_district,
                state_senate_district,
                state_house_district
            FROM address
            WHERE
                ($1::text IS NULL OR line_1 ILIKE $1)
                AND ($2::text IS NULL OR city = $2)
                AND ($3::text IS NULL OR state = $3)
                AND ($4::text IS NULL OR country = $4)
                AND ($5::text IS NULL OR postal_code LIKE $5 || '%')
            ORDER BY city, line_1
            LIMIT $6
            "#,
            line_1_contains,
            params.city.as_deref(),
            params.state.as_ref().map(|s| s.to_string()),
            params.country.as_deref(),
            postal_code_prefix,
            limit
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn extended_mn_by_address_id(
        pool: &PgPool,
        address_id: &uuid::Uuid,
    ) -> Result<Option<AddressExtendedMN>, sqlx::Error> {
        let address_school_district = sqlx::query!(
            r#"
            SELECT gid, sdnumber, sdtype
            FROM p6t_state_mn.school_district_boundaries AS sd
            JOIN address a ON a.id = $1
            WHERE ST_Contains(ST_SetSRID(sd.geom, 26915), ST_Transform(a.geom, 26915))
            "#,
            address_id,
        )
        .fetch_optional(pool)
        .await?;

        let record = match address_school_district {
            Some(rec) => {
                let found_sdnumber = rec.sdnumber.unwrap_or_else(|| "".to_string());

                if found_sdnumber == "2180" {
                    // Special case for ISD 2180 (MACCRAY). Consult the shapefile and not the crosswalk table.
                    sqlx::query_as!(AddressExtendedMN,
                        r#"
                        SELECT vd.gid,
                            vd.vtdid AS voting_tabulation_district_id,
                            vd.countycode AS county_code,
                            vd.countyname AS county_name,
                            vd.countyfips AS county_fips,
                            vd.mcdfips AS municipality_fips,
                            vd.pctcode AS precinct_code,
                            vd.pctname AS precinct_name,
                            vd.mcdname AS municipality_name,
                            vd.ctycomdist AS county_commissioner_district,
                            vd.juddist AS judicial_district,
                            vd.swcdist_n AS soil_and_water_district,
                            vd.hospdist_n AS hospital_district,
                            vd.ward,
                            sd.sdnumber AS school_district_number,
                            sd.shortname AS school_district_name,
                            sd.sdtype AS school_district_type,
                            isd.id::varchar(4) AS school_subdistrict_code,
                            isd.schsubdist AS school_subdistrict_name
                        FROM p6t_state_mn.bdry_votingdistricts AS vd
                        JOIN address a ON a.id = $1
                        JOIN p6t_state_mn.school_district_boundaries sd ON sd.gid = $2
                        LEFT JOIN (
                            SELECT isd2.id as id, schsubdist, sdnumber FROM p6t_state_mn.isd2180 as isd2
                            JOIN address a2 ON a2.id = $1
                            WHERE ST_Contains(ST_SetSRID(isd2.geom, 26915), ST_Transform(a2.geom, 26915))
                            ) AS isd ON isd.sdnumber = sd.sdnumber
                        WHERE ST_Contains(ST_SetSRID(vd.geom, 26915), ST_Transform(a.geom, 26915))
                        "#,
                        address_id,
                        rec.gid,
                    )
                    .fetch_optional(pool)
                    .await?
                } else if found_sdnumber == "2853" {
                    // Special case for ISD 2853 (Lac Qui Parle Valley). Consult the shapefile and not the crosswalk table.
                    sqlx::query_as!(AddressExtendedMN,
                        r#"
                        SELECT vd.gid,
                            vd.vtdid AS voting_tabulation_district_id,
                            vd.countycode AS county_code,
                            vd.countyname AS county_name,
                            vd.countyfips AS county_fips,
                            vd.mcdfips AS municipality_fips,
                            vd.pctcode AS precinct_code,
                            vd.pctname AS precinct_name,
                            vd.mcdname AS municipality_name,
                            vd.ctycomdist AS county_commissioner_district,
                            vd.juddist AS judicial_district,
                            vd.swcdist_n AS soil_and_water_district,
                            vd.hospdist_n AS hospital_district,
                            vd.ward,
                            sd.sdnumber AS school_district_number,
                            sd.shortname AS school_district_name,
                            sd.sdtype AS school_district_type,
                            SUBSTRING(isd.schsubdist, 10) AS school_subdistrict_code,
                            isd.schsubdist AS school_subdistrict_name
                        FROM p6t_state_mn.bdry_votingdistricts AS vd
                        JOIN address a ON a.id = $1
                        JOIN p6t_state_mn.school_district_boundaries sd ON sd.gid = $2
                        LEFT JOIN (
                            SELECT * FROM p6t_state_mn.isd2853 as isd2
                            JOIN address a2 ON a2.id = $1
                            WHERE ST_Contains(ST_SetSRID(isd2.geom, 26915), ST_Transform(a2.geom, 26915))
                            ) AS isd ON isd.countycode = vd.countycode AND isd.pctcode = vd.pctcode
                        WHERE ST_Contains(ST_SetSRID(vd.geom, 26915), ST_Transform(a.geom, 26915))
                        "#,
                        address_id,
                        rec.gid,
                    )
                    .fetch_optional(pool)
                    .await?
                } else {
                    // General case to use the crosswalk table for subdistricts.
                    sqlx::query_as!(AddressExtendedMN,
                        r#"
                        SELECT vd.gid,
                            vd.vtdid AS voting_tabulation_district_id,
                            vd.countycode AS county_code,
                            vd.countyname AS county_name,
                            vd.countyfips AS county_fips,
                            vd.mcdfips AS municipality_fips,
                            vd.pctcode AS precinct_code,
                            vd.pctname AS precinct_name,
                            vd.mcdname AS municipality_name,
                            vd.ctycomdist AS county_commissioner_district,
                            vd.juddist AS judicial_district,
                            vd.swcdist_n AS soil_and_water_district,
                            vd.hospdist_n AS hospital_district,
                            vd.ward,
                            sd.sdnumber AS school_district_number,
                            sd.shortname AS school_district_name,
                            sd.sdtype AS school_district_type,
                            cw.school_subdistrict_code,
                            INITCAP(cw.school_subdistrict_name) as school_subdistrict_name
                        FROM p6t_state_mn.bdry_votingdistricts AS vd
                        JOIN address a ON a.id = $1
                        JOIN p6t_state_mn.school_district_boundaries sd ON sd.gid = $2
                        LEFT JOIN p6t_state_mn.precinct_school_subdistrict_crosswalk AS cw ON cw.county_id = vd.countycode AND cw.precinct_code = vd.pctcode
                        WHERE ST_Contains(ST_SetSRID(vd.geom, 26915), ST_Transform(a.geom, 26915))
                        "#,
                        address_id,
                        rec.gid,
                    )
                    .fetch_optional(pool)
                    .await?
                }
            }
            None => {
                tracing::info!("No school district found for address {}", address_id);
                // Address does not live in a school district. This could happen if the address is outside Minnesota,
                // or (highly unlikely) the school boundary shapefile doesn't match the boundaries of the Minnesota voting districts.
                sqlx::query_as!(AddressExtendedMN,
                    r#"
                    SELECT vd.gid,
                    vtdid AS voting_tabulation_district_id,
                    countycode AS county_code, countyname AS county_name, countyfips AS county_fips,
                    mcdfips AS municipality_fips,
                    pctcode AS precinct_code, pctname AS precinct_name,
                    ctycomdist AS county_commissioner_district,
                    juddist AS judicial_district,
                    mcdname AS municipality_name,
                    swcdist_n AS soil_and_water_district,
                    hospdist_n AS hospital_district,
                    ward,
                    cw.school_district_number,
                    cw.school_district_name,
                    NULL as school_district_type,
                    cw.school_subdistrict_code,
                    cw.school_subdistrict_name
                    FROM p6t_state_mn.bdry_votingdistricts AS vd
                    JOIN address a ON a.id = $1
                    LEFT JOIN p6t_state_mn.precinct_school_subdistrict_crosswalk AS cw ON cw.county_id = vd.countycode AND cw.precinct_code = vd.pctcode
                    WHERE ST_Contains(ST_SetSRID(vd.geom, 26915), ST_Transform(a.geom, 26915))
                "#,
                    address_id,
                )
                .fetch_optional(pool)
                .await?
            }
        };

        Ok(record)
    }
}

impl AddressExtendedMN {
    pub fn county_commissioner_district_norm(&self) -> Option<String> {
        self.county_commissioner_district
            .as_ref()
            .map(|d| d.trim_start_matches('0').to_string())
    }

    pub fn judicial_district_norm(&self) -> Option<String> {
        self.judicial_district
            .as_ref()
            .map(|d| d.trim_start_matches('0').to_string())
    }

    pub fn parsed_soil_and_water_district(&self) -> Option<String> {
        extract_district_or_direction(
            self.soil_and_water_district
                .as_ref()
                .map(|d| d.trim_start_matches('0').to_string()),
        )
    }

    pub fn hospital_district_norm(&self) -> Option<String> {
        self.hospital_district.clone()
    }

    pub fn school_district_norm(&self) -> Option<String> {
        self.school_district_number
            .as_ref()
            .map(|d| d.trim_start_matches('0').to_string())
    }

    pub fn school_district_type_norm(&self) -> Option<String> {
        self.school_district_type.clone()
    }

    pub fn school_subdistrict_norm(&self) -> Option<String> {
        self.school_subdistrict_code
            .as_ref()
            .map(|d| d.trim_start_matches('0').to_string())
    }

    pub fn ward_norm(&self) -> Option<String> {
        self.ward.as_ref().map(|d| {
            if let Some(pos) = d.find('-') {
                d[(pos + 1)..].trim_start_matches('0').to_string()
            } else {
                d.trim_start_matches('0').to_string()
            }
        })
    }

    pub fn city_norm(&self, base_city: &str) -> String {
        self.municipality_name
            .as_ref()
            .map(|m| m.replace("Twp", "Township"))
            .unwrap_or_else(|| base_city.to_string())
    }
}

fn extract_district_or_direction(input: Option<String>) -> Option<String> {
    let re = regex::Regex::new(r"(District\s*(\d+)|(East|West|North|South))").unwrap();

    input.and_then(|d| {
        re.captures(&d).and_then(|cap| {
            if let Some(district) = cap.get(2) {
                // Extract district number, remove leading zeros
                Some(district.as_str().trim_start_matches('0').to_string())
            } else {
                cap.get(3).map(|direction| direction.as_str().to_string())
            }
        })
    })
}
