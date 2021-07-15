use graphql_client_web::Response;

use crate::error::UiError;

pub mod auth;
pub mod dicebot;

/// Extensions to the GraphQL web response type to add convenience,
/// particularly when working with errors.
trait ResponseExt<T> {
    /// Get the data from the response, or gather all server-side
    /// errors into a UiError variant.
    fn data(self) -> Result<T, UiError>;
}

impl<T> ResponseExt<T> for Response<T> {
    fn data(self) -> Result<T, UiError> {
        let data = self.data;
        let errors = self.errors;

        let data = data.ok_or_else(|| {
            UiError::ApiError(
                errors
                    .map(|errors| {
                        errors
                            .into_iter()
                            .map(|e| e.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .unwrap_or("unknown error".into()),
            )
        })?;

        Ok(data)
    }
}
