use serde::{Deserialize, Serialize};

// Documentation for PKCE flow:
// https://www.oauth.com/oauth2-servers/pkce/
//
// Tutorial:
// https://developer.okta.com/blog/2019/08/22/okta-authjs-pkce/
//
// 1. Generate "code verifier", a cryptographically random string using the
//    characters A-Z, a-z, 0-9, and the punctuation characters -._~ (hyphen,
//    period, underscore, and tilde), between 43 and 128 characters long.
// 2. Create "code challenge" = base66_url_encode(sha256(code_verifier))
// 3.

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationCodeResponseType {
    Code,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum AuthorizationCodeChallengeMethod {
    Plain,
    #[serde(rename = "S256")]
    Sha256,
}

#[derive(Deserialize, Serialize)]
pub struct AuthorizationCodeRequest {
    pub response_type: AuthorizationCodeResponseType,
    pub client_id: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: AuthorizationCodeChallengeMethod,
    pub redirect_uri: String,
}

// #[derive(Deserialize, Serialize)]
// pub struct AuthorizationCodeError {
//     // TODO
//     //error=invalid_request and the error_description or error_uri
// };

#[derive(Deserialize, Serialize)]
pub struct AuthorizationCode {
    pub code: String,
    pub state: String,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessTokenGrantType {
    AuthorizationCode,
}

#[derive(Deserialize, Serialize)]
pub struct AccessTokenRequest {
    pub grant_type: AccessTokenGrantType,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub code_verifier: String,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessTokenType {
    Bearer,
}

#[derive(Deserialize, Serialize)]
pub struct AccessToken {
    pub token: String,
    pub token_type: AccessTokenType,
}
