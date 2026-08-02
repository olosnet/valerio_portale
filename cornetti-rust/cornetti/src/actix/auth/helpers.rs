use std::sync::Arc;

use crate::auth::confs::JwtAuthConf;
use crate::auth::models::{
    DefaultLoginResponse, JwtDefaultClaims, JwtDefaultToken, RefreshAuthResponse, SessionStoreData,
};
use crate::auth::traits::{BaseJwtToken, SessionStore};
use crate::errors;
use crate::core::models::CornettiResult;
use actix_web::{HttpRequest, cookie::Cookie};

/// Generates a JWT access token and optional cookies.
///
/// # Errors
///
/// Returns a 500 error if token encoding fails.
async fn generate_access_token(
    conf: &JwtAuthConf,
    identity: String,
    session_id: String,
) -> CornettiResult<(
    JwtDefaultToken,
    String,
    Option<Cookie<'_>>,
    Option<Cookie<'_>>,
)> {
    let access_token: JwtDefaultToken =
        JwtDefaultToken::new(conf.clone(), identity, session_id, false);

    let access_token_encoded: String = access_token
        .encode(conf)
        .map_err(|e| errors::auth_errors::jwt_encode_error().with_internal_detail(e.to_string()))?;

    let access_cookie: Option<Cookie> = if conf.jwt_search_in_cookies {
        Some(
            Cookie::build(&conf.access_cookie.name, access_token_encoded.clone())
                .path(&conf.access_cookie.path)
                .http_only(true)
                .secure(conf.access_cookie.secure)
                .same_site((&conf.access_cookie.same_site).into())
                .finish(),
        )
    } else {
        None
    };

    let csrf_access_cookie: Option<Cookie> = if conf.jwt_csrf_cookie_enable {
        Some(
            Cookie::build(
                &conf.csrf_access_cookie.name,
                access_token.claims.csrf.clone().unwrap(),
            )
            .path(&conf.csrf_access_cookie.path)
            .http_only(false)
            .secure(false)
            .same_site((&conf.jwt_csrf_cookie_same_site).into())
            .finish(),
        )
    } else {
        None
    };

    Ok((
        access_token,
        access_token_encoded,
        access_cookie,
        csrf_access_cookie,
    ))
}

/// Adds access and refresh tokens to the session store.
///
/// # Errors
///
/// Returns a `CornettiError` if the store operation fails.
async fn add_to_store<S: SessionStore>(
    tenant_id: &str,
    access_token: &JwtDefaultToken,
    refresh_token: Option<&JwtDefaultToken>,
    req: HttpRequest,
    session_store: Option<Arc<S>>,
) -> CornettiResult<()> {
    if let Some(store) = session_store {
        let ip_addr: String = req
            .peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_default();

        let user_agent = req
            .headers()
            .get("User-Agent")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if let Some(refresh_token) = refresh_token {
            let refresh_session_store_data = SessionStoreData::new(
                refresh_token.claims.clone(),
                ip_addr.clone(),
                user_agent.clone(),
            );

            store
                .add_token(tenant_id, &refresh_session_store_data)
                .await?;
        }

        let access_session_store_data =
            SessionStoreData::new(access_token.claims.clone(), ip_addr, user_agent);

        store
            .add_token(tenant_id, &access_session_store_data)
            .await?;
    }

    Ok(())
}

/// Generates a complete authentication response for a user login.
///
/// Returns the login response DTO and optional cookies (access, refresh, CSRF access,
/// CSRF refresh).
///
/// # Errors
///
/// Returns a `CornettiError` if token generation or session storage fails.
pub async fn generate_auth_tokens_and_response<'a, T, S: SessionStore>(
    conf: &'a JwtAuthConf,
    user: T,
    identity: String,
    tenant_id: &'a str,
    req: HttpRequest,
    session_store: Option<Arc<S>>,
) -> CornettiResult<(
    DefaultLoginResponse<T>,
    Option<Cookie<'a>>,
    Option<Cookie<'a>>,
    Option<Cookie<'a>>,
    Option<Cookie<'a>>,
)> {
    let session_id = uuid::Uuid::new_v4().to_string();

    let (access_token, access_token_encoded, access_cookie, csrf_access_cookie) =
        generate_access_token(conf, identity.clone(), session_id.clone()).await?;

    let refresh_token: Option<JwtDefaultToken> = if conf.refresh_cookie.enable {
        Some(JwtDefaultToken::new(
            conf.clone(),
            identity,
            session_id,
            true,
        ))
    } else {
        None
    };

    let refresh_token_encoded: Option<String> = if let Some(refresh_token) = refresh_token.as_ref()
    {
        Some(
            refresh_token
                .encode(conf)
                .map_err(|e| errors::auth_errors::jwt_encode_error().with_internal_detail(e.to_string()))?,
        )
    } else {
        None
    };

    let refresh_expires_result: Option<usize> = if conf.refresh_cookie.enable {
        Some(conf.refresh_cookie.expire_minutes * 60)
    } else {
        None
    };

    let refresh_cookie: Option<Cookie> =
        if conf.jwt_search_in_cookies && refresh_token_encoded.is_some() {
            Some(
                Cookie::build(
                    &conf.refresh_cookie.name,
                    refresh_token_encoded.clone().unwrap(),
                )
                .path(&conf.refresh_cookie.path)
                .http_only(true)
                .secure(conf.refresh_cookie.secure)
                .same_site((&conf.refresh_cookie.same_site).into())
                .finish(),
            )
        } else {
            None
        };

    let csrf_refresh_cookie: Option<Cookie> =
        if let (true, Some(token)) = (conf.jwt_csrf_cookie_enable, refresh_token.as_ref()) {
            Some(
                Cookie::build(
                    &conf.csrf_refresh_cookie.name,
                    token.claims.csrf.clone().unwrap(),
                )
                .path(&conf.csrf_refresh_cookie.path)
                .http_only(false)
                .secure(false)
                .same_site((&conf.jwt_csrf_cookie_same_site).into())
                .finish(),
            )
        } else {
            None
        };

    add_to_store(
        tenant_id,
        &access_token,
        refresh_token.as_ref(),
        req,
        session_store,
    )
    .await?;

    let result = (
        DefaultLoginResponse {
            access_token: if conf.jwt_search_in_headers {
                Some(access_token_encoded)
            } else {
                None
            },
            refresh_token: if conf.jwt_search_in_headers {
                refresh_token_encoded
            } else {
                None
            },
            expires_in: if conf.jwt_search_in_headers {
                Some(access_token.claims.exp)
            } else {
                None
            },
            refresh_expires_in: if conf.jwt_search_in_headers {
                refresh_expires_result
            } else {
                None
            },
            identity: user,
        },
        access_cookie,
        refresh_cookie,
        csrf_access_cookie,
        csrf_refresh_cookie,
    );

    Ok(result)
}

/// Refreshes authentication tokens and generates a response for token refresh requests.
///
/// # Errors
///
/// Returns a `CornettiError` if token generation or session storage fails.
pub async fn refresh_auth_tokens_and_response<'a, T, S: SessionStore>(
    conf: &'a JwtAuthConf,
    user: T,
    claims: JwtDefaultClaims,
    tenant_id: &'a str,
    req: HttpRequest,
    session_store: Option<Arc<S>>,
) -> CornettiResult<(
    RefreshAuthResponse<T>,
    Option<Cookie<'a>>,
    Option<Cookie<'a>>,
)> {
    let identity = claims.sub;
    let session_id = claims.session_id;

    let (access_token, access_token_encoded, access_cookie, csrf_access_cookie) =
        generate_access_token(conf, identity, session_id).await?;

    add_to_store(tenant_id, &access_token, None, req, session_store).await?;

    let result = (
        RefreshAuthResponse {
            access_token: if conf.jwt_search_in_headers {
                Some(access_token_encoded)
            } else {
                None
            },
            expires_in: if conf.jwt_search_in_headers {
                Some(access_token.claims.exp)
            } else {
                None
            },
            identity: user,
        },
        access_cookie,
        csrf_access_cookie,
    );

    Ok(result)
}

/// Invalidates a user session, removing tokens from the store and returning
/// the names of cookies that should be cleared.
///
/// # Errors
///
/// Returns a `CornettiError` if session removal from the store fails.
pub async fn invalidate_session<'a, S: SessionStore>(
    conf: &'a JwtAuthConf,
    session_store: Option<Arc<S>>,
    identity: String,
    session_id: String,
    tenant_id: &str,
) -> CornettiResult<Vec<&'a str>> {
    if let Some(store) = session_store {
        store
            .remove_session(tenant_id, &identity, &session_id)
            .await?;
    }
    let mut cookie_rem: Vec<&str> = Vec::new();

    if conf.jwt_search_in_cookies {
        cookie_rem.push(&conf.access_cookie.name);
        cookie_rem.push(&conf.refresh_cookie.name);
    }

    if conf.jwt_csrf_cookie_enable {
        cookie_rem.push(&conf.csrf_access_cookie.name);
        cookie_rem.push(&conf.csrf_refresh_cookie.name);
    }

    Ok(cookie_rem)
}
