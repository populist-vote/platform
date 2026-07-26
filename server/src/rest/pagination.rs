use axum::extract::{rejection::QueryRejection, Query};
use serde::{Deserialize, Serialize};

use super::error::ApiError;

pub const DEFAULT_LIMIT: usize = 25;
pub const MAX_LIMIT: usize = 100;

#[derive(Debug, Default, Deserialize)]
pub struct PaginationQuery {
    limit: Option<String>,
    offset: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub count: usize,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Pagination {
    pub limit: usize,
    pub offset: usize,
}

impl Pagination {
    pub fn from_query(
        query: Result<Query<PaginationQuery>, QueryRejection>,
        instance: &str,
    ) -> Result<Self, ApiError> {
        let Query(query) = query.map_err(|error| {
            ApiError::malformed_query(
                format!("The query string could not be parsed: {error}"),
                instance,
            )
        })?;

        let limit = parse_parameter(query.limit, "limit", DEFAULT_LIMIT, instance)?;
        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(ApiError::invalid_parameter(
                "limit",
                format!("must be between 1 and {MAX_LIMIT}"),
                instance,
            ));
        }

        let offset = parse_parameter(query.offset, "offset", 0, instance)?;

        Ok(Self { limit, offset })
    }

    pub fn page<T: Clone>(&self, items: &[T]) -> (Vec<T>, PaginationMeta) {
        let total = items.len();
        let data = items
            .iter()
            .skip(self.offset)
            .take(self.limit)
            .cloned()
            .collect::<Vec<_>>();
        let meta = PaginationMeta {
            count: data.len(),
            total,
            limit: self.limit,
            offset: self.offset,
        };
        (data, meta)
    }
}

fn parse_parameter(
    value: Option<String>,
    name: &'static str,
    default: usize,
    instance: &str,
) -> Result<usize, ApiError> {
    value
        .map(|value| {
            value.parse::<usize>().map_err(|_| {
                ApiError::invalid_parameter(name, "must be a non-negative integer", instance)
            })
        })
        .transpose()
        .map(|value| value.unwrap_or(default))
}
