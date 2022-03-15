use mail_client::config;

#[test]
fn test_login() {
    let conf = config::Config {
        connection: config::Connection {
            hostname: "127.0.0.1".to_string(),
            username: "readonly-test@greenmail".to_string(),
            password: "readonly-test@greenmail".to_string(),
            port: 3993,
        },
        imap_options: None,
        scripts: None,
    };
    let mut session = mail_client::login(&conf).unwrap();
    session
        .logout()
        .unwrap();
}
