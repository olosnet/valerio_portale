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

            let apply = |op: &mut utoipa::openapi::path::Operation, method: &CornettiHttpMethod| {
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
                }
            };

            if let Some(op) = path.get.as_mut() {
                apply(op, &CornettiHttpMethod::GET);
            }
            if let Some(op) = path.post.as_mut() {
                apply(op, &CornettiHttpMethod::POST);
            }
            if let Some(op) = path.put.as_mut() {
                apply(op, &CornettiHttpMethod::PUT);
            }
            if let Some(op) = path.patch.as_mut() {
                apply(op, &CornettiHttpMethod::PATCH);
            }
            if let Some(op) = path.delete.as_mut() {
                apply(op, &CornettiHttpMethod::DELETE);
            }
            if let Some(op) = path.head.as_mut() {
                apply(op, &CornettiHttpMethod::HEAD);
            }
            if let Some(op) = path.options.as_mut() {
                apply(op, &CornettiHttpMethod::OPTIONS);
            }
        });
    }
}
