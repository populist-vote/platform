#[cfg(test)]
mod tests {
    use crate::tests::harness::TestHarness;
    use uuid::Uuid;

    async fn insert_office(harness: &TestHarness, slug: &str) -> anyhow::Result<Uuid> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO office (slug, title, political_scope, election_scope)
            VALUES ($1, $2, 'state', 'state')
            RETURNING id
            "#,
            slug,
            "Test Office"
        )
        .fetch_one(&harness.pool)
        .await?;
        Ok(id)
    }

    async fn insert_race(
        harness: &TestHarness,
        slug: &str,
        title: &str,
        office_id: Uuid,
    ) -> anyhow::Result<Uuid> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO race (slug, title, office_id, race_type)
            VALUES ($1, $2, $3, 'general')
            RETURNING id
            "#,
            slug,
            title,
            office_id
        )
        .fetch_one(&harness.pool)
        .await?;
        Ok(id)
    }

    async fn insert_politician(
        harness: &TestHarness,
        slug: &str,
        first_name: &str,
        last_name: &str,
        email: &str,
    ) -> anyhow::Result<Uuid> {
        let full_name = format!("{first_name} {last_name}");
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO politician (slug, first_name, last_name, full_name, home_state, email)
            VALUES ($1, $2, $3, $4, 'TX', $5)
            RETURNING id
            "#,
            slug,
            first_name,
            last_name,
            full_name,
            email
        )
        .fetch_one(&harness.pool)
        .await?;
        Ok(id)
    }

    async fn add_candidate(
        harness: &TestHarness,
        race_id: Uuid,
        politician_id: Uuid,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO race_candidates (race_id, candidate_id)
            VALUES ($1, $2)
            "#,
            race_id,
            politician_id
        )
        .execute(&harness.pool)
        .await?;
        Ok(())
    }

    fn parse_csv(csv_string: &str) -> anyhow::Result<(csv::StringRecord, Vec<csv::StringRecord>)> {
        let mut reader = csv::Reader::from_reader(csv_string.as_bytes());
        let headers = reader.headers()?.clone();
        let rows = reader.records().collect::<Result<Vec<_>, _>>()?;
        Ok((headers, rows))
    }

    #[tokio::test]
    async fn download_all_candidate_guide_data_includes_embed_columns() -> anyhow::Result<()> {
        let harness = TestHarness::from_existing().await?;
        let suffix = Uuid::new_v4().simple().to_string();
        let user_id = harness
            .create_user(&format!("guide-export-{suffix}@example.com"), None)
            .await?;
        let organization_id = harness
            .create_organization(&format!("Guide Export Org {suffix}"))
            .await?;

        let office_id = insert_office(&harness, &format!("guide-export-office-{suffix}")).await?;
        let race_one_id = insert_race(
            &harness,
            &format!("guide-export-race-1-{suffix}"),
            "Texas Governor General 2026",
            office_id,
        )
        .await?;
        let race_two_id = insert_race(
            &harness,
            &format!("guide-export-race-2-{suffix}"),
            "Texas Senate District 1 General 2026",
            office_id,
        )
        .await?;
        let race_without_embed_id = insert_race(
            &harness,
            &format!("guide-export-race-3-{suffix}"),
            "Texas House District 1 General 2026",
            office_id,
        )
        .await?;

        let alice_id = insert_politician(
            &harness,
            &format!("alice-guide-{suffix}"),
            "Alice",
            "Adams",
            &format!("alice-{suffix}@example.com"),
        )
        .await?;
        let bob_id = insert_politician(
            &harness,
            &format!("bob-guide-{suffix}"),
            "Bob",
            "Baker",
            &format!("bob-{suffix}@example.com"),
        )
        .await?;
        let cara_id = insert_politician(
            &harness,
            &format!("cara-guide-{suffix}"),
            "Cara",
            "Clark",
            &format!("cara-{suffix}@example.com"),
        )
        .await?;

        add_candidate(&harness, race_one_id, alice_id).await?;
        add_candidate(&harness, race_one_id, bob_id).await?;
        add_candidate(&harness, race_two_id, cara_id).await?;
        add_candidate(&harness, race_without_embed_id, alice_id).await?;

        let upsert_mutation = r#"
            mutation UpsertCandidateGuide($input: UpsertCandidateGuideInput!) {
                upsertCandidateGuide(input: $input) {
                    id
                }
            }
        "#;
        let upsert_response: serde_json::Value = harness
            .execute_query(
                upsert_mutation,
                Some(async_graphql::Variables::from_json(serde_json::json!({
                    "input": {
                        "name": "Export Test Guide",
                        "organizationId": organization_id.to_string(),
                        "raceIds": [race_one_id.to_string(), race_two_id.to_string()]
                    }
                }))),
                Some(user_id),
                None,
            )
            .await?;

        let candidate_guide_id = upsert_response["upsertCandidateGuide"]["id"]
            .as_str()
            .expect("candidate guide id")
            .to_string();
        let candidate_guide_uuid = Uuid::parse_str(&candidate_guide_id)?;

        sqlx::query!(
            r#"
            INSERT INTO candidate_guide_races (candidate_guide_id, race_id)
            VALUES ($1, $2)
            "#,
            candidate_guide_uuid,
            race_without_embed_id
        )
        .execute(&harness.pool)
        .await?;

        let extra_embed_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO embed (
                id, organization_id, name, embed_type, attributes, created_by, updated_by
            )
            VALUES (
                $1, $2, 'duplicate race embed', 'candidate_guide',
                jsonb_build_object('candidateGuideId', $3::text, 'raceId', $4::text),
                $5, $5
            )
            "#,
            extra_embed_id,
            organization_id,
            candidate_guide_id,
            race_one_id.to_string(),
            user_id
        )
        .execute(&harness.pool)
        .await?;

        let embeds = sqlx::query!(
            r#"
            SELECT id, attributes->>'raceId' AS race_id
            FROM embed
            WHERE embed_type = 'candidate_guide'
                AND attributes->>'candidateGuideId' = $1
            ORDER BY created_at ASC
            "#,
            candidate_guide_id
        )
        .fetch_all(&harness.pool)
        .await?;

        let first_race_embed_id = embeds
            .iter()
            .find(|row| row.race_id.as_deref() == Some(&race_one_id.to_string()))
            .expect("embed for race one")
            .id;
        let second_race_embed_id = embeds
            .iter()
            .find(|row| row.race_id.as_deref() == Some(&race_two_id.to_string()))
            .expect("embed for race two")
            .id;

        let download_mutation = r#"
            mutation DownloadAllCandidateGuideData($candidateGuideId: ID!, $raceId: ID) {
                downloadAllCandidateGuideData(
                    candidateGuideId: $candidateGuideId
                    raceId: $raceId
                )
            }
        "#;

        let all_races: serde_json::Value = harness
            .execute_query(
                download_mutation,
                Some(async_graphql::Variables::from_json(serde_json::json!({
                    "candidateGuideId": candidate_guide_id
                }))),
                Some(user_id),
                None,
            )
            .await?;

        let csv_string = all_races["downloadAllCandidateGuideData"]
            .as_str()
            .expect("csv string");
        let (headers, rows) = parse_csv(csv_string)?;

        assert_eq!(
            headers.iter().collect::<Vec<_>>(),
            vec![
                "race_title",
                "first_name",
                "middle_name",
                "last_name",
                "preferred_name",
                "suffix",
                "full_name",
                "email",
                "form_link",
                "was_candidate_emailed",
                "last_submission",
                "widget_script",
                "embed_code",
            ]
        );
        assert_eq!(rows.len(), 4, "one row per candidate across all races");

        let expected_widget_script = format!(
            r#"<script async src="{}/widget-client-v2.js"></script>"#,
            config::Config::default()
                .web_app_url
                .as_str()
                .trim_end_matches('/')
        );
        let widget_script_idx = headers
            .iter()
            .position(|header| header == "widget_script")
            .unwrap();
        let embed_code_idx = headers
            .iter()
            .position(|header| header == "embed_code")
            .unwrap();
        let first_name_idx = headers
            .iter()
            .position(|header| header == "first_name")
            .unwrap();
        let race_title_idx = headers
            .iter()
            .position(|header| header == "race_title")
            .unwrap();

        for row in &rows {
            assert_eq!(
                row.get(widget_script_idx),
                Some(expected_widget_script.as_str())
            );
        }

        let alice_race_one = rows
            .iter()
            .find(|row| {
                row.get(first_name_idx) == Some("Alice")
                    && row.get(race_title_idx) == Some("Texas Governor General 2026")
            })
            .expect("alice in race one");
        assert_eq!(
            alice_race_one.get(embed_code_idx),
            Some(
                format!(
                    r#"<div class="populist-embed" data-embed-id="{first_race_embed_id}"></div>"#
                )
                .as_str()
            )
        );

        let bob_race_one = rows
            .iter()
            .find(|row| row.get(first_name_idx) == Some("Bob"))
            .expect("bob in race one");
        assert_eq!(
            bob_race_one.get(embed_code_idx),
            alice_race_one.get(embed_code_idx),
            "candidates in the same race share embed code"
        );

        let cara_row = rows
            .iter()
            .find(|row| row.get(first_name_idx) == Some("Cara"))
            .expect("cara in race two");
        assert_eq!(
            cara_row.get(embed_code_idx),
            Some(
                format!(
                    r#"<div class="populist-embed" data-embed-id="{second_race_embed_id}"></div>"#
                )
                .as_str()
            )
        );

        let alice_without_embed = rows
            .iter()
            .find(|row| {
                row.get(first_name_idx) == Some("Alice")
                    && row.get(race_title_idx) == Some("Texas House District 1 General 2026")
            })
            .expect("alice in race without embed");
        assert_eq!(alice_without_embed.get(embed_code_idx), Some(""));

        let race_one_only: serde_json::Value = harness
            .execute_query(
                download_mutation,
                Some(async_graphql::Variables::from_json(serde_json::json!({
                    "candidateGuideId": candidate_guide_id,
                    "raceId": race_one_id.to_string()
                }))),
                Some(user_id),
                None,
            )
            .await?;
        let (_, race_one_rows) = parse_csv(
            race_one_only["downloadAllCandidateGuideData"]
                .as_str()
                .expect("filtered csv"),
        )?;
        assert_eq!(race_one_rows.len(), 2);
        assert!(race_one_rows
            .iter()
            .all(|row| { row.get(race_title_idx) == Some("Texas Governor General 2026") }));

        sqlx::query!(
            "DELETE FROM embed WHERE organization_id = $1",
            organization_id
        )
        .execute(&harness.pool)
        .await?;
        sqlx::query!(
            "DELETE FROM candidate_guide_races WHERE candidate_guide_id = $1",
            candidate_guide_uuid
        )
        .execute(&harness.pool)
        .await?;
        sqlx::query!(
            "DELETE FROM candidate_guide WHERE id = $1",
            candidate_guide_uuid
        )
        .execute(&harness.pool)
        .await?;
        sqlx::query!(
            "DELETE FROM race_candidates WHERE race_id = ANY($1::uuid[])",
            &[race_one_id, race_two_id, race_without_embed_id] as &[Uuid]
        )
        .execute(&harness.pool)
        .await?;
        sqlx::query!(
            "DELETE FROM race WHERE id = ANY($1::uuid[])",
            &[race_one_id, race_two_id, race_without_embed_id] as &[Uuid]
        )
        .execute(&harness.pool)
        .await?;
        sqlx::query!(
            "DELETE FROM politician WHERE id = ANY($1::uuid[])",
            &[alice_id, bob_id, cara_id] as &[Uuid]
        )
        .execute(&harness.pool)
        .await?;
        sqlx::query!("DELETE FROM office WHERE id = $1", office_id)
            .execute(&harness.pool)
            .await?;
        sqlx::query!(
            "DELETE FROM organization_users WHERE organization_id = $1",
            organization_id
        )
        .execute(&harness.pool)
        .await?;
        sqlx::query!("DELETE FROM organization WHERE id = $1", organization_id)
            .execute(&harness.pool)
            .await?;
        sqlx::query!("DELETE FROM user_profile WHERE user_id = $1", user_id)
            .execute(&harness.pool)
            .await?;
        sqlx::query!("DELETE FROM populist_user WHERE id = $1", user_id)
            .execute(&harness.pool)
            .await?;

        harness.cleanup().await?;
        Ok(())
    }
}
