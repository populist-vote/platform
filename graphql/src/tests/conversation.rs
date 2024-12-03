use crate::tests::harness::TestHarness;
use rand::seq::SliceRandom;

#[tokio::test]
async fn test_conversation_load_and_groups() -> anyhow::Result<()> {
    const NUM_USERS_PER_GROUP: usize = 5;
    const NUM_GROUPS: usize = 3;
    const NUM_USERS: usize = NUM_USERS_PER_GROUP * NUM_GROUPS;
    const NUM_STATEMENTS: usize = 100;
    const VOTES_PER_STATEMENT: usize = 10;

    // Set up test harness and create organization
    let harness = TestHarness::new().await?;
    let organization_id = harness
        .create_organization("Combined Test Organization")
        .await?;

    // Create users with a simple loop
    let mut user_ids = Vec::with_capacity(NUM_USERS);
    for i in 0..NUM_USERS {
        let email = format!("user-{}@example.com", i);
        let user_id = harness.create_user(&email, None).await?;
        user_ids.push(user_id);
    }
    println!("Created {} users", user_ids.len());

    // Create conversation
    let conversation_id = {
        let create_conversation_query = r#"
            mutation CreateConversation($input: CreateConversationInput!) {
                createConversation(input: $input) {
                    id
                    topic
                }
            }
        "#;

        let conversation_variables = serde_json::json!({
            "input": {
                "topic": "Combined Load and Groups Test",
                "organizationId": organization_id.to_string()
            }
        });

        let conversation_response: serde_json::Value = harness
            .execute_query(
                create_conversation_query,
                Some(async_graphql::Variables::from_json(conversation_variables)),
                None,
                None,
            )
            .await?;

        conversation_response["createConversation"]["id"]
            .as_str()
            .expect("Conversation ID should be present")
            .to_string()
    };

    // Create statements with a simple loop
    let create_statement_query = r#"
        mutation CreateStatement($input: AddStatementInput!) {
            addStatement(input: $input) {
                id
                content
            }
        }
    "#;

    let mut statement_ids = Vec::with_capacity(NUM_STATEMENTS);
    for i in 0..NUM_STATEMENTS {
        let user_id = &user_ids[i % user_ids.len()];

        let statement_variables = serde_json::json!({
            "input": {
                "conversationId": conversation_id,
                "content": format!("Test statement {}", i),
                "userId": Some(user_id.to_string())
            }
        });

        let response: serde_json::Value = harness
            .execute_query(
                create_statement_query,
                Some(async_graphql::Variables::from_json(statement_variables)),
                Some(user_id.clone()),
                None,
            )
            .await?;

        let statement_id = response["addStatement"]["id"].as_str().unwrap().to_string();
        statement_ids.push(statement_id);

        if (i + 1) % 10 == 0 {
            println!("Created {} statements", i + 1);
        }
    }

    println!("Created {} total statements", statement_ids.len());

    // Helper function to determine vote type based on user group and statement
    let get_vote_type = |user_idx: usize, statement_idx: usize| -> &'static str {
        let group = user_idx / NUM_USERS_PER_GROUP;
        match group {
            0 => {
                if statement_idx % 5 == 0 {
                    "NEUTRAL"
                } else {
                    "SUPPORT"
                }
            } // Group A: Mostly supportive
            1 => {
                if statement_idx % 5 == 0 {
                    "NEUTRAL"
                } else {
                    "OPPOSE"
                }
            } // Group B: Mostly opposing
            _ => match statement_idx % 3 {
                // Group C: Mixed opinions
                0 => "SUPPORT",
                1 => "OPPOSE",
                _ => "NEUTRAL",
            },
        }
    };

    // Create votes with predetermined patterns
    let vote_mutation = r#"
        mutation VoteOnStatement($statementId: ID!, $voteType: ArgumentPosition!) {
            voteOnStatement(statementId: $statementId, voteType: $voteType) {
                id
            }
        }  
    "#;

    let start_time = std::time::Instant::now();
    let mut total_votes = 0;

    // Create votes for each user following their group's pattern
    for (user_idx, user_id) in user_ids.iter().enumerate() {
        for (stmt_idx, statement_id) in statement_ids.iter().enumerate() {
            let vote_type = get_vote_type(user_idx, stmt_idx);

            let vote_variables = serde_json::json!({
                "statementId": statement_id,
                "voteType": vote_type
            });

            let response: serde_json::Value = harness
                .execute_query(
                    vote_mutation,
                    Some(async_graphql::Variables::from_json(vote_variables)),
                    Some(user_id.clone()),
                    // TODO handle session id's for better testing
                    Some(uuid::Uuid::new_v4()),
                )
                .await?;

            assert!(response["voteOnStatement"]["id"].is_string());
            total_votes += 1;

            if total_votes % 100 == 0 {
                println!("Created {} votes", total_votes);
            }
        }
    }

    let duration = start_time.elapsed();
    println!("Created {} total votes in {:?}", total_votes, duration);

    // Verify opinion groups
    let opinion_groups_query = r#"
        query GetOpinionGroups($id: ID!) {
            conversationById(id: $id) {
                opinionGroups {
                    id
                    characteristicVotes {
                        statementId
                        meanSentiment
                        consensusLevel
                        significanceLevel
                    }
                }
                statements(limit: 100) {
                    id
                    content
                    agreeCount
                    createdAt
                    author {
                        id
                        firstName
                        lastName
                        profilePictureUrl
                    }
                }
                stats {
                    totalParticipants
                    totalVotes
                    totalStatements
                    avgVotesPerParticipant
                }
            }
        }
    "#;

    let final_response: serde_json::Value = harness
        .execute_query(
            opinion_groups_query,
            Some(async_graphql::Variables::from_json(serde_json::json!({
                "id": conversation_id
            }))),
            Some(user_ids[0].clone()),
            None,
        )
        .await?;

    // Verify statements count
    let total_statements = final_response["conversationById"]["statements"]
        .as_array()
        .unwrap()
        .len();
    assert_eq!(total_statements, NUM_STATEMENTS);

    // Verify opinion groups
    let groups = final_response["conversationById"]["opinionGroups"]
        .as_array()
        .expect("Should have opinion groups");

    println!("Found {} opinion groups: ", groups.len());

    assert!(
        groups.len() >= 2 && groups.len() <= 4,
        "Should find 2-4 groups given the voting patterns"
    );

    // Verify group properties
    for group in groups {
        let users = group["users"].as_array().unwrap();
        let characteristic_votes = group["characteristicVotes"].as_array().unwrap();

        assert!(!users.is_empty(), "Group should have users");
        assert!(
            !characteristic_votes.is_empty(),
            "Group should have characteristic votes"
        );

        // Check vote properties
        for vote in characteristic_votes {
            let mean = vote["meanSentiment"].as_f64().unwrap();
            let consensus = vote["consensusLevel"].as_f64().unwrap();
            let significance = vote["significanceLevel"].as_f64().unwrap();

            assert!(
                (-1.0..=1.0).contains(&mean),
                "Mean sentiment should be between -1 and 1"
            );
            assert!(
                (0.0..=1.0).contains(&consensus),
                "Consensus should be between 0 and 1"
            );
            assert!(
                (0.0..=1.0).contains(&significance),
                "Significance should be between 0 and 1"
            );

            if consensus > 0.7 {
                assert!(
                    significance > 0.5,
                    "High-consensus votes should have significant participation"
                );
            }
        }
    }

    Ok(())
}
