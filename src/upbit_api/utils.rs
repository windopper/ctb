use sha2::{Digest, Sha512};
use serde_json::json;
use uuid::Uuid;
use hex;
use jwt::SignWithKey;
use hmac::{self, Hmac, Mac};

use crate::env_var;

pub const UPBIT_BASE_URL: &str = "https://api.upbit.com/v1";

// 요청할 파라미터가 있는 경우 해싱된 query_hash를 추가
// 알고리즘은 SHA256
pub fn create_jwt_token(query: &Option<String>) -> Option<String> {
    let query_hash = if let Some(query) = query {
        let mut hasher = Sha512::new();
        hasher.update(query);
        let result = hasher.finalize();
        hex::encode(result)
    } else {
        "".to_string()
    };

    let payload = json!({
        "access_key": env_var("UPBIT_ACCESS_KEY"),
        "nonce": Uuid::new_v4().to_string(),
        "query_hash": query_hash,
        "query_hash_alg": "SHA512",
    });

    let secret_key = env_var("UPBIT_SECRET_KEY");
    let hmac_secret_key: Hmac<Sha512> = Hmac::<Sha512>::new_from_slice(secret_key.as_bytes()).ok()?;
    // jwt token sign
    let jwt_token = payload.sign_with_key(&hmac_secret_key).ok()?;
    Some(jwt_token)
}

pub fn create_authorization_token(query: &Option<String>) -> Option<String> {
    let jwt_token = create_jwt_token(query)?;
    let authorization_token = format!("Bearer {}", jwt_token);
    Some(authorization_token)
}

pub async fn request_upbit_api(path: &str, query: Option<String>) -> Option<String> {
    let authorization_token = create_authorization_token(&query)?;
    let client = reqwest::Client::new();
    let response = client.get(format!("{}{}{}", UPBIT_BASE_URL, path, query.unwrap_or_default()))
        .header("Authorization", authorization_token)
        .send()
        .await.ok()?;
    let body = response.text().await.ok()?;
    Some(body)
}
