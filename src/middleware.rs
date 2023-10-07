use axum::{
    extract::{FromRequest, Request},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

/*

async fn jwt_auth_middleware(req: Request, next: Next) -> Result<impl IntoResponse, Response> {
    let req = req.

    Ok()
}

async fn authenticate_request(req: Request) -> Result<Request, Response> {
    Ok()
}

fn extract_token_from_header(req: &Request) -> Result<String, > {

}
*/

//represents the actual JWT
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, //UID user id
    user_id: i32,
    exp: usize, // Expiry
}

#[derive(Debug, Serialize)]
struct AuthBody {
    access_token: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct AuthPayload {
    client_id: String,
    client_secret: String,
}

#[derive(Debug)]
enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}
