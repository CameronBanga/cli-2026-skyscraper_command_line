use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use p256::ecdsa::{signature::Signer, SigningKey};
use serde_json::json;
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct DpopKeyPair {
    signing_key: SigningKey,
    jwk_thumbprint: String,
    public_jwk: serde_json::Value,
}

impl DpopKeyPair {
    pub fn generate() -> Result<Self> {
        let signing_key = SigningKey::random(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key();
        let point = verifying_key.to_encoded_point(false);

        let x = URL_SAFE_NO_PAD.encode(point.x().ok_or_else(|| anyhow::anyhow!("no x"))?);
        let y = URL_SAFE_NO_PAD.encode(point.y().ok_or_else(|| anyhow::anyhow!("no y"))?);

        let public_jwk = json!({
            "kty": "EC",
            "crv": "P-256",
            "x": x,
            "y": y,
        });

        let thumbprint_input = format!(
            r#"{{"crv":"P-256","kty":"EC","x":"{}","y":"{}"}}"#,
            x, y
        );
        let thumbprint_hash = Sha256::digest(thumbprint_input.as_bytes());
        let jwk_thumbprint = URL_SAFE_NO_PAD.encode(thumbprint_hash);

        Ok(DpopKeyPair {
            signing_key,
            jwk_thumbprint,
            public_jwk,
        })
    }

    pub fn create_proof(
        &self,
        htm: &str,
        htu: &str,
        nonce: Option<&str>,
        access_token: Option<&str>,
    ) -> Result<String> {
        let header = json!({
            "typ": "dpop+jwt",
            "alg": "ES256",
            "jwk": self.public_jwk,
        });

        let now = chrono::Utc::now().timestamp();
        let jti = uuid::Uuid::new_v4().to_string();

        let mut payload = json!({
            "jti": jti,
            "htm": htm,
            "htu": htu,
            "iat": now,
        });

        if let Some(nonce) = nonce {
            payload["nonce"] = json!(nonce);
        }

        if let Some(token) = access_token {
            let ath = Sha256::digest(token.as_bytes());
            payload["ath"] = json!(URL_SAFE_NO_PAD.encode(ath));
        }

        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header)?);
        let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload)?);
        let signing_input = format!("{}.{}", header_b64, payload_b64);

        let signature: p256::ecdsa::Signature = self.signing_key.sign(signing_input.as_bytes());
        let sig_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

        Ok(format!("{}.{}.{}", header_b64, payload_b64, sig_b64))
    }

    pub fn jwk_thumbprint(&self) -> &str {
        &self.jwk_thumbprint
    }

    pub fn public_jwk(&self) -> &serde_json::Value {
        &self.public_jwk
    }
}
