//! Cliente HTTP compartido. Los códigos de estado se devuelven tal cual:
//! cada llamador decide qué es "no existe" (404) y qué es "frena" (429).

use crate::Result;
use std::time::Duration;

const MAX_BODY_BYTES: u64 = 20 * 1024 * 1024;

pub struct Http {
    agent: ureq::Agent,
}

pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
}

impl Http {
    pub fn new() -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(20)))
            .user_agent(concat!("MisJuegos/", env!("CARGO_PKG_VERSION")))
            .http_status_as_error(false)
            .tls_config(tls_config())
            .build()
            .into();
        Http { agent }
    }

    pub fn get(&self, url: &str) -> Result<Response> {
        let mut response = self.agent.get(url).call()?;
        let status = response.status().as_u16();
        let body = response.body_mut().with_config().limit(MAX_BODY_BYTES).read_to_vec()?;
        Ok(Response { status, body })
    }
}

fn tls_config() -> ureq::tls::TlsConfig {
    let builder = ureq::tls::TlsConfig::builder();
    #[cfg(windows)]
    let builder = builder.provider(ureq::tls::TlsProvider::NativeTls).root_certs(ureq::tls::RootCerts::PlatformVerifier);
    builder.build()
}

impl Default for Http {
    fn default() -> Self {
        Self::new()
    }
}
