use crate::VotesmartProxy;
use reqwest::{Error, Response};

pub struct District<'a>(pub &'a VotesmartProxy);

impl District<'_> {
    /// This method grabs district IDs according to the office and state.
    pub async fn get_by_office_state(
        &self,
        office_id: i32,
        state_id: String,
        district_name: Option<&str>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&officeId={office_id}&stateId={state_id}&districtName={district_name}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "District.getByOfficeState",
            office_id = office_id,
            state_id = state_id,
            district_name = district_name.unwrap_or("")
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs district IDs according to the zip code.
    pub async fn get_by_zip(&self, zip5: i32, zip4: Option<&str>) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&zip5={zip5}&zip4={zip4}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "District.getByZip",
            zip5 = zip5,
            zip4 = zip4.unwrap_or(""),
        );

        self.0.client.get(url).send().await
    }
}

#[cfg(test)]
mod tests {
    use serde_json;

    use crate::VotesmartProxy;

    #[tokio::test]
    async fn test_get_by_office_state() {
        let proxy = VotesmartProxy::new().unwrap();
        let response = proxy
            .district()
            .get_by_office_state(29949, "CO".to_string(), None)
            .await
            .unwrap();

        let status = response.status();
        let json = response.json::<serde_json::Value>().await.unwrap();
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
        assert_eq!(status.is_success(), true);
    }

    #[tokio::test]
    async fn test_get_by_zip() {
        let proxy = VotesmartProxy::new().unwrap();
        let response = proxy.district().get_by_zip(80521, None).await.unwrap();

        let status = response.status();
        let json = response.json::<serde_json::Value>().await.unwrap();
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
        assert_eq!(status.is_success(), true);
    }
}
