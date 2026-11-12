//! TLS config for every `ureq` agent: rustls with the pure Rust RustCrypto provider (no ring, no C or assembly).

use std::sync::Arc;

pub fn tls_config() -> ureq::tls::TlsConfig {
    ureq::tls::TlsConfig::builder()
        .provider(ureq::tls::TlsProvider::Rustls)
        .unversioned_rustls_crypto_provider(Arc::new(rustls_rustcrypto::provider()))
        .build()
}

/// `HttpClient.PostAsJsonAsync(url, payload)` off the game loop: a detached thread, a failure is logged.
/// A non-2xx answer is not an error: the response is dropped.
pub fn post_json_detached(url: &str, body: &str) {
    let (url, body) = (url.to_owned(), body.to_owned());
    std::thread::spawn(move || {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .tls_config(tls_config())
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .http_status_as_error(false)
            .build()
            .into();
        if let Err(e) = agent
            .post(&url)
            .content_type("application/json; charset=utf-8")
            .send(body)
        {
            tracing::error!("HTTP POST to {url} failed: {e}");
        }
    });
}
