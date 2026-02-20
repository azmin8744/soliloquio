use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum PostSortBy {
    #[graphql(name = "CREATED_AT")]
    CreatedAt,
    #[graphql(name = "UPDATED_AT")]
    UpdatedAt,
    #[graphql(name = "TITLE")]
    Title,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum SortDirection {
    #[graphql(name = "ASC")]
    Asc,
    #[graphql(name = "DESC")]
    Desc,
}
