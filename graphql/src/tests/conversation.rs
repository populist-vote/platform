use crate::tests::harness::TestHarness;
use rand::seq::SliceRandom;
use rand::Rng;

#[tokio::test]
async fn test_conversation_load_and_groups() -> anyhow::Result<()> {
    const NUM_USERS: usize = 25;
    const NUM_STATEMENTS: usize = 100;
    const NUM_VOTES: usize = 500;
    const NUM_GROUPS: usize = 3;

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

    struct GroupBehavior {
        support_prob: f64,
        neutral_prob: f64,
    }

    const GROUP_BEHAVIORS: [GroupBehavior; 5] = [
        GroupBehavior {
            support_prob: 0.8,
            neutral_prob: 0.9,
        }, // Strong supporters
        GroupBehavior {
            support_prob: 0.6,
            neutral_prob: 0.85,
        }, // Moderate supporters
        GroupBehavior {
            support_prob: 0.4,
            neutral_prob: 0.7,
        }, // Centrists
        GroupBehavior {
            support_prob: 0.2,
            neutral_prob: 0.35,
        }, // Moderate opposers
        GroupBehavior {
            support_prob: 0.1,
            neutral_prob: 0.2,
        }, // Strong opposers
    ];

    let get_vote_type = |user_idx: usize| -> &'static str {
        let mut rng = rand::thread_rng();
        let group = user_idx / (NUM_USERS / NUM_GROUPS);
        let behavior = &GROUP_BEHAVIORS[group];

        let random = rng.gen_range(0.0..1.0);
        if random < behavior.support_prob {
            "SUPPORT"
        } else if random < behavior.neutral_prob {
            "NEUTRAL"
        } else {
            "OPPOSE"
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

    let mut total_votes = 0;
    let mut rng = rand::thread_rng();
    let mut vote_pairs: Vec<(uuid::Uuid, String, &'static str)> = Vec::with_capacity(NUM_VOTES);

    for _ in 0..NUM_VOTES {
        let user_idx = rng.gen_range(0..NUM_USERS);
        let user_id = &user_ids[user_idx];
        let statement_id = statement_ids.choose(&mut rng).unwrap();
        let vote_type = get_vote_type(user_idx);
        vote_pairs.push((user_id.clone(), statement_id.clone(), vote_type));
    }

    for (user_id, statement_id, vote_type) in vote_pairs {
        let vote_variables = serde_json::json!({
            "statementId": statement_id,
            "voteType": vote_type
        });

        let response: serde_json::Value = harness
            .execute_query(
                vote_mutation,
                Some(async_graphql::Variables::from_json(vote_variables)),
                Some(user_id),
                Some(uuid::Uuid::new_v4()),
            )
            .await?;

        assert!(response["voteOnStatement"]["id"].is_string());
        total_votes += 1;

        if total_votes % 100 == 0 {
            println!("Created {} votes", total_votes);
        }
    }

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
                    summary
                }
                statements(limit: 200) {
                    id
                    content
                    supportVotes
                    opposeVotes
                    neutralVotes
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
        groups.len() >= 2 && groups.len() <= 5,
        "Should have between 2 and 5 opinion groups"
    );

    // Verify group properties
    for group in groups {
        let characteristic_votes = group["characteristicVotes"].as_array().unwrap();
        let summary = group["summary"].as_str().unwrap();

        println!("group = {:?}", group);

        assert!(
            !characteristic_votes.is_empty(),
            "Group should have characteristic votes"
        );

        // Test that summary meets basic quality standards
        assert!(
            !summary.is_empty() && summary.len() > 20,
            "Summary should be a meaningful length. Got: '{}'",
            summary
        );

        // Test that it's not just an error message
        assert!(
            !summary.contains("unavailable"),
            "Summary should be available. Got: '{}'",
            summary
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
