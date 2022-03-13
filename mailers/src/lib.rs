use sendgrid::v3::{Content, Email, Message, Personalization, Sender};
pub struct EmailPrototype {
    pub recipient: String,
    pub subject: String,
    pub template_id: Option<String>,
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
    use crate::{EmailClient, EmailPrototype};
    use dotenv::dotenv;
    #[tokio::test]
    async fn test_send_email() {
        dotenv().ok();
        let prototype = EmailPrototype {
            recipient: "wileymckayconte@gmail.com".to_string(),
            subject: "Test Email".to_string(),
            template_id: None,
            template_data: None,
        };
        let result = EmailClient::send_mail(prototype).await;
        assert_eq!(result.unwrap(), 202);
    }
}
