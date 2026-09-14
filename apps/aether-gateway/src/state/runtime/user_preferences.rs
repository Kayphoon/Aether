use crate::{AppState, GatewayError, GatewayUserPreferenceView};

impl AppState {
    pub(crate) async fn read_user_preferences(
        &self,
        user_id: &str,
    ) -> Result<Option<GatewayUserPreferenceView>, GatewayError> {
        self.data
            .read_user_preferences(user_id)
            .await
            .map(|value| value.map(Into::into))
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn write_user_preferences<T>(
        &self,
        preferences: T,
    ) -> Result<Option<GatewayUserPreferenceView>, GatewayError>
    where
        T: Into<GatewayUserPreferenceView>,
    {
        let preferences = preferences.into();
        let raw_preferences: crate::data::state::StoredUserPreferenceRecord = preferences.into();
        self.data
            .write_user_preferences(&raw_preferences)
            .await
            .map(|value| value.map(Into::into))
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }
}
