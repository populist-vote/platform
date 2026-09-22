# Organization and profile invitation email

`invite.html` and `invite.txt` are the reviewed sources for the invitation’s SendGrid dynamic template. Welcome, password reset, and password change emails use their existing templates.

- Template: `d-f5047d97b8994724bd39e909e8a86ac6`
- Version: `0f128499-b097-4bb1-ad7b-fefcbdc91c90`
- Subject: `Your Populist invitation`
- Previous template (kept for rollback): `d-4a43724169e54aa7a3a12553f7163808`

The new template is already provisioned in the same SendGrid account as the previous template. Its HTML and plain text were fetched back and verified against these files. Deploying the backend change to `INVITE_EMAIL_TEMPLATE_ID` selects it; there is no live-template activation step. The old template is unchanged.

## Preview and test without delivering email

```sh
cargo test -p mailers
cargo run -p mailers --example preview_invite > /tmp/populist-invite.html
```

Open the resulting HTML in a browser at desktop and mobile widths. Pass an organization name as the example’s first argument to preview longer names. These checks do not deliver email. Browser previews do not replace testing in Outlook or Gmail; the Outlook conditional button is checked for the exact invitation URL by the template tests.

Template data comes from `send_invite_email`: `organization_name`, `politician_name`, `recipient_email`, and `invite_url`. The HTML uses escaped Handlebars values; plain text preserves the original invitation URL. The table layout has a 600px desktop maximum, padding on cells, and inline styles. Both the Outlook VML button and regular HTML button use `invite_url`.

For future changes, update these sources, preview and test them, then provision a separate SendGrid **dynamic** template with a code-editor version. Upload both files, set `generate_plain_content` to `false`, and use the subject above. Verify the uploaded content, update the template ID in `mailers/src/lib.rs`, and record the new IDs here in the same PR. This keeps reviewed template changes tied to a backend deployment instead of changing active production emails immediately.
