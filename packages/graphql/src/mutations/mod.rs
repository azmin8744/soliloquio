use async_graphql::MergedObject;
mod posts;
mod users;

#[derive(MergedObject, Default)]
pub struct Mutations(
    posts::PostMutation,
    users::UserMutation,
);

