use lettre::{
    transport::smtp::{authentication::Credentials, response::Response, Error},
    Message, SmtpTransport, Transport,
};

pub struct Mailer {
    smtp: SmtpTransport,
}

impl Mailer {
    pub fn new(host: &str, username: &str, password: &str) -> Self {
        Mailer {
            smtp: SmtpTransport::relay(host)
                .unwrap()
                .credentials(Credentials::new(username.to_owned(), password.to_owned()))
                .build(),
        }
    }

    pub fn from(smtp: SmtpTransport) -> Self {
        Mailer { smtp }
    }

    pub fn send(&self, from: &str, to: &str, subject: &str, body: &str) -> Result<Response, Error> {
        let email = Message::builder()
            .from(from.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .body(body.to_string())
            .unwrap();
        self.smtp.send(&email)
    }
}
