use anyhow::Result;
use tracing::{info, warn};

use super::client::BlueskyClient;
use super::session::{self, SessionData};

pub enum AuthResult {
    Success(String),
    NeedsLogin,
}

pub async fn try_restore_session(client: &BlueskyClient) -> AuthResult {
    match session::load_session() {
        Ok(Some(session_data)) => {
            info!("Found saved session for {}", session_data.handle);
            match client
                .login_app_password(&session_data.handle, &session_data.access_jwt)
                .await
            {
                Ok(_) => {
                    info!("Session restored for {}", session_data.handle);
                    AuthResult::Success(session_data.handle)
                }
                Err(e) => {
                    warn!("Failed to restore session: {}", e);
                    AuthResult::NeedsLogin
                }
            }
        }
        Ok(None) => {
            info!("No saved session found");
            AuthResult::NeedsLogin
        }
        Err(e) => {
            warn!("Error loading session: {}", e);
            AuthResult::NeedsLogin
        }
    }
}

pub async fn login_with_app_password(
    client: &BlueskyClient,
    identifier: &str,
    password: &str,
) -> Result<String> {
    client.login_app_password(identifier, password).await?;

    let did = client.did().await.unwrap_or_default();
    let handle = identifier.to_string();

    let session_data = SessionData {
        did,
        handle: handle.clone(),
        access_jwt: password.to_string(),
        refresh_jwt: String::new(),
        pds_endpoint: None,
    };
    session::save_session(&session_data)?;

    Ok(handle)
}

pub fn logout() -> Result<()> {
    session::clear_session()?;
    Ok(())
}
