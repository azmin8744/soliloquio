use async_graphql::MergedObject;
mod assets;
mod posts;
mod users;

#[derive(MergedObject, Default)]
pub struct Queries(users::UserQueries, posts::PostQueries, assets::AssetQueries);
