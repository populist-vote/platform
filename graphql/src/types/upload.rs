use async_graphql::{SimpleObject, ID};

#[derive(Clone, SimpleObject)]
pub struct FileInfo {
    id: ID,
    filename: String,
    mimetype: Option<String>,
}
