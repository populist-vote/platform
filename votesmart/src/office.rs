use crate::VotesmartProxy;
use reqwest::{Error, Response};

pub struct Office<'a>(pub &'a VotesmartProxy);

impl Office<'_> {
    /// This method dumps all office types Votesmart keeps track of
    pub async fn get_types(&self) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Office.getTypes",
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps the branches of government and their IDs
    pub async fn get_branches(&self) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Office.getBranches",
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps the levels of government and their IDs
    pub async fn get_levels(&self) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Office.getLevels",
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps offices Votesmart keeps track of according to type.
    pub async fn get_offices_by_type(&self, office_type_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&officeTypeId={office_type_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Office.getOfficesByType",
            office_type_id = office_type_id
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps offices Votesmart keeps track of according to level.
    pub async fn get_offices_by_level(&self, office_level_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&levelId={office_level_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Office.getOfficesByLevel",
            office_level_id = office_level_id
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps offices Votesmart keeps track of according to type and level.
    pub async fn get_offices_by_level_type(
        &self,
        office_type_id: i32,
        office_level_id: i32,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&officeTypeId={office_type_id}&officeLevelId={office_level_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Office.getOfficesByTypeLevel",
            office_type_id = office_type_id,
            office_level_id = office_level_id
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps offices Votesmart keeps track of according to branch and level.
    pub async fn get_offices_by_branch_level(
        &self,
        branch_id: i32,
        office_level_id: i32,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&branchId={branch_id}&officeLevelId={office_level_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Office.getOfficesByBranchLevel",
            branch_id = branch_id,
            office_level_id = office_level_id
        );

        self.0.client.get(url).send().await
    }
}
