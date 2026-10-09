//! The one non-GET request the API layer may send: a form POST to the OAuth token endpoint.
//! Injected as a trait so tests use a loopback fake and nothing else can POST.

/// A transport that can POST a form and return (status, body). `Err` means no HTTP response at all.
pub trait FormPost: Send + Sync {
    fn post_form(&self, url: &str, form: &[(&str, &str)]) -> Result<(u16, String), String>;
}

/// The real transport (ureq over rustls). Only the token store calls it, and only with the token URL.
pub struct UreqForm {
    agent: ureq::Agent,
}

impl Default for UreqForm {
    fn default() -> Self {
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .build();
        UreqForm {
            agent: config.into(),
        }
    }
}

impl FormPost for UreqForm {
    fn post_form(&self, url: &str, form: &[(&str, &str)]) -> Result<(u16, String), String> {
        let mut resp = self
            .agent
            .post(url)
            .send_form(form.iter().copied())
            .map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let body = resp
            .body_mut()
            .read_to_string()
            .map_err(|e| e.to_string())?;
        Ok((status, body))
    }
}
