use crate::VotesmartProxy;
use reqwest::{Error, Response};

pub struct Votes<'a>(pub &'a VotesmartProxy);

impl Votes<'_> {
    /// This method dumps categories that contain released bills according to year and state.
    pub async fn get_categories(
        &self,
        year: i32,
        state_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&year={year}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getCategories",
            year = year,
            state_id = state_id.unwrap_or("NA".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps general information on a bill.
    pub async fn get_bill(&self, bill_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&billId={bill_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getBill",
            bill_id = bill_id,
        );

        self.0.client.get(url).send().await
    }

    /// This gets detailed action information on a certain stage of the bill.
    pub async fn get_bill_action(&self, action_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&actionId={action_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getBillAction",
            action_id = action_id,
        );

        self.0.client.get(url).send().await
    }

    /// Method provides votes listed by candidate on a certain bill action.
    pub async fn get_bill_action_votes(&self, action_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&actionId={action_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getBillActionVotes",
            action_id = action_id,
        );

        self.0.client.get(url).send().await
    }

    /// Returns a single vote according to official and action.
    pub async fn get_bill_action_vote_by_official(
        &self,
        action_id: i32,
        candidate_id: i32,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&actionId={action_id}&candidateId={candidate_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getBillActionVotesByOfficial",
            action_id = action_id,
            candidate_id = candidate_id
        );

        self.0.client.get(url).send().await
    }

    /// Returns a list of bills that are like the billNumber input.
    pub async fn get_by_bill_number(&self, bill_number: String) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&billNumber={bill_number}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getByBillNumber",
            bill_number = bill_number,
        );

        self.0.client.get(url).send().await
    }

    // Returns a list of bills that fit the category, year, and state input.
    pub async fn get_bills_by_category_year_state(
        &self,
        category_id: i32,
        year: i32,
        state_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&categoryId={category_id}&year={year}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getBillsByCategoryYearState",
            category_id = category_id,
            year = year,
            state_id = state_id.unwrap_or("NA".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// Returns a list of bills that fit the year and state input.
    pub async fn get_bills_by_year_state(
        &self,
        year: i32,
        state_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&year={year}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getBillsByYearState",
            year = year,
            state_id = state_id.unwrap_or("NA".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// Returns a list of bills that fit the candidate and year.
    pub async fn get_bills_by_official_year_office(
        &self,
        candidate_id: i32,
        year: i32,
        office_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&year={year}&officeId={office_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getByBillNumber",
            candidate_id = candidate_id,
            year = year,
            office_id = office_id.unwrap_or("NULL".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// Returns a list of bills that fit the candidate and category.
    pub async fn get_bills_by_official_category_office(
        &self,
        candidate_id: i32,
        category_id: i32,
        office_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&categoryId={category_id}&officeId={office_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getBillsByOfficialCategoryOffice",
            candidate_id = candidate_id,
            category_id = category_id,
            office_id = office_id.unwrap_or("NULL".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps all the bills an official has voted on based on the candidateId, officeId, categoryId, and year
    pub async fn get_by_official(
        &self,
        candidate_id: i32,
        office_id: Option<String>,
        category_id: Option<String>,
        year: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&categoryId={category_id}&officeId={office_id}&year={year}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getByOfficial",
            candidate_id = candidate_id,
            category_id = category_id.unwrap_or("NULL".to_string()),
            office_id = office_id.unwrap_or("NULL".to_string()),
            year = year.unwrap_or("NULL".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// Returns a list of bills that fit the sponsor's candidateId and year.
    pub async fn get_bills_by_sponsor_year(
        &self,
        candidate_id: i32,
        year: i32,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&year={year}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getBillsBySponsorYear",
            candidate_id = candidate_id,
            year = year
        );

        self.0.client.get(url).send().await
    }

    /// Returns a list of bills that fit the sponsor's candidateId and category.
    pub async fn get_bills_by_sponsor_category(
        &self,
        candidate_id: i32,
        category_id: i32,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&categoryId={category_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getBillsBySponsorCategory",
            candidate_id = candidate_id,
            category_id = category_id
        );

        self.0.client.get(url).send().await
    }

    /// Returns a list of recent bills according to the state. Max returned is 100 or however much less you want.
    pub async fn get_bills_by_state_recent(
        &self,
        state_id: String,
        amount: Option<i32>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&state_id={state_id}&amount={amount}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getBillsByStateRecent",
            state_id = state_id,
            amount = amount.unwrap_or(100)
        );

        self.0.client.get(url).send().await
    }

    /// Returns a list of vetoes according to candidate.
    pub async fn get_vetoes(&self, candidate_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Votes.getVetoes",
            candidate_id = candidate_id
        );

        self.0.client.get(url).send().await
    }
}
