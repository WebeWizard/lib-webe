use super::Request;
use super::Responder;
use super::Response;
use super::Validation;
use super::ValidationResult;

use super::file::{FileResponder, FileResponderError};

use async_trait::async_trait;

/// A single-page-app fallback responder.
///
/// Captures stray endpoints that the SPA handles client-side (for example, a
/// user refreshing on a deep link such as `/flash/23434455`) by serving a single
/// application index file. Internally delegates to a [`FileResponder`] mounted on
/// the application file.
///
/// > Renamed from `SPAResponder` in the web crate revamp (see the crate README
/// > migration notes).
pub struct SpaResponder {
    app_file_path: String,
    file_responder: FileResponder,
}

impl SpaResponder {
    /// Creates a responder that always serves `app_file_path` from within
    /// `mount_point`.
    ///
    /// Returns [`FileResponderError`] when the mount point cannot be resolved.
    pub fn new(
        mount_point: String,
        app_file_path: String,
    ) -> Result<SpaResponder, FileResponderError> {
        match FileResponder::new(mount_point, String::new()) {
            Ok(file_responder) => Ok(SpaResponder {
                app_file_path,
                file_responder,
            }),
            Err(error) => Err(error),
        }
    }
}

#[async_trait]
impl Responder for SpaResponder {
    async fn validate(
        &self,
        request: &Request,
        _params: &Vec<(String, String)>,
        validation: Validation,
    ) -> ValidationResult {
        // pass on to the internal file responder, but fudge the param so the
        // app file path becomes the complete path (empty param name)
        let fudged_params = vec![(String::new(), self.app_file_path.clone())];
        self.file_responder
            .validate(request, &fudged_params, validation)
            .await
    }

    async fn build_response(
        &self,
        request: &mut Request,
        params: &Vec<(String, String)>,
        validation: Validation,
    ) -> Result<Response, u16> {
        self.file_responder
            .build_response(request, params, validation)
            .await
    }
}
