/// Common utility functions.
pub mod common {
    use rand::{RngExt, rng};

    /// Reads a value from the given environment variable, falling back to a file
    /// whose path is specified by the `env_file` environment variable.
    ///
    /// Returns the file contents trimmed, or the environment variable value if
    /// the file cannot be read.
    pub fn env_or_envfile(env: &str, env_file: &str) -> Option<String> {
        let env_file_path = std::env::var(env_file).unwrap_or_else(|_| env_file.to_string());
        let result = std::fs::read_to_string(env_file_path);

        match result {
            Ok(content) => Some(content.trim().to_string()),
            Err(_) => std::env::var(env).ok(),
        }
    }

    /// Generates a random alphanumeric string with length between `min` and `max` inclusive.
    pub fn generate_random_string(min: usize, max: usize) -> String {
        let length = rng().random_range(min..=max);
        let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            .chars()
            .collect();
        (0..length)
            .map(|_| chars[rng().random_range(0..chars.len())])
            .collect()
    }

    /// Prepends the API prefix to the given path.
    ///
    /// If the prefix is empty, returns the normalized path unchanged.
    /// If the path is `"/"`, returns the prefix followed by `"/"`.
    pub fn apply_api_prefix(prefix: &str, path: &str) -> String {
        let normalized_path = if path.is_empty() {
            "/".to_string()
        } else if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };

        let prefix = prefix.trim_matches('/');
        if prefix.is_empty() {
            return normalized_path;
        }

        if normalized_path == "/" {
            return format!("{}/", prefix);
        }

        format!("/{}{}", prefix, normalized_path)
    }
}

/// Security-related helpers: password hashing, verification, generation.
pub mod sec {

    use argon2::{
        Argon2, PasswordHash, PasswordVerifier,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    use rand::seq::IndexedRandom;
    use std::collections::HashSet;

    /// Hashes a password using Argon2 with a random salt.
    ///
    /// Returns the PHC-encoded hash string.
    ///
    /// # Panics
    ///
    /// Panics if Argon2 hashing fails (unexpected for valid input).
    pub fn hash_password(password: &String) -> String {
        let salt: SaltString = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    /// Verifies a password against an Argon2 PHC-encoded hash.
    ///
    /// # Panics
    ///
    /// Panics if `source_hash` is not a valid PHC string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cornetti::core::helpers::sec::{hash_password, verify_password};
    ///
    /// let password = "my-password".to_string();
    /// let hash = hash_password(&password);
    /// assert!(verify_password(&hash, "my-password"));
    /// assert!(!verify_password(&hash, "wrong-password"));
    /// ```
    pub fn verify_password(source_hash: &str, password: &str) -> bool {
        let parsed_hash = PasswordHash::new(source_hash).unwrap();
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    /// Generates a random password of the given length.
    ///
    /// `types` is an optional slice of character categories:
    /// `"chars"` (lower+upper), `"digits"` (0-9), `"symbols"` (special chars).
    /// Defaults to all three if `None`.
    ///
    /// # Panics
    ///
    /// Panics if the combined character set is smaller than the requested length.
    pub fn random_pass(length: usize, types: Option<&[&str]>) -> String {
        let types = types.unwrap_or(&["chars", "digits", "symbols"]);

        let mut chars = String::new();
        let types_set: HashSet<&str> = types.iter().cloned().collect();

        if types_set.contains("chars") {
            let lower = "abcdefghijklmnopqrstuvwxyz";
            let upper = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
            chars.push_str(lower);
            chars.push_str(upper);
        }

        if types_set.contains("digits") {
            let num = "0123456789";
            chars.push_str(num);
        }

        if types_set.contains("symbols") {
            let symbols = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
            chars.push_str(symbols);
        }

        let chars_vec: Vec<char> = chars.chars().collect();

        if chars_vec.len() < length {
            panic!("Not enough characters to generate a password of the requested length");
        }

        let mut rng = rand::rng();

        let password: String = chars_vec.sample(&mut rng, length).cloned().collect();

        password
    }
}

/// Utoipa OpenAPI documentation helpers.
pub mod utoipa {
    use utoipa::{
        OpenApi,
        openapi::{OpenApi as OpenApiType, PathsBuilder},
    };

    use crate::core::helpers::common::apply_api_prefix;

    /// Merges multiple `OpenApiType` documents into one, starting from `T::openapi()`.
    pub fn combine_api_docs<T: OpenApi>(modules: Vec<OpenApiType>) -> OpenApiType {
        let mut combined = T::openapi();

        for module_doc in modules {
            combined.merge(module_doc);
        }

        combined
    }

    /// Assigns auto-generated operation IDs to all paths in the OpenAPI document.
    ///
    /// Format: `{module_name}::{http_method}_{counter}`.
    pub fn auto_operation_id(doc: &mut utoipa::openapi::OpenApi, module_name: &str) {
        let mut counter: usize = 0;
        doc.paths.paths.iter_mut().for_each(|(_, path)| {
            if let Some(op) = path.get.as_mut() {
                op.operation_id = Some(format!("{}::get_{}", module_name, counter));
            }
            if let Some(op) = path.post.as_mut() {
                op.operation_id = Some(format!("{}::post_{}", module_name, counter));
            }
            if let Some(op) = path.put.as_mut() {
                op.operation_id = Some(format!("{}::put_{}", module_name, counter));
            }
            if let Some(op) = path.delete.as_mut() {
                op.operation_id = Some(format!("{}::delete_{}", module_name, counter));
            }
            if let Some(op) = path.patch.as_mut() {
                op.operation_id = Some(format!("{}::patch_{}", module_name, counter));
            }
            if let Some(op) = path.options.as_mut() {
                op.operation_id = Some(format!("{}::options_{}", module_name, counter));
            }
            if let Some(op) = path.head.as_mut() {
                op.operation_id = Some(format!("{}::head_{}", module_name, counter));
            }

            counter += 1;
        });
    }

    /// Prepends a context path to all path entries in the OpenAPI document.
    pub fn auto_context_path(doc: &mut utoipa::openapi::OpenApi, context_path: &str) {
        let context_path = context_path.trim_end_matches('/');

        let existing_paths = doc.paths.clone();

        let mut new_paths = PathsBuilder::new();

        for (path, path_item) in existing_paths.paths {
            let new_path = if path.is_empty() && !context_path.starts_with('/') {
                format!("/{}", context_path)
            } else if path.starts_with('/') && path.len() == 1 || path.is_empty() {
                context_path.to_string()
            } else if path.starts_with('/') {
                format!("{}{}", context_path, path)
            } else {
                format!("{}/{}", context_path, path)
            };

            new_paths = new_paths.path(new_path, path_item);
        }

        doc.paths = new_paths.build();
    }

    /// Trait for types that can produce an OpenAPI document from a module-specific template.
    pub trait BaseApiDoc {
        fn api_doc<T: utoipa::OpenApi>(&self) -> utoipa::openapi::OpenApi;
    }

    /// Entry for building an OpenAPI document with auto-generated operation IDs
    /// and context path from base configuration.
    pub struct ApiDocEntry<'a> {
        /// Module name used in operation IDs.
        pub module_name: String,
        /// Context path prepended to all routes.
        pub context_path: String,
        /// Base application configuration for API prefix.
        pub base_conf: &'a crate::core::confs::BaseConf,
    }

    impl BaseApiDoc for ApiDocEntry<'_> {
        fn api_doc<T: utoipa::OpenApi>(&self) -> utoipa::openapi::OpenApi {
            let mut doc: utoipa::openapi::OpenApi = T::openapi();

            let prefix = &apply_api_prefix(&self.base_conf.api_prefix, &self.context_path);
            auto_operation_id(&mut doc, &self.module_name);
            auto_context_path(&mut doc, prefix);

            doc
        }
    }
}
