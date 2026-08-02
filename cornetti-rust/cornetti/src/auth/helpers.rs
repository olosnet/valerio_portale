/// Utoipa helpers for JWT authentication security schemes.
pub mod utoipa {

    use std::sync::Arc;

    use utoipa::openapi::{
        Components,
        security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    };

    use crate::core::{
        helpers::{
            common::apply_api_prefix,
            utoipa::{BaseApiDoc, auto_context_path, auto_operation_id},
        },
        models::{CornettiHttpFilter, CornettiHttpMethod},
    };

    /// Registers JWT security scheme definitions in the OpenAPI components
    /// based on the authentication configuration.
    ///
    /// Adds cookie-based and header-based schemes depending on what is enabled.
    pub fn get_jwt_auth_security_schemes(
        components: &mut Components,
        auth_conf: &crate::auth::confs::JwtAuthConf,
    ) {
        if auth_conf.enable_auth {
            if auth_conf.jwt_search_in_cookies {
                if auth_conf.refresh_cookie.enable {
                    components.add_security_scheme(
                        "JWTCookieRefresh",
                        SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(
                            &auth_conf.refresh_cookie.name,
                        ))),
                    );
                }

                components.add_security_scheme(
                    "JWTCookieAuth",
                    SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(
                        &auth_conf.access_cookie.name,
                    ))),
                );
            }

            if auth_conf.jwt_search_in_headers {
                if auth_conf.refresh_cookie.enable {
                    components.add_security_scheme(
                        "JWTBearerRefresh",
                        SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(
                            &auth_conf.refresh_cookie.name,
                        ))),
                    );
                }

                components.add_security_scheme(
                    "JWTBearerAuth",
                    SecurityScheme::Http(
                        HttpBuilder::new()
                            .scheme(HttpAuthScheme::Bearer)
                            .bearer_format("JWT")
                            .build(),
                    ),
                )
            }

            if auth_conf.jwt_csrf_cookie_enable {
                components.add_security_scheme(
                    "JWTCsrfCookie",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new(
                        &auth_conf.jwt_csrf_check_header_name,
                    ))),
                );
            }
        }
    }

    /// Applies JWT security requirements to all operations in the OpenAPI document,
    /// respecting exclude/only filters.
    pub fn methods_security_schemes(
        doc: &mut utoipa::openapi::OpenApi,
        auth_conf: &crate::auth::confs::JwtAuthConf,
        exclude: Arc<[CornettiHttpFilter]>,
        only: Arc<[CornettiHttpFilter]>,
    ) {
        if auth_conf.enable_auth {
            let mut sec_requirements: Vec<utoipa::openapi::SecurityRequirement> =
                if auth_conf.jwt_search_in_cookies {
                    vec![utoipa::openapi::SecurityRequirement::new(
                        "JWTCookieAuth",
                        ["/*"],
                    )]
                } else {
                    vec![]
                };

            if auth_conf.jwt_search_in_headers {
                sec_requirements.push(utoipa::openapi::SecurityRequirement::new(
                    "JWTBearerAuth",
                    ["/*"],
                ));
            }

            let csrf_sec_requirement = if auth_conf.jwt_csrf_cookie_enable {
                Some(utoipa::openapi::SecurityRequirement::new(
                    "JWTCsrfCookie",
                    ["/*"],
                ))
            } else {
                None
            };

            write_security_schemes(
                doc,
                auth_conf,
                exclude,
                only,
                sec_requirements,
                csrf_sec_requirement,
            );
        }
    }

    /// Applies JWT refresh-token security requirements to all operations.
    pub fn refresh_methods_security_schemes(
        doc: &mut utoipa::openapi::OpenApi,
        auth_conf: &crate::auth::confs::JwtAuthConf,
        exclude: Arc<[CornettiHttpFilter]>,
        only: Arc<[CornettiHttpFilter]>,
    ) {
        if auth_conf.enable_auth {
            let mut sec_requirements: Vec<utoipa::openapi::SecurityRequirement> =
                if auth_conf.jwt_search_in_cookies {
                    vec![utoipa::openapi::SecurityRequirement::new(
                        "JWTCookieRefresh",
                        ["/*"],
                    )]
                } else {
                    vec![]
                };

            if auth_conf.jwt_search_in_headers {
                sec_requirements.push(utoipa::openapi::SecurityRequirement::new(
                    "JWTBearerRefresh",
                    ["/*"],
                ));
            }

            let csrf_sec_requirement = if auth_conf.jwt_csrf_cookie_enable {
                Some(utoipa::openapi::SecurityRequirement::new(
                    "JWTCsrfCookie",
                    ["/*"],
                ))
            } else {
                None
            };

            write_security_schemes(
                doc,
                auth_conf,
                exclude,
                only,
                sec_requirements,
                csrf_sec_requirement,
            );
        }
    }

    fn write_security_schemes(
        doc: &mut utoipa::openapi::OpenApi,
        auth_conf: &crate::auth::confs::JwtAuthConf,
        exclude: Arc<[CornettiHttpFilter]>,
        only: Arc<[CornettiHttpFilter]>,
        sec_requirements: Vec<utoipa::openapi::SecurityRequirement>,
        csrf_sec_requirement: Option<utoipa::openapi::SecurityRequirement>,
    ) {
        let default = only.is_empty();

        doc.paths.paths.iter_mut().for_each(|(url, path)| {
            let exclude_rule = if !exclude.is_empty() {
                exclude.iter().find(|f| f.path_match(url.to_string()))
            } else {
                None
            };

            let only_rule = if !only.is_empty() && exclude_rule.is_none() {
                only.iter().find(|f| f.path_match(url.to_string()))
            } else {
                None
            };

            let apply_to_op = |op: &mut utoipa::openapi::path::Operation, method: &CornettiHttpMethod| {
                let insert = if let Some(rule) = &exclude_rule {
                    !rule.method_match(method.clone())
                } else if let Some(rule) = &only_rule {
                    rule.method_match(method.clone())
                } else {
                    default
                };

                if insert {
                    op.security
                        .get_or_insert_with(Vec::new)
                        .extend(sec_requirements.clone());

                    if auth_conf
                        .jwt_csrf_http_methods
                        .contains(method)
                        && let Some(csrf_sec) = csrf_sec_requirement.as_ref()
                    {
                        op.security
                            .get_or_insert_with(Vec::new)
                            .push(csrf_sec.clone());
                    }
                }
            };

            if let Some(op) = path.get.as_mut() {
                apply_to_op(op, &CornettiHttpMethod::GET);
            }
            if let Some(op) = path.post.as_mut() {
                apply_to_op(op, &CornettiHttpMethod::POST);
            }
            if let Some(op) = path.put.as_mut() {
                apply_to_op(op, &CornettiHttpMethod::PUT);
            }
            if let Some(op) = path.delete.as_mut() {
                apply_to_op(op, &CornettiHttpMethod::DELETE);
            }
            if let Some(op) = path.patch.as_mut() {
                apply_to_op(op, &CornettiHttpMethod::PATCH);
            }
            if let Some(op) = path.options.as_mut() {
                apply_to_op(op, &CornettiHttpMethod::OPTIONS);
            }
            if let Some(op) = path.head.as_mut() {
                apply_to_op(op, &CornettiHttpMethod::HEAD);
            }
        });
    }

    /// OpenAPI doc entry for auth-protected modules.
    pub struct AuthApiDocEntry<'a> {
        /// Module name for operation IDs.
        pub module_name: String,
        /// Context path for the module.
        pub context_path: String,
        /// Base application configuration.
        pub base_conf: &'a crate::core::confs::BaseConf,
        /// JWT authentication configuration.
        pub auth_conf: &'a crate::auth::confs::JwtAuthConf,
        /// Filters for paths to exclude from security requirements.
        pub security_schemes_exclude: Arc<[CornettiHttpFilter]>,
        /// Filters for paths to restrict security requirements to.
        pub security_schemes_only: Arc<[CornettiHttpFilter]>,
    }

    impl BaseApiDoc for AuthApiDocEntry<'_> {
        fn api_doc<T: utoipa::OpenApi>(&self) -> utoipa::openapi::OpenApi {
            let mut doc: utoipa::openapi::OpenApi = T::openapi();

            auto_operation_id(&mut doc, &self.module_name);
            auto_context_path(
                &mut doc,
                &apply_api_prefix(&self.base_conf.api_prefix, &self.context_path),
            );

            if self.auth_conf.enable_auth {
                methods_security_schemes(
                    &mut doc,
                    self.auth_conf,
                    self.security_schemes_exclude.clone(),
                    self.security_schemes_only.clone(),
                );
            }

            doc
        }
    }

    /// OpenAPI doc entry for modules that support both authentication and token refresh.
    pub struct AuthRefreshApiDocEntry<'a> {
        /// Module name for operation IDs.
        pub module_name: String,
        /// Context path for the module.
        pub context_path: String,
        /// Base application configuration.
        pub base_conf: &'a crate::core::confs::BaseConf,
        /// JWT authentication configuration.
        pub auth_conf: &'a crate::auth::confs::JwtAuthConf,
        /// Security scheme exclude filters.
        pub security_schemes_exclude: Arc<[CornettiHttpFilter]>,
        /// Security scheme only filters.
        pub security_schemes_only: Arc<[CornettiHttpFilter]>,
        /// Refresh security scheme exclude filters.
        pub refresh_security_schemes_exclude: Arc<[CornettiHttpFilter]>,
        /// Refresh security scheme only filters.
        pub refresh_security_schemes_only: Arc<[CornettiHttpFilter]>,
    }

    impl BaseApiDoc for AuthRefreshApiDocEntry<'_> {
        fn api_doc<T: utoipa::OpenApi>(&self) -> utoipa::openapi::OpenApi {
            let mut doc: utoipa::openapi::OpenApi = T::openapi();

            auto_operation_id(&mut doc, &self.module_name);
            auto_context_path(
                &mut doc,
                &apply_api_prefix(&self.base_conf.api_prefix, &self.context_path),
            );

            if self.auth_conf.enable_auth {
                methods_security_schemes(
                    &mut doc,
                    self.auth_conf,
                    self.security_schemes_exclude.clone(),
                    self.security_schemes_only.clone(),
                );

                refresh_methods_security_schemes(
                    &mut doc,
                    self.auth_conf,
                    self.refresh_security_schemes_exclude.clone(),
                    self.refresh_security_schemes_only.clone(),
                );
            }

            doc
        }
    }
}
