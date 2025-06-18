use async_graphql::MergedObject;
mod posts;
mod users;
mod input_validators;

#[derive(MergedObject, Default)]
pub struct Mutations(
    posts::PostMutation,
    users::UserMutation,
);

