/// API key authentication middleware for actix-web.
pub mod authentication {
    use std::{
        future::{Ready, ready},
        rc::Rc,
        sync::Arc,
    };

    use actix_web::{
        Error, HttpMessage, HttpResponse,
        body::{BoxBody, EitherBody},
        dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
        http::StatusCode,
    };
    use futures_util::future::LocalBoxFuture;

    use crate::{
        auth_apikey::{confs::ApiKeyAuthConf, services::AuthApiKeyAuthService},
        core::{
            errors,
            models::{CornettiHttpFilter, CornettiResult},
            traits::To,
        },
    };

    /// Middleware factory for API key authentication.
    ///
    /// Reads the API key from a configurable HTTP header, validates it via
    /// `AuthApiKeyAuthService`, and inserts `AuthApiKey` into request extensions.
    pub struct ApiKeyMiddleware {
        auth_conf: Arc<ApiKeyAuthConf>,
        exclude: Arc<[CornettiHttpFilter]>,
        only: Arc<[CornettiHttpFilter]>,
        auth_service: Arc<AuthApiKeyAuthService>,
    }

    impl ApiKeyMiddleware {
        /// Creates a new API key middleware.
        ///
        /// `exclude`: paths excluded from authentication.
        /// `only`: paths restricted to authentication (empty = all paths).
        pub fn new(
            auth_conf: Arc<ApiKeyAuthConf>,
            exclude: Arc<[CornettiHttpFilter]>,
            only: Arc<[CornettiHttpFilter]>,
            auth_service: Arc<AuthApiKeyAuthService>,
        ) -> Self {
            Self {
                auth_conf,
                exclude,
                only,
                auth_service,
            }
        }
    }

    impl<S, B> Transform<S, ServiceRequest> for ApiKeyMiddleware
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<EitherBody<BoxBody, B>>;
        type Error = Error;
        type Transform = ApiKeyMiddlewareService<S>;
        type InitError = ();
        type Future = Ready<Result<Self::Transform, Self::InitError>>;

        fn new_transform(&self, service: S) -> Self::Future {
            ready(Ok(ApiKeyMiddlewareService {
                service: Rc::new(service),
                auth_conf: self.auth_conf.clone(),
                exclude: self.exclude.clone(),
                only: self.only.clone(),
                auth_service: self.auth_service.clone(),
            }))
        }
    }

    pub struct ApiKeyMiddlewareService<S> {
        service: Rc<S>,
        auth_conf: Arc<ApiKeyAuthConf>,
        exclude: Arc<[CornettiHttpFilter]>,
        only: Arc<[CornettiHttpFilter]>,
        auth_service: Arc<AuthApiKeyAuthService>,
    }

    impl<S> ApiKeyMiddlewareService<S> {
        fn limits_found(&self, limits: &Arc<[CornettiHttpFilter]>, req: &ServiceRequest) -> bool {
            for limit in limits.iter() {
                if limit.rule_match(req.path().to(), req.method().into()) {
                    return true;
                }
            }

            false
        }

        fn read_api_key(
            &self,
            req: &ServiceRequest,
            auth_conf: &ApiKeyAuthConf,
        ) -> CornettiResult<String> {
            let header_name = auth_conf.header_name.as_str();

            let header_value = req.headers().get(header_name).ok_or_else(|| {
                errors::authentication::custom_auth_error().with_internal_detail(format!(
                    "Missing {} header",
                    header_name
                ))
            })?;

            let header_value = header_value.to_str().map_err(|_| {
                errors::authentication::custom_auth_error().with_internal_detail(format!(
                    "Invalid {} header",
                    header_name
                ))
            })?;

            if header_value.trim().is_empty() {
                return Err(errors::authentication::custom_auth_error().with_internal_detail(format!(
                    "Invalid {} header",
                    header_name
                )));
            }

            Ok(header_value.to_string())
        }
    }

    impl<S, B> Service<ServiceRequest> for ApiKeyMiddlewareService<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<EitherBody<BoxBody, B>>;
        type Error = Error;
        type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

        forward_ready!(service);

        fn call(&self, req: ServiceRequest) -> Self::Future {
            let mut enabled = false;

            if self.auth_conf.enable_auth {
                enabled = true;

                if !self.exclude.is_empty() {
                    enabled = !self.limits_found(&self.exclude, &req);
                } else if !self.only.is_empty() {
                    enabled = self.limits_found(&self.only, &req);
                }
            }

            if !enabled {
                let service = self.service.clone();

                return Box::pin(async move {
                    let response = service.call(req).await?;
                    Ok(response.map_into_right_body())
                });
            }

            let api_key_header = self.read_api_key(&req, &self.auth_conf);
            let (http_req, payload) = req.into_parts();

            let api_key_header = match api_key_header {
                Ok(api_key_header) => api_key_header,
                Err(err) => {
                    let status = StatusCode::from_u16(err.status)
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                    let response = HttpResponse::build(status).json(err).map_into_left_body();

                    return Box::pin(async move { Ok(ServiceResponse::new(http_req, response)) });
                }
            };

            let service = self.service.clone();
            let auth_service = self.auth_service.clone();

            Box::pin(async move {
                match auth_service.authenticate(&api_key_header).await {
                    Ok(api_key) => {
                        http_req.extensions_mut().insert(api_key);
                        let updated_req = ServiceRequest::from_parts(http_req, payload);
                        let response = service.call(updated_req).await?;
                        Ok(response.map_into_right_body())
                    }
                    Err(err) => {
                        let status = StatusCode::from_u16(err.status)
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                        let response = HttpResponse::build(status).json(err).map_into_left_body();
                        Ok(ServiceResponse::new(http_req, response))
                    }
                }
            })
        }
    }
}
