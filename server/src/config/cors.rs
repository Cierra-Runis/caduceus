use actix_cors::Cors;
use actix_web::http::Method;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    allow_origins: Option<Vec<String>>,
    allow_methods: Option<Vec<String>>,
    allow_headers: Option<Vec<String>>,
    expose_headers: Option<Vec<String>>,
    max_age: Option<usize>,
    preflight: Option<bool>,
    send_wildcard: Option<bool>,
    supports_credentials: Option<bool>,
    vary_header: Option<bool>,
    block_on_origin_mismatch: Option<bool>,
}

impl From<CorsConfig> for Cors {
    fn from(config: CorsConfig) -> Self {
        let mut cors = Cors::default();

        if let Some(origins) = &config.allow_origins {
            for origin in origins {
                cors = cors.allowed_origin(origin.as_str());
            }
        } else {
            cors = cors.allow_any_origin();
        }

        if let Some(methods) = &config.allow_methods {
            let valid_methods: Vec<Method> = methods
                .iter()
                .filter_map(|m| Method::from_bytes(m.as_bytes()).ok())
                .collect();
            cors = cors.allowed_methods(valid_methods);
        } else {
            cors = cors.allow_any_method();
        }

        if let Some(headers) = &config.allow_headers {
            for header in headers {
                cors = cors.allowed_header(header);
            }
        } else {
            cors = cors.allow_any_header();
        }

        if let Some(headers) = &config.expose_headers {
            cors = cors.expose_headers(headers);
        }

        if let Some(max_age) = config.max_age {
            cors = cors.max_age(max_age);
        }

        if matches!(&config.preflight, Some(false)) {
            cors = cors.disable_preflight();
        }

        if matches!(&config.send_wildcard, Some(true)) {
            cors = cors.send_wildcard();
        }

        // Enable credentials by default unless explicitly disabled in config
        let credentials_enabled = !matches!(&config.supports_credentials, Some(false));
        if credentials_enabled {
            cors = cors.supports_credentials();
        }

        if matches!(&config.vary_header, Some(false)) {
            cors = cors.disable_vary_header();
        }

        if let Some(block) = config.block_on_origin_mismatch {
            cors = cors.block_on_origin_mismatch(block);
        }

        cors
    }
}
