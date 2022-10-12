use super::{AddressResult, AddressExtendedMNResult};
use crate::{context::ApiContext, guard::UserGuard, is_admin};
use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject, ID};
use db::{models::enums::State, Address, AddressExtendedMN, UserWithProfile};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct UserResult {
    pub id: ID,
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile_picture_url: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateUserProfileInput {
    pub email: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[ComplexObject]
impl UserResult {
    #[graphql(guard = "UserGuard::new(&self.id)", visible = "is_admin")]
    async fn address(&self, ctx: &Context<'_>) -> Result<Option<AddressResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(Address,
            r#"
            SELECT a.id, a.line_1, a.line_2, a.city, a.county, a.state AS "state:State", a.postal_code, a.country, a.congressional_district, a.state_senate_district, a.state_house_district FROM address AS a
            JOIN user_profile up ON user_id = $1
            JOIN address ON up.address_id = a.id
        "#,
            uuid::Uuid::parse_str(&self.id)?,
        )
        .fetch_optional(&db_pool)
        .await.unwrap();

        match record {
            Some(address) => Ok(Some(address.into())),
            None => Ok(None),
        }
    }

    #[graphql(guard = "UserGuard::new(&self.id)", visible = "is_admin")]
    async fn address_extended_mn(&self, ctx: &Context<'_>) -> Result<Option<AddressExtendedMNResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record: std::option::Option<AddressExtendedMN>;

        // First order of business is to geolocate the user's school district. This
        // is needed to join on the school district boundaries map.
        let users_school_district = sqlx::query!(
            r#"
            SELECT gid, sdnumber
            FROM p6t_state_mn.school_district_boundaries AS sd
            JOIN user_profile up ON up.user_id = $1
            JOIN address a ON up.address_id = a.id
            WHERE ST_Contains(ST_SetSRID(sd.geom, 26915), ST_Transform(a.geom, 26915))
            "#,
            uuid::Uuid::parse_str(&self.id)?,
        )
        .fetch_optional(&db_pool)
        .await?;

        match users_school_district {
            Some(rec) => {
                // User lives in a Minnesota school district. They should also live in a voting district too.
                // tracing::info!("sdnumber: {:?}",
                //     rec.sdnumber
                // );
                let found_sdnumber: String;
                match rec.sdnumber {
                    Some(sdnumber) => {
                        found_sdnumber = sdnumber;
                    },
                    None => {
                        found_sdnumber = "".to_string();
                    },
                }

                if found_sdnumber == "2180" {
                    // Special case for ISD 2180 (MACCRAY). Consult the shapefile and not the crosswalk table.
                    record = sqlx::query_as!(AddressExtendedMN,
                        r#"
                        SELECT vd.gid,
                            vd.vtdid AS voting_tabulation_district_id,
                            vd.countycode AS county_code,
                            vd.countyname AS county_name,
                            vd.pctcode AS precinct_code,
                            vd.pctname AS precinct_name,
                            vd.ctycomdist AS county_commissioner_district,
                            vd.juddist AS judicial_district,
                            sd.sdnumber AS school_district_number,
                            sd.shortname AS school_district_name,
                            isd.id::varchar(4) AS school_subdistrict_code,
                            isd.schsubdist AS school_subdistrict_name
                        FROM p6t_state_mn.bdry_votingdistricts AS vd
                        JOIN user_profile up ON up.user_id = $1
                        JOIN address a ON up.address_id = a.id
                        JOIN p6t_state_mn.school_district_boundaries sd ON sd.gid = $2
                        LEFT JOIN (
                            SELECT isd2.id as id, schsubdist, sdnumber FROM p6t_state_mn.isd2180 as isd2
                            JOIN user_profile up2 ON up2.user_id = $1
                            JOIN address a2 ON up2.address_id = a2.id
                            WHERE ST_Contains(ST_SetSRID(isd2.geom, 26915), ST_Transform(a2.geom, 26915))
                            ) AS isd ON isd.sdnumber = sd.sdnumber
                        WHERE ST_Contains(ST_SetSRID(vd.geom, 26915), ST_Transform(a.geom, 26915))
                    "#,
                        uuid::Uuid::parse_str(&self.id)?,
                        rec.gid,
                    )
                    .fetch_optional(&db_pool)
                    .await.unwrap();
                } else if found_sdnumber == "2853" {
                    // Special case for ISD 2853 (Lac Qui Parle Valley). Consult the shapefile and not the crosswalk table.
                    record = sqlx::query_as!(AddressExtendedMN,
                        r#"
                         SELECT vd.gid,
                            vd.vtdid AS voting_tabulation_district_id,
                            vd.countycode AS county_code,
                            vd.countyname AS county_name,
                            vd.pctcode AS precinct_code,
                            vd.pctname AS precinct_name,
                            vd.ctycomdist AS county_commissioner_district,
                            vd.juddist AS judicial_district,
                            sd.sdnumber AS school_district_number,
                            sd.shortname AS school_district_name,
                            SUBSTRING(isd.schsubdist, 10) AS school_subdistrict_code,
                            isd.schsubdist AS school_subdistrict_name
                        FROM p6t_state_mn.bdry_votingdistricts AS vd
                        JOIN user_profile up ON up.user_id = $1
                        JOIN address a ON up.address_id = a.id
                        JOIN p6t_state_mn.school_district_boundaries sd ON sd.gid = $2
                        LEFT JOIN (
                            SELECT * FROM p6t_state_mn.isd2853 as isd2
                            JOIN user_profile up2 ON up2.user_id = $1
                            JOIN address a2 ON up2.address_id = a2.id
                            WHERE ST_Contains(ST_SetSRID(isd2.geom, 26915), ST_Transform(a2.geom, 26915))
                            ) AS isd ON isd.countycode = vd.countycode AND isd.pctcode = vd.pctcode
                        WHERE ST_Contains(ST_SetSRID(vd.geom, 26915), ST_Transform(a.geom, 26915))
                    "#,
                        uuid::Uuid::parse_str(&self.id)?,
                        rec.gid,
                    )
                    .fetch_optional(&db_pool)
                    .await.unwrap();
                } else {
                    // General case to use the cross walk table for subdistricts. Note that the cross walk table is only
                    // for school districts that have subdistricts.
                    record = sqlx::query_as!(AddressExtendedMN,
                        r#"
                        SELECT vd.gid,
                            vd.vtdid AS voting_tabulation_district_id,
                            vd.countycode AS county_code,
                            vd.countyname AS county_name,
                            vd.pctcode AS precinct_code,
                            vd.pctname AS precinct_name,
                            vd.ctycomdist AS county_commissioner_district,
                            vd.juddist AS judicial_district,
                            sd.sdnumber AS school_district_number,
                            sd.shortname AS school_district_name,
                            cw.school_subdistrict_code,
                            INITCAP(cw.school_subdistrict_name) as school_subdistrict_name
                        FROM p6t_state_mn.bdry_votingdistricts AS vd
                        JOIN user_profile up ON up.user_id = $1
                        JOIN address a ON up.address_id = a.id
                        JOIN p6t_state_mn.school_district_boundaries sd ON sd.gid = $2
                        LEFT JOIN p6t_state_mn.precinct_school_subdistrict_crosswalk AS cw ON cw.county_id = vd.countycode AND cw.precinct_code = vd.pctcode
                        WHERE ST_Contains(ST_SetSRID(vd.geom, 26915), ST_Transform(a.geom, 26915))
                    "#,
                        uuid::Uuid::parse_str(&self.id)?,
                        rec.gid,
                    )
                    .fetch_optional(&db_pool)
                    .await.unwrap();
                }
            },
            None => {
                // User does not live in a school district. This could happen if address is
                // outside Minnesota, or (highly unlikely) the school boundary shapefile doesn't match
                // the boundaries of the Minnesota voting districts.
                record = sqlx::query_as!(AddressExtendedMN,
                    r#"
                    SELECT vd.gid,
                    vtdid AS voting_tabulation_district_id,
                    countycode AS county_code, countyname AS county_name,
                    pctcode AS precinct_code, pctname AS precinct_name,
                    ctycomdist AS county_commissioner_district,
                    juddist AS judicial_district,
                    cw.school_district_number,
                    cw.school_district_name,
                    cw.school_subdistrict_code,
                    cw.school_subdistrict_name
                    FROM p6t_state_mn.bdry_votingdistricts AS vd
                    JOIN user_profile up ON up.user_id = $1
                    JOIN address a ON up.address_id = a.id
                    LEFT JOIN p6t_state_mn.precinct_school_subdistrict_crosswalk AS cw ON cw.county_id = vd.countycode AND cw.precinct_code = vd.pctcode
                    WHERE ST_Contains(ST_SetSRID(vd.geom, 26915), ST_Transform(a.geom, 26915))
                "#,
                    uuid::Uuid::parse_str(&self.id)?,
                )
                .fetch_optional(&db_pool)
                .await.unwrap();
            },
        };

        match record {
            Some(address_extended_mn) => Ok(Some(address_extended_mn.into())),
            None => Ok(None),
        }
    }
}

impl From<UserWithProfile> for UserResult {
    fn from(user: UserWithProfile) -> Self {
        Self {
            id: user.id.into(),
            username: user.username,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            profile_picture_url: user.profile_picture_url,
        }
    }
}
