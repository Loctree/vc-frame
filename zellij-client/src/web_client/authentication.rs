use crate::web_client::utils::parse_cookies;
use axum::body::Body;
use axum::http::header::SET_COOKIE;
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use zellij_utils::web_authentication_tokens::{
    hash_token, is_session_token_read_only, validate_session_token,
};

#[derive(Clone)]
pub struct SessionTokenHash(pub String);

#[derive(Clone, Copy)]
pub struct IsReadOnly(pub bool);

pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let cookies = parse_cookies(&request);

    let session_token = match cookies.get("session_token") {
        Some(token) => token.clone(),
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match validate_session_token(&session_token) {
        Ok(true) => {
            // Check if this is a read-only token
            let is_read_only = is_session_token_read_only(&session_token).unwrap_or(true);

            // Compute session token hash for client ownership verification
            let session_token_hash = hash_token(&session_token);

            // Store in request extensions for downstream handlers
            let mut request = request;
            request.extensions_mut().insert(IsReadOnly(is_read_only));
            request
                .extensions_mut()
                .insert(SessionTokenHash(session_token_hash));

            let response = next.run(request).await;
            Ok(response)
        },
        Ok(false) | Err(_) => {
            // revoke session_token as if it exists it's no longer valid
            let mut response = Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap();

            // The session cookie is always issued as `Secure` (see `login_handler`), so a
            // single `Secure` tombstone expires it. A cookie is keyed by name, domain and path,
            // so this also expires a non-`Secure` `session_token` left by an older build.
            let clear_cookie = Cookie::build(("session_token", ""))
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .path("/")
                .max_age(time::Duration::seconds(0))
                .build();

            response
                .headers_mut()
                .append(SET_COOKIE, clear_cookie.to_string().parse().unwrap());

            Ok(response)
        },
    }
}
