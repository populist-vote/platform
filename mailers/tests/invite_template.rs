use handlebars::Handlebars;
use serde_json::{json, Value};

const HTML: &str = include_str!("../templates/invite.html");
const TEXT: &str = include_str!("../templates/invite.txt");
const INVITE_URL: &str = "https://staging.populist.us/register?inviteToken=9370dbac-bba8-46bf-b899-2c0b8fe93b57&email=invitee%2Borganization%40example.com";

fn render(template: &str, organization: Value, politician: Value) -> String {
    Handlebars::new()
        .render_template(
            template,
            &json!({
                "organization_name": organization,
                "politician_name": politician,
                "recipient_email": "invitee+organization@example.com",
                "invite_url": INVITE_URL,
            }),
        )
        .unwrap()
}

#[test]
fn every_email_link_preserves_the_invitation_including_outlook() {
    let html = render(HTML, json!("Populist"), Value::Null);
    let escaped_url = handlebars::html_escape(INVITE_URL);
    assert_eq!(html.matches(&format!("href=\"{escaped_url}\"")).count(), 3);
    let outlook_button = html
        .split("<v:roundrect")
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(outlook_button.contains(&format!("href=\"{escaped_url}\"")));
    assert!(!html.contains("https://populist.us/login"));
    let text = render(TEXT, json!("Populist"), Value::Null);
    assert!(text.contains(INVITE_URL));
    assert!(!text.contains("&amp;"));
    assert!(text.contains("invitee+organization@example.com"));
}

#[test]
fn invitation_copy_handles_organizations_profiles_and_generic_invites() {
    for (organization, politician, expected) in [
        (
            json!("Populist"),
            Value::Null,
            "You’ve been invited to join Populist.",
        ),
        (
            Value::Null,
            json!("Jane Doe"),
            "help manage Jane Doe’s profile",
        ),
        (
            json!("Newsroom"),
            json!("Jane Doe"),
            "You can also help manage Jane Doe’s profile",
        ),
        (Value::Null, Value::Null, "You’ve been invited to Populist."),
    ] {
        let text = render(TEXT, organization.clone(), politician.clone());
        assert!(text.contains(expected), "{text}");
        assert!(text.contains("sign in to an existing account or create a new one"));
        assert!(!text.contains("Populist on Populist"));
        let html = render(HTML, organization, politician);
        assert!(!html.contains("{{"));
    }
}

#[test]
fn names_and_recipient_email_cannot_inject_html() {
    let html = Handlebars::new()
        .render_template(
            HTML,
            &json!({
                "organization_name": "<script>alert(1)</script> & News",
                "politician_name": "<img src=x onerror=alert(1)>",
                "recipient_email": "<b>alias</b>@example.com",
                "invite_url": INVITE_URL,
            }),
        )
        .unwrap();
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt; &amp; News"));
    assert!(html.contains("&lt;b&gt;alias&lt;/b&gt;@example.com"));
    assert!(!html.contains("<script>"));
    assert!(!html.contains("<img src=x"));
}
