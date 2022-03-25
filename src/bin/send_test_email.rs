use anyhow::Result;
use lettre;
use lettre::Transport;
use lettre_email;
use native_tls;

fn tls() -> native_tls::TlsConnector {
    native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap()
}

fn smtp(user: &str) -> lettre::SmtpTransport {
    let creds = lettre::smtp::authentication::Credentials::new(user.to_string(), user.to_string());
    lettre::SmtpClient::new(
        &format!(
            "{}:3465",
            std::env::var("TEST_HOST").unwrap_or("127.0.0.1".to_string())
        ),
        lettre::ClientSecurity::Wrapper(lettre::ClientTlsParameters {
            connector: tls(),
            domain: "smpt.example.com".to_string(),
        }),
    )
    .unwrap()
    .credentials(creds)
    .transport()
}

fn main() -> Result<()> {
    let to = "test@greenmail.com";
    let mut s = smtp(&to);

    let e = lettre_email::Email::builder()
        .from("sender@localhost")
        .to(to)
        .subject("My first e-mail")
        .text("Hello world from SMTP")
        .build()
        .unwrap();
    s.send(e.into())?;

    Ok(())
}
