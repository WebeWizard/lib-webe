use lettre::smtp::authentication::IntoCredentials;
use lettre::smtp::error::Error;
use lettre::{SmtpClient, SmtpConnectionManager};
use r2d2::Pool;

#[derive(Debug)]
pub enum EmailError {
    PoolError(r2d2::Error),
    ManagerError(Error),
    ClientError(Error),
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
            match SmtpConnectionManager::new(email_client) {
                Ok(email_manager) => match Pool::builder().max_size(1).build(email_manager) {
                    Ok(email_pool) => return Ok(email_pool),
                    Err(err) => return Err(EmailError::PoolError(err)),
                },
                Err(err) => return Err(EmailError::ManagerError(err)),
            }
        }
        Err(err) => return Err(EmailError::ClientError(err)),
    }
}
