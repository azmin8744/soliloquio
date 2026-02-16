use async_graphql::MergedObject;
mod posts;
mod users;

#[derive(MergedObject, Default)]
pub struct Queries(users::UserQueries, posts::PostQueries);
