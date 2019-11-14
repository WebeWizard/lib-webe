use lettre::smtp::authentication::IntoCredentials;
use lettre::smtp::error::Error as LettreError;
use lettre::{SmtpClient, SmtpConnectionManager, Transport};
use lettre_email::error::Error as LettreEmailError;
use lettre_email::EmailBuilder;
use r2d2::Pool;
use std::ops::DerefMut;

pub type EmailManager = Pool<SmtpConnectionManager>;

#[derive(Debug)]
pub enum EmailError {
  PoolError(r2d2::Error),
  ManagerError(LettreError),
  ClientError(LettreEmailError),
}

impl From<LettreError> for EmailError {
  fn from(err: LettreError) -> EmailError {
    EmailError::ManagerError(err)
  }
}

impl From<r2d2::Error> for EmailError {
  fn from(err: r2d2::Error) -> EmailError {
    EmailError::PoolError(err)
  }
}

impl From<LettreEmailError> for EmailError {
  fn from(err: LettreEmailError) -> EmailError {
    EmailError::ClientError(err)
  }
}

impl From<EmailError> for crate::AuthError {
  fn from(err: EmailError) -> crate::AuthError {
    crate::AuthError::EmailError(err)
  }
}

pub fn create_smtp_pool(
  address: String,
  user: String,
  pass: String,
) -> Result<Pool<SmtpConnectionManager>, EmailError> {
  // build the email connection pool
  match SmtpClient::new_simple(address.as_str()) {
    Ok(mut email_client) => {
      let email_creds = (user, pass).into_credentials();
      email_client = email_client.credentials(email_creds);
      let email_manager = SmtpConnectionManager::new(email_client)?;
      let pool = Pool::builder().max_size(1).build(email_manager)?;
      return Ok(pool);
    }
    Err(err) => return Err(EmailError::ManagerError(err)),
  }
}

pub trait EmailApi {
  fn send_email(&self, builder: EmailBuilder) -> Result<(), EmailError>;
}

impl EmailApi for EmailManager {
  fn send_email(&self, builder: EmailBuilder) -> Result<(), EmailError> {
    let mut conn = self.get()?;
    let client = conn.deref_mut();
    let email = builder.build()?;
    client.send(email.into())?;
    return Ok(());
  }
}
