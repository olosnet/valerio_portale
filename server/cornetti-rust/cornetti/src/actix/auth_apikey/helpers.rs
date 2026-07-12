/// Utoipa helpers for API key authentication security schemes.
pub mod utoipa {
    use std::sync::Arc;

    use utoipa::openapi::{
        Components,
        security::{ApiKey, ApiKeyValue, SecurityScheme},
    };

    use crate::{
        auth_apikey::confs::ApiKeyAuthConf,
        core::models::{CornettiHttpFilter, CornettiHttpMethod},
    };

    /// Registers the API key security scheme in the OpenAPI components.
    pub fn get_apikey_auth_security_schemes(
        components: &mut Components,
        auth_conf: &ApiKeyAuthConf,
    ) {
        if auth_conf.enable_auth {
            components.add_security_scheme(
                "ApiKeyAuth",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new(&auth_conf.header_name))),
            );
        }
    }

    /// Applies API key security requirements to all operations in the OpenAPI document,
    /// respecting exclude/only filters.
    pub fn methods_security_schemes(
        doc: &mut utoipa::openapi::OpenApi,
        auth_conf: &ApiKeyAuthConf,
        exclude: Arc<[CornettiHttpFilter]>,
        only: Arc<[CornettiHttpFilter]>,
    ) {
        if auth_conf.enable_auth {
            write_security_schemes(
                doc,
                exclude,
                only,
                vec![utoipa::openapi::SecurityRequirement::new(
                    "ApiKeyAuth",
                    ["/*"],
                )],
            );
        }
    }

    fn write_security_schemes(
        doc: &mut utoipa::openapi::OpenApi,
        exclude: Arc<[CornettiHttpFilter]>,
        only: Arc<[CornettiHttpFilter]>,
        sec_requirements: Vec<utoipa::openapi::SecurityRequirement>,
    ) {
        let default = only.is_empty();

        doc.paths.paths.iter_mut().for_each(|(url, path)| {
            let exclude_rule = if !exclude.is_empty() {
                exclude.iter().find(|rule| rule.path_match(url.to_string()))
            } else {
                None
            };

            let only_rule = if !only.is_empty() && exclude_rule.is_none() {
                only.iter().find(|rule| rule.path_match(url.to_string()))
            } else {
                None
            };

            path.get.as_mut().map(|op| {
                let insert = if let Some(rule) = exclude_rule {
                    !rule.method_match(CornettiHttpMethod::GET)
                } else if let Some(rule) = only_rule {
                    rule.method_match(CornettiHttpMethod::GET)
                } else {
                    default
                };

                if insert {
                    op.security
                        .get_or_insert_with(Vec::new)
                        .extend(sec_requirements.clone());
                }
            });

            path.post.as_mut().map(|op| {
                let insert = if let Some(rule) = exclude_rule {
                    !rule.method_match(CornettiHttpMethod::POST)
                } else if let Some(rule) = only_rule {
                    rule.method_match(CornettiHttpMethod::POST)
                } else {
                    default
                };

                if insert {
                    op.security
                        .get_or_insert_with(Vec::new)
                        .extend(sec_requirements.clone());
                }
            });

            path.put.as_mut().map(|op| {
                let insert = if let Some(rule) = exclude_rule {
                    !rule.method_match(CornettiHttpMethod::PUT)
                } else if let Some(rule) = only_rule {
                    rule.method_match(CornettiHttpMethod::PUT)
                } else {
                    default
                };

                if insert {
                    op.security
                        .get_or_insert_with(Vec::new)
                        .extend(sec_requirements.clone());
                }
            });

            path.patch.as_mut().map(|op| {
                let insert = if let Some(rule) = exclude_rule {
                    !rule.method_match(CornettiHttpMethod::PATCH)
                } else if let Some(rule) = only_rule {
                    rule.method_match(CornettiHttpMethod::PATCH)
                } else {
                    default
                };

                if insert {
                    op.security
                        .get_or_insert_with(Vec::new)
                        .extend(sec_requirements.clone());
                }
            });

            path.delete.as_mut().map(|op| {
                let insert = if let Some(rule) = exclude_rule {
                    !rule.method_match(CornettiHttpMethod::DELETE)
                } else if let Some(rule) = only_rule {
                    rule.method_match(CornettiHttpMethod::DELETE)
                } else {
                    default
                };

                if insert {
                    op.security
                        .get_or_insert_with(Vec::new)
                        .extend(sec_requirements.clone());
                }
            });

            path.head.as_mut().map(|op| {
                let insert = if let Some(rule) = exclude_rule {
                    !rule.method_match(CornettiHttpMethod::HEAD)
                } else if let Some(rule) = only_rule {
                    rule.method_match(CornettiHttpMethod::HEAD)
                } else {
                    default
                };

                if insert {
                    op.security
                        .get_or_insert_with(Vec::new)
                        .extend(sec_requirements.clone());
                }
            });

            path.options.as_mut().map(|op| {
                let insert = if let Some(rule) = exclude_rule {
                    !rule.method_match(CornettiHttpMethod::OPTIONS)
                } else if let Some(rule) = only_rule {
                    rule.method_match(CornettiHttpMethod::OPTIONS)
                } else {
                    default
                };

                if insert {
                    op.security
                        .get_or_insert_with(Vec::new)
                        .extend(sec_requirements.clone());
                }
            });
        });
    }
}
