use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{
        header::{self, HeaderMap},
        request::Parts,
        StatusCode,
    },
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    Extension, Form, Json, RequestPartsExt,
};

use axum_extra::{
    headers::{authorization::Bearer, Authorization, ContentType},
    TypedHeader,
};

use headers::{Cookie, HeaderMapExt};

use crate::KEYS;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use argon2::{self, Config};

pub async fn check_client(request: Request, next: Next) -> Result<impl IntoResponse, Redirect> {
    //user is logged in
    let request = check_auth(request).await;
    if request.is_ok() {
        Ok(next
            .run(request.expect("We just checked is user is logged in"))
            .await)
    } else {
        Err(Redirect::temporary("/login"))
    }
}

async fn check_auth(request: Request) -> Result<Request, Response> {
    let (mut head, body) = request.into_parts();

    if let Some(cookies) = head.headers.typed_get::<Cookie>() {
        let user_id =
            extract_user_id(cookies).map_err(|_| AuthError::MissingCredentials.into_response())?;
        head.extensions.insert(user_id);
        Ok(Request::from_parts(head, body))
    } else {
        Err(AuthError::InvalidToken.into_response())
    }
}

fn extract_user_id(cookie: Cookie) -> Result<i32, AuthError> {
    if let Some(token) = cookie.get("token") {
        let token = jsonwebtoken::decode::<Claims>(token, &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;
        Ok(token.claims.user_id)
    } else {
        Err(AuthError::InvalidToken)
    }
}


pub async fn authorize(Json(payload): Json<AuthPayload>) -> impl IntoResponse {
    //println!("{:?}", payload);
    // Check if the user sent the credentials
    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return Err(AuthError::MissingCredentials);
    }
    // Here you can check the user credentials from a database
    if payload.client_id != "foo" || payload.client_secret != "bar" {
        return Err(AuthError::WrongCredentials);
    }
    let claims = Claims {
        sub: "b@b.com".to_owned(),
        user_id: 1,
        exp: 2000000000, // May 2033
    };
    // Create the authorization token
    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    println!("ok {:?}", payload);

    let html = leptos::ssr::render_to_string(move |cx| {

    })
    let cookie_str = format!("token={}; HttpOnly; SameSite=Strict", token);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .header(header::SET_COOKIE, cookie_str)
        .body(html.0)
        .unwrap();
    Ok(response)
}

pub fn hash_password(pasword: String) -> String {
    let config = Config::default();
    let hash = argon2::hash_encoded(pasword.as_bytes(), b"randomsalt", &config).unwrap();
    hash
}


impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String, //subject
    exp: usize,  //expiry
    user_id: i32,
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}
