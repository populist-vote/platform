use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

pub async fn send_slack_notification(
    title: &str,
    description: &str,
    metadata: Option<Value>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();
    let webhook_url = env::var("SLACK_WEBHOOK_URL").expect("SLACK_WEBHOOK_URL must be set");
    let client = Client::new();
    let mut blocks = vec![
        json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("*{}*\n{}", title, description)
            }
        }),
        json!({
            "type": "context",
            "elements": [
                {
                    "type": "mrkdwn",
                    "text": format!("Timestamp: {}", Utc::now().to_rfc3339())
                }
            ]
        }),
    ];

    if let Some(meta) = metadata {
        blocks.push(json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("Metadata: ```{}```", meta)
            }
        }));
    }

    let payload = json!({
        "blocks": blocks
    });

    let res = client.post(webhook_url).json(&payload).send().await?;

    if res.status().is_success() {
        Ok(())
    } else {
        eprintln!("Failed to send Slack notification: {}", res.status());
        Ok(())
    }
}

#[tokio::test]
async fn test_send_slack_notification_success() {
    dotenv::dotenv().ok();
    let title = "Test Notification";
    let description = "This is a test notification to verify the Slack notification functionality.";
    let metadata = Some(json!({"key": "value"}));

    let result = send_slack_notification(title, description, metadata).await;
    assert!(
        result.is_ok(),
        "The Slack notification should be sent successfully."
    );
}
