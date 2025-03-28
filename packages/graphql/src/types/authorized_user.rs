use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct AuthorizedUser {
    pub token: String,
    pub refresh_token: String,
}
