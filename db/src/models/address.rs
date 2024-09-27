use async_graphql::InputObject;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use super::enums::State;

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

#[derive(FromRow, Debug, Clone)]
pub struct AddressExtendedMN {
    pub gid: i32,
    pub voting_tabulation_district_id: Option<String>,
    pub county_code: Option<String>,
    pub county_name: Option<String>,
    pub precinct_code: Option<String>,
    pub precinct_name: Option<String>,
    pub municipality_name: Option<String>,
    pub county_commissioner_district: Option<String>,
    pub judicial_district: Option<String>,
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
                            vd.pctcode AS precinct_code,
                            vd.pctname AS precinct_name,
                            vd.mcdname AS municipality_name,
                            vd.ctycomdist AS county_commissioner_district,
                            vd.juddist AS judicial_district,
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
                            vd.pctcode AS precinct_code,
                            vd.pctname AS precinct_name,
                            vd.mcdname AS municipality_name,
                            vd.ctycomdist AS county_commissioner_district,
                            vd.juddist AS judicial_district,
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
                            vd.pctcode AS precinct_code,
                            vd.pctname AS precinct_name,
                            vd.mcdname AS municipality_name,
                            vd.ctycomdist AS county_commissioner_district,
                            vd.juddist AS judicial_district,
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
                    countycode AS county_code, countyname AS county_name,
                    pctcode AS precinct_code, pctname AS precinct_name,
                    ctycomdist AS county_commissioner_district,
                    juddist AS judicial_district,
                    vd.mcdname AS municipality_name,
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
