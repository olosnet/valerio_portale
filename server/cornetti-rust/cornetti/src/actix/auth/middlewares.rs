/// JWT authentication middleware for actix-web.
pub mod authentication {

    use crate::auth::confs::JwtAuthConf;
    use crate::auth::models::{AuthenticationStatus, JwtDefaultClaims, JwtDefaultToken};
    use crate::auth::traits::{BaseJwtToken, SessionStore};
    use crate::core::models::CornettiHttpFilter;
    use crate::core::traits::To;
    use actix_web::body::{BoxBody, EitherBody};
    use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
    use actix_web::http::Method;
    use actix_web::{Error, HttpMessage, HttpResponse};
    use futures_util::future::LocalBoxFuture;
    use std::future::{Ready, ready};
    use std::sync::Arc;

    /// Middleware factory for JWT authentication.
    ///
    /// Validates tokens from headers (`Authorization: Bearer ...`) and/or cookies,
    /// checks CSRF tokens if configured, and optionally validates against a session store.
    ///
    /// On success, inserts `JwtDefaultClaims` into request extensions.
    ///
    /// # Cancellation
    ///
    /// The async session store check may be dropped if the request is cancelled
    /// between middleware validation and handler execution.
    pub struct JWTMiddleware<T: SessionStore + 'static> {
        auth_conf: Arc<JwtAuthConf>,
        refresh_mode: bool,
        exclude: Arc<[CornettiHttpFilter]>,
        only: Arc<[CornettiHttpFilter]>,
        store: Option<Arc<T>>,
        tenant_id: Option<String>,
    }

    impl<T: SessionStore + 'static> JWTMiddleware<T> {
        /// Creates a new JWT middleware.
        ///
        /// `refresh_mode`: when `true`, expects refresh tokens instead of access tokens.
        /// `exclude`: paths excluded from authentication.
        /// `only`: paths restricted to authentication (empty = all paths).
        /// `store`: optional session store for token validation.
        pub fn new(
            auth_conf: Arc<JwtAuthConf>,
            refresh_mode: bool,
            exclude: Arc<[CornettiHttpFilter]>,
            only: Arc<[CornettiHttpFilter]>,
            store: Option<Arc<T>>,
        ) -> Self {
            JWTMiddleware {
                auth_conf,
                refresh_mode,
                exclude,
                only,
                store,
                tenant_id: None,
            }
        }

        /// Sets the tenant identifier for multi-tenancy.
        pub fn with_tenant_id(mut self, tenant_id: String) -> Self {
            self.tenant_id = Some(tenant_id);
            self
        }
    }

    impl<S, B, T> Transform<S, ServiceRequest> for JWTMiddleware<T>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
        T: SessionStore + 'static,
    {
        type Response = ServiceResponse<EitherBody<BoxBody, B>>;
        type Error = Error;
        type Transform = JWTMiddlewareService<S, T>;
        type InitError = ();
        type Future = Ready<Result<Self::Transform, Self::InitError>>;

        fn new_transform(&self, service: S) -> Self::Future {
            ready(Ok(JWTMiddlewareService {
                service,
                auth_conf: self.auth_conf.clone(),
                refresh_mode: self.refresh_mode,
                exclude: self.exclude.clone(),
                only: self.only.clone(),
                store: self.store.clone(),
                tenant_id: self.tenant_id.clone(),
            }))
        }
    }

    pub struct JWTMiddlewareService<S, T: SessionStore + 'static> {
        service: S,
        auth_conf: Arc<JwtAuthConf>,
        refresh_mode: bool,
        exclude: Arc<[CornettiHttpFilter]>,
        only: Arc<[CornettiHttpFilter]>,
        store: Option<Arc<T>>,
        tenant_id: Option<String>,
    }

    impl<S, T: SessionStore + 'static> JWTMiddlewareService<S, T> {
        fn check_csrf(
            &self,
            req: &ServiceRequest,
            auth_conf: &JwtAuthConf,
            claims: &JwtDefaultClaims,
        ) -> bool {
            let methods = auth_conf
                .jwt_csrf_http_methods
                .iter()
                .map(|m| m.into())
                .collect::<Vec<Method>>();

            if methods.contains(req.method()) {
                if let Some(csrf_header) = req.headers().get(&auth_conf.jwt_csrf_check_header_name)
                {
                    let csrf_header = csrf_header.to_str().unwrap_or_default();

                    if csrf_header == claims.csrf.as_deref().unwrap_or_default() {
                        return true;
                    }
                }

                return false;
            }
            true
        }

        fn search_in_headers(
            &self,
            req: &ServiceRequest,
            auth_conf: &JwtAuthConf,
        ) -> (AuthenticationStatus, Option<JwtDefaultToken>) {
            let mut jwt_token: Option<JwtDefaultToken> = None;
            let mut status: AuthenticationStatus = AuthenticationStatus::Valid;

            if let Some(auth_header) = req.headers().get("Authorization") {
                let auth_header = auth_header.to_str().unwrap_or_default();

                let parts: Vec<&str> = auth_header.split_whitespace().collect::<Vec<&str>>();
                if parts.len() != 2 || parts[0] != "Bearer" {
                    status = AuthenticationStatus::InvalidAuthHeader;
                } else {
                    let token = parts[1].trim();

                    match JwtDefaultToken::decode(token, auth_conf) {
                        Err(_) => {
                            status = AuthenticationStatus::InvalidToken;
                        }
                        Ok(tkn) => {
                            if self.refresh_mode && !tkn.claims.refresh {
                                status = AuthenticationStatus::InvalidToken;
                            } else {
                                if auth_conf.jwt_csrf_cookie_enable
                                    && !self.check_csrf(req, auth_conf, &tkn.claims)
                                {
                                    status = AuthenticationStatus::InvalidCsrfToken;
                                } else {
                                    jwt_token = Some(tkn);
                                }
                            }
                        }
                    };
                }
            } else {
                status = AuthenticationStatus::MissingAuthHeader
            }

            (status, jwt_token)
        }

        fn search_in_cookies(
            &self,
            req: &ServiceRequest,
            auth_conf: &JwtAuthConf,
        ) -> (AuthenticationStatus, Option<JwtDefaultToken>) {
            let mut jwt_token: Option<JwtDefaultToken> = None;
            let mut status: AuthenticationStatus = AuthenticationStatus::Valid;

            let cookie_name: &String = if self.refresh_mode {
                &auth_conf.jwt_refresh_cookie_name
            } else {
                &auth_conf.jwt_access_cookie_name
            };

            if let Some(cookie) = req.cookie(cookie_name) {
                match JwtDefaultToken::decode(cookie.value(), auth_conf) {
                    Err(_) => {
                        status = AuthenticationStatus::InvalidToken;
                    }
                    Ok(tkn) => {
                        let is_invalid = (self.refresh_mode && !tkn.claims.refresh)
                            || (!self.refresh_mode && tkn.claims.refresh);

                        if is_invalid {
                            status = AuthenticationStatus::InvalidToken;
                        } else {
                            if auth_conf.jwt_csrf_cookie_enable
                                && !self.check_csrf(req, auth_conf, &tkn.claims)
                            {
                                status = AuthenticationStatus::InvalidCsrfToken;
                            } else {
                                jwt_token = Some(tkn);
                            }
                        }
                    }
                };
            } else {
                status = AuthenticationStatus::InvalidAuthCookie;
            }

            (status, jwt_token)
        }

        fn limits_found(&self, limits: &Arc<[CornettiHttpFilter]>, req: &ServiceRequest) -> bool {
            for e in limits.iter() {
                if e.rule_match(req.path().to(), req.method().into()) {
                    return true;
                }
            }
            false
        }
    }

    impl<S, B, T> Service<ServiceRequest> for JWTMiddlewareService<S, T>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
        T: SessionStore + 'static,
    {
        type Response = ServiceResponse<EitherBody<BoxBody, B>>;
        type Error = Error;
        type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

        forward_ready!(service);

        fn call(&self, req: ServiceRequest) -> Self::Future {
            let auth_conf = self.auth_conf.clone();
            let service = &self.service;

            let mut status: AuthenticationStatus = AuthenticationStatus::Disabled;
            let mut jwt_token: Option<JwtDefaultToken> = None;

            let mut enabled = false;
            if auth_conf.enable_auth {
                enabled = true;

                if !self.exclude.is_empty() {
                    enabled = !self.limits_found(&self.exclude, &req);
                } else if !self.only.is_empty() {
                    enabled = self.limits_found(&self.only, &req);
                }
            }

            if enabled {
                if auth_conf.jwt_search_in_cookies {
                    (status, jwt_token) = self.search_in_cookies(&req, &auth_conf);
                }

                if auth_conf.jwt_search_in_headers && status != AuthenticationStatus::Valid {
                    (status, jwt_token) = self.search_in_headers(&req, &auth_conf);
                }

                if !auth_conf.jwt_search_in_headers && !auth_conf.jwt_search_in_cookies {
                    status = AuthenticationStatus::Unauthorized;
                }
            }

            let store = self.store.clone();
            let (http_req, payload) = req.into_parts();

            if status == AuthenticationStatus::Valid {
                let claims = jwt_token.unwrap().claims;

                http_req.extensions_mut().insert(claims.clone());
                let updated_req = ServiceRequest::from_parts(http_req, payload);

                let fut = service.call(updated_req);
                let tid = self.tenant_id.as_deref().unwrap_or(crate::core::models::DEFAULT_TENANT_ID).to_string();

                Box::pin(async move {
                    let mut status = status;

                    if store.is_some() {
                        if claims.refresh {
                            status = match store
                                .as_ref()
                                .unwrap()
                                .get_refresh_token(&tid, &claims.jti)
                                .await
                            {
                                Ok(Some(_)) => AuthenticationStatus::Valid,
                                Ok(None) => AuthenticationStatus::InvalidToken,
                                Err(_) => AuthenticationStatus::StoreError,
                            };
                        } else {
                            status = match store.as_ref().unwrap().get_auth_token(&tid, &claims.jti).await
                            {
                                Ok(Some(_)) => AuthenticationStatus::Valid,
                                Ok(None) => AuthenticationStatus::InvalidToken,
                                Err(_) => AuthenticationStatus::StoreError,
                            };
                        }
                    }

                    if status != AuthenticationStatus::Valid {
                        let err: Option<crate::core::models::CornettiError> = status.err();
                        let res = fut.await?;
                        let (http_req, _) = res.into_parts();
                        let response = HttpResponse::Unauthorized().json(err).map_into_left_body();
                        return Ok(ServiceResponse::new(http_req, response));
                    }

                    let res = fut.await?;
                    Ok(res.map_into_right_body())
                })
            } else if status == AuthenticationStatus::Disabled {
                let updated_req = ServiceRequest::from_parts(http_req, payload);
                let fut = service.call(updated_req);
                Box::pin(async move {
                    let res: ServiceResponse<B> = fut.await?;
                    Ok(res.map_into_right_body())
                })
            } else {
                let err: Option<crate::core::models::CornettiError> = status.err();
                let response = HttpResponse::Unauthorized().json(err).map_into_left_body();
                Box::pin(async move { Ok(ServiceResponse::new(http_req, response)) })
            }
        }
    }
}

/// JWT authorization middleware for actix-web.
pub mod authorization {
    use crate::core::models::CornettiHttpFilter;
    use actix_web::body::{BoxBody, EitherBody};
    use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
    use actix_web::{Error, HttpMessage};
    use futures_util::future::{LocalBoxFuture, err};
    use std::future::{Ready, ready};
    use std::sync::Arc;

    /// Middleware factory for JWT authorization (permission checking).
    ///
    /// Requires `JwtDefaultClaims` to have been inserted by the authentication
    /// middleware. Checks the identity's permissions against the required set.
    pub struct JwtAuthorizationMiddleware<T: crate::auth::traits::IdentityAuthorization> {
        permissions: Arc<[String]>,
        exclude: Arc<[CornettiHttpFilter]>,
        identity_permissions: Arc<T>,
        tenant_id: Option<String>,
    }

    impl<T> JwtAuthorizationMiddleware<T>
    where
        T: crate::auth::traits::IdentityAuthorization,
    {
        /// Creates a new authorization middleware.
        ///
        /// `permissions`: required permission names.
        /// `exclude`: paths excluded from authorization checks.
        /// `identity`: permission resolver for the authenticated identity.
        pub fn new(
            permissions: Arc<[String]>,
            exclude: Arc<[CornettiHttpFilter]>,
            identity: Arc<T>,
        ) -> Self {
            Self {
                permissions,
                exclude,
                identity_permissions: identity,
                tenant_id: None,
            }
        }

        /// Sets the tenant identifier for multi-tenancy.
        pub fn with_tenant_id(mut self, tenant_id: String) -> Self {
            self.tenant_id = Some(tenant_id);
            self
        }
    }

    impl<S, B, T> Transform<S, ServiceRequest> for JwtAuthorizationMiddleware<T>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
        T: crate::auth::traits::IdentityAuthorization + 'static,
    {
        type Response = ServiceResponse<EitherBody<BoxBody, B>>;
        type Error = Error;
        type Transform = JwtAuthorizationMiddlewareService<S, T>;
        type InitError = ();
        type Future = Ready<Result<Self::Transform, Self::InitError>>;

        fn new_transform(&self, service: S) -> Self::Future {
            ready(Ok(JwtAuthorizationMiddlewareService {
                service,
                permissions: self.permissions.clone(),
                exclude: self.exclude.clone(),
                identity_permissions: self.identity_permissions.clone(),
                tenant_id: self.tenant_id.clone(),
            }))
        }
    }

    pub struct JwtAuthorizationMiddlewareService<S, T> {
        service: S,
        permissions: Arc<[String]>,
        exclude: Arc<[CornettiHttpFilter]>,
        identity_permissions: Arc<T>,
        tenant_id: Option<String>,
    }

    impl<S, B, T> Service<ServiceRequest> for JwtAuthorizationMiddlewareService<S, T>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
        T: crate::auth::traits::IdentityAuthorization + 'static,
    {
        type Response = ServiceResponse<EitherBody<BoxBody, B>>;
        type Error = Error;
        type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

        forward_ready!(service);

        fn call(&self, req: ServiceRequest) -> Self::Future {
            let permissions = self.permissions.clone();
            let exclude: Arc<[CornettiHttpFilter]> = self.exclude.clone();
            let identity_permissions = self.identity_permissions.clone();
            let method = req.method().clone();
            let path = req.path().to_string();
            let claims: Option<crate::auth::models::JwtDefaultClaims> = req
                .extensions()
                .get::<crate::auth::models::JwtDefaultClaims>()
                .cloned();
            let fut = self.service.call(req);
            let tid = self.tenant_id.clone().unwrap_or_else(|| crate::core::models::DEFAULT_TENANT_ID.to_string());

            let mut has_authorization = exclude
                .iter()
                .any(|f| f.rule_match(path.to_string(), (&method).into()));

            Box::pin(async move {
                if !has_authorization
                    && let Some(claims) = claims {
                        has_authorization = match identity_permissions
                            .get_identity_permissions(&tid, &claims.sub)
                            .await
                        {
                            Ok(user_permissions) => user_permissions
                                .iter()
                                .find(|p| permissions.contains(p.0))
                                .map(|p1| match method {
                                    actix_web::http::Method::GET => p1.1.read,
                                    actix_web::http::Method::POST => p1.1.create,
                                    actix_web::http::Method::PUT => p1.1.modify,
                                    actix_web::http::Method::DELETE => p1.1.delete,
                                    actix_web::http::Method::PATCH => p1.1.modify,
                                    _ => true,
                                })
                                .unwrap_or(false),
                            Err(_) => false,
                        };
                    }

                if has_authorization {
                    let res = fut.await?;
                    Ok(res.map_into_right_body())
                } else {
                    let corn_err = crate::core::errors::authorization::insufficient_permissions();
                    err(corn_err.into()).await
                }
            })
        }
    }
}
