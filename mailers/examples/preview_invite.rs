//! Render the checked-in invite template locally without sending email.
//! cargo run -p mailers --example preview_invite > /tmp/populist-invite.html
use handlebars::Handlebars;
use serde_json::json;

fn main() {
    let organization = std::env::args().nth(1).unwrap_or_else(|| "Populist".into());
    let html = Handlebars::new()
        .render_template(
            include_str!("../templates/invite.html"),
            &json!({
                "organization_name": organization,
                "politician_name": null,
                "recipient_email": "invitee+organization@example.com",
                "invite_url": "https://staging.populist.us/register?inviteToken=9370dbac-bba8-46bf-b899-2c0b8fe93b57&email=invitee%2Borganization%40example.com",
            }),
        )
        .unwrap();
    println!("{html}");
}
