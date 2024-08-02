use sendgrid::v3::{Email, Message, Personalization, Sender};
use serde_json::json;

static POPULIST_FROM_EMAIL: &str = "info@populist.us";

static WELCOME_EMAIL_TEMPLATE_ID: &str = "d-edaebe0011f441348a0f310c05813cb0";
static FORGOT_PASSWORD_TEMPLATE_ID: &str = "d-819b5a97194e4b3e99efa5ec2d9c6e6e";
static PASSWORD_CHANGED_TEMPLATE_ID: &str = "d-a5a79e8740864187aadfdd0bc07bbb97";
pub struct EmailClient {
    from: Email,
    sender: Sender,
}

impl Default for EmailClient {
    fn default() -> Self {
        let api_key = dotenv::var("SENDGRID_API_KEY").expect("SENDGRID_API_KEY must be set");
        Self {
            from: Email::new(POPULIST_FROM_EMAIL).set_name("Populist"),
            sender: Sender::new(api_key),
        }
    }
}

impl EmailClient {
    pub fn new(api_key: String) -> Self {
        Self {
            from: Email::new(POPULIST_FROM_EMAIL).set_name("Populist"),
            sender: Sender::new(api_key),
        }
    }

    // Send out email with username, and link to confirm & set password
    pub async fn send_welcome_email(
        &self,
        recipient_email: String,
        account_confirmation_url: String,
    ) -> Result<u16, sendgrid::SendgridError> {
        let p = Personalization::new(Email::new(&recipient_email))
            .add_dynamic_template_data_json(&json!({
                "account_confirmation_url": &account_confirmation_url
            }))
            .unwrap();
        let mail = Message::new(self.from.clone())
            .set_template_id(WELCOME_EMAIL_TEMPLATE_ID)
            .add_personalization(p);
        let response = self.sender.send(&mail).await;
        let status = response.unwrap().status();
        Ok(status.into())
    }

    pub async fn send_invite_email(
        &self,
        recipient_email: String,
        invite_url: String,
    ) -> Result<u16, sendgrid::SendgridError> {
        let p = Personalization::new(Email::new(&recipient_email))
            .add_dynamic_template_data_json(&json!({ "invite_url": &invite_url }))
            .unwrap();
        let mail = Message::new(self.from.clone())
            .set_template_id(WELCOME_EMAIL_TEMPLATE_ID)
            .add_personalization(p);
        let response = self.sender.send(&mail).await;
        let status = response.unwrap().status();
        Ok(status.into())
    }

    pub async fn send_reset_password_email(
        &self,
        recipient_email: String,
        reset_password_url: String,
    ) -> Result<u16, sendgrid::SendgridError> {
        let p = Personalization::new(Email::new(&recipient_email))
            .add_dynamic_template_data_json(&json!({ "reset_password_url": &reset_password_url }))
            .unwrap();
        let mail = Message::new(self.from.clone())
            .set_template_id(FORGOT_PASSWORD_TEMPLATE_ID)
            .add_personalization(p);
        let response = self.sender.send(&mail).await;
        let status = response.unwrap().status();
        Ok(status.into())
    }

    pub async fn send_password_changed_email(
        &self,
        recipient_email: String,
    ) -> Result<u16, sendgrid::SendgridError> {
        let p = Personalization::new(Email::new(&recipient_email));
        let mail = Message::new(self.from.clone())
            .set_template_id(PASSWORD_CHANGED_TEMPLATE_ID)
            .add_personalization(p);

        let response = self.sender.send(&mail).await;
        let status = response.unwrap().status();
        Ok(status.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::EmailClient;
    use dotenv::dotenv;

    #[tokio::test]
    #[ignore]
    async fn test_send_welcome_email() {
        dotenv().ok();
        let client = EmailClient::default();
        let result = client
            .send_welcome_email(
                "wileymckayconte@gmail.com".to_string(),
                "https://populist.us/test".to_string(),
            )
            .await;
        assert_eq!(result.unwrap(), 202);
    }

    #[tokio::test]
    #[ignore]
    async fn test_send_reset_password_email() {
        dotenv().ok();
        let client = EmailClient::default();
        let result = client
            .send_reset_password_email(
                "wiley@populist.us".to_string(),
                "https://populist.us/test".to_string(),
            )
            .await;

        assert_eq!(result.unwrap(), 202);
    }

    #[tokio::test]
    #[ignore]
    async fn test_password_changed_email() {
        dotenv().ok();
        let client = EmailClient::default();
        let result = client
            .send_password_changed_email("wiley@populist.us".to_string())
            .await;
        assert_eq!(result.unwrap(), 202);
    }
}
