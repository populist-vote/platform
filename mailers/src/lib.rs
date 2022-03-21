use sendgrid::v3::{Content, Email, Message, Personalization, Sender};
use serde_json::json;
use std::collections::HashMap;

static POPULIST_FROM_EMAIL: &str = "info@populist.us";

static WELCOME_EMAIL_ID: &str = "d-edaebe0011f441348a0f310c05813cb0";
static FORGOT_PASSWORD_ID: &str = "d-819b5a97194e4b3e99efa5ec2d9c6e6e";

pub struct EmailPrototype {
    pub recipient: String,
    pub subject: String,
    pub template_id: String,
    pub template_data: Option<String>,
}

pub struct EmailClient;

impl EmailClient {
    pub async fn send_mail(prototype: EmailPrototype) -> Result<u16, sendgrid::SendgridError> {
        let p = Personalization::new(Email::new(&prototype.recipient));
        let mail = Message::new(Email::new("info@populist.us"))
            .set_subject(&prototype.subject)
            .add_content(
                Content::new().set_content_type("text/html").set_value(
                    prototype
                        .template_data
                        .unwrap_or_else(|| "Some default email data".to_string()),
                ),
            )
            .add_personalization(p)
            .set_template_id(&prototype.template_id);

        let api_key = std::env::var("SENDGRID_API_KEY").unwrap();
        let sender = Sender::new(api_key);
        let response = sender.send(&mail).await;
        let status = response.unwrap().status();
        Ok(status.into())
    }

    // Send out email with username, and link to confirm & set password
    pub async fn send_welcome_email(
        recipient_email: String,
        account_confirmation_url: String,
    ) -> Result<u16, sendgrid::SendgridError> {
        let p = Personalization::new(Email::new(&recipient_email))
            .add_dynamic_template_data_json(&json!({
                "account_confirmation_url": &account_confirmation_url
            }))
            .unwrap();
        let mail = Message::new(Email::new(POPULIST_FROM_EMAIL))
            .set_template_id(WELCOME_EMAIL_ID)
            .add_personalization(p);

        let api_key = std::env::var("SENDGRID_API_KEY").unwrap();
        let sender = Sender::new(api_key);
        let response = sender.send(&mail).await;
        let status = response.unwrap().status();
        Ok(status.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::{EmailClient, EmailPrototype, WELCOME_EMAIL_ID};
    use dotenv::dotenv;
    #[tokio::test]
    async fn test_send_email() {
        dotenv().ok();
        let prototype = EmailPrototype {
            recipient: "wileymckayconte@gmail.com".to_string(),
            subject: "Test Email".to_string(),
            template_id: WELCOME_EMAIL_ID.to_string(),
            template_data: None,
        };
        let result = EmailClient::send_mail(prototype).await;
        assert_eq!(result.unwrap(), 202);
    }

    #[tokio::test]
    async fn test_send_welcome_email() {
        dotenv().ok();
        let result = EmailClient::send_welcome_email(
            "wileymckayconte@gmail.com".to_string(),
            "https://populist.us/test".to_string(),
        )
        .await;
        assert_eq!(result.unwrap(), 202);
    }
}
