use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sha2::{Digest, Sha256};
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::info;

use super::dpop::DpopKeyPair;

const REDIRECT_PORT: u16 = 23847;
const CLIENT_ID: &str = "http://localhost";

pub struct OAuthFlow {
    dpop: DpopKeyPair,
    code_verifier: String,
    state: String,
    auth_server: String,
    token_endpoint: String,
    par_endpoint: Option<String>,
}

impl OAuthFlow {
    pub async fn start(handle: &str) -> Result<Self> {
        let dpop = DpopKeyPair::generate()?;
        let code_verifier = generate_code_verifier();
        let state = uuid::Uuid::new_v4().to_string();

        let (auth_server, token_endpoint, par_endpoint) = discover_auth_server(handle).await?;

        Ok(OAuthFlow {
            dpop,
            code_verifier,
            state,
            auth_server,
            token_endpoint,
            par_endpoint,
        })
    }

    pub async fn authorize(&self) -> Result<String> {
        let code_challenge = {
            let hash = Sha256::digest(self.code_verifier.as_bytes());
            URL_SAFE_NO_PAD.encode(hash)
        };

        let redirect_uri = format!("http://127.0.0.1:{}/callback", REDIRECT_PORT);

        let auth_url = if let Some(ref par_endpoint) = self.par_endpoint {
            let dpop_proof = self.dpop.create_proof("POST", par_endpoint, None, None)?;

            let client = reqwest::Client::new();
            let resp = client
                .post(par_endpoint)
                .header("DPoP", &dpop_proof)
                .form(&[
                    ("response_type", "code"),
                    ("client_id", CLIENT_ID),
                    ("redirect_uri", &redirect_uri),
                    ("state", &self.state),
                    ("code_challenge", &code_challenge),
                    ("code_challenge_method", "S256"),
                    ("scope", "atproto transition:generic"),
                ])
                .send()
                .await?;

            let body: serde_json::Value = resp.json().await?;
            let request_uri = body["request_uri"]
                .as_str()
                .ok_or_else(|| anyhow!("No request_uri in PAR response"))?;

            format!(
                "{}?client_id={}&request_uri={}",
                self.auth_server, CLIENT_ID, request_uri
            )
        } else {
            format!(
                "{}?response_type=code&client_id={}&redirect_uri={}&state={}&code_challenge={}&code_challenge_method=S256&scope=atproto+transition:generic",
                self.auth_server, CLIENT_ID, redirect_uri, self.state, code_challenge
            )
        };

        info!("Opening browser for OAuth authorization");
        open::that(&auth_url)?;

        let code = wait_for_callback(&self.state).await?;
        Ok(code)
    }

    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        let redirect_uri = format!("http://127.0.0.1:{}/callback", REDIRECT_PORT);
        let dpop_proof =
            self.dpop
                .create_proof("POST", &self.token_endpoint, None, None)?;

        let client = reqwest::Client::new();
        let resp = client
            .post(&self.token_endpoint)
            .header("DPoP", &dpop_proof)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &redirect_uri),
                ("client_id", CLIENT_ID),
                ("code_verifier", &self.code_verifier),
            ])
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;

        Ok(TokenResponse {
            access_token: body["access_token"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            refresh_token: body["refresh_token"]
                .as_str()
                .map(|s| s.to_string()),
            did: body["sub"].as_str().unwrap_or("").to_string(),
            dpop_nonce: body["dpop_nonce"]
                .as_str()
                .map(|s| s.to_string()),
        })
    }

    pub fn dpop(&self) -> &DpopKeyPair {
        &self.dpop
    }
}

#[derive(Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub did: String,
    pub dpop_nonce: Option<String>,
}

fn generate_code_verifier() -> String {
    let bytes: [u8; 32] = rand::random();
    URL_SAFE_NO_PAD.encode(bytes)
}

async fn discover_auth_server(
    handle: &str,
) -> Result<(String, String, Option<String>)> {
    let pds_url = format!("https://bsky.social/xrpc/com.atproto.identity.resolveHandle?handle={}", handle);
    let client = reqwest::Client::new();

    let resp = client.get(&pds_url).send().await?;
    let body: serde_json::Value = resp.json().await?;
    let _did = body["did"].as_str().unwrap_or("");

    let well_known = "https://bsky.social/.well-known/oauth-authorization-server";
    let resp = client.get(well_known).send().await?;
    let meta: serde_json::Value = resp.json().await?;

    let auth_endpoint = meta["authorization_endpoint"]
        .as_str()
        .unwrap_or("https://bsky.social/oauth/authorize")
        .to_string();
    let token_endpoint = meta["token_endpoint"]
        .as_str()
        .unwrap_or("https://bsky.social/oauth/token")
        .to_string();
    let par_endpoint = meta["pushed_authorization_request_endpoint"]
        .as_str()
        .map(|s| s.to_string());

    Ok((auth_endpoint, token_endpoint, par_endpoint))
}

async fn wait_for_callback(expected_state: &str) -> Result<String> {
    let listener =
        TcpListener::bind(format!("127.0.0.1:{}", REDIRECT_PORT))?;
    listener.set_nonblocking(true)?;

    let expected_state = expected_state.to_string();
    let (tx, rx) = oneshot::channel::<String>();
    let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::from_std(listener).unwrap();
        if let Ok((mut stream, _)) = listener.accept().await {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]);

            if let Some(path) = request.lines().next() {
                if let Some(query_start) = path.find('?') {
                    let query = &path[query_start + 1..path.rfind(' ').unwrap_or(path.len())];
                    let params: Vec<(String, String)> = url::form_urlencoded::parse(query.as_bytes())
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect();

                    let state = params.iter().find(|(k, _)| k == "state").map(|(_, v)| v.clone());
                    let code = params.iter().find(|(k, _)| k == "code").map(|(_, v)| v.clone());

                    if let (Some(state), Some(code)) = (state, code) {
                        if state == expected_state {
                            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Authorization successful!</h1><p>You can close this window and return to Skyscraper.</p></body></html>";
                            let _ = stream.write_all(response.as_bytes()).await;
                            if let Some(tx) = tx.lock().await.take() {
                                let _ = tx.send(code);
                            }
                        }
                    }
                }
            }
        }
    });

    let code = tokio::time::timeout(std::time::Duration::from_secs(120), rx)
        .await
        .map_err(|_| anyhow!("OAuth callback timed out"))?
        .map_err(|_| anyhow!("OAuth callback channel closed"))?;

    Ok(code)
}
