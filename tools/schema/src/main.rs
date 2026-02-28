use async_graphql::*;
use graphql::mutations::Mutations as MutationRoot;
use graphql::public::PublicQueryRoot;
use graphql::queries::Queries as QueryRoot;
use graphql::subscriptions::Subscriptions as SubscriptionRoot;
use std::fs;

fn main() -> std::io::Result<()> {
    let schema = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot,
    )
    .finish();

    let public_schema = Schema::build(
        PublicQueryRoot::default(),
        EmptyMutation,
        EmptySubscription,
    )
    .finish();

    fs::write("../../schema.graphql", schema.sdl())?;
    fs::write("../../public.schema.graphql", public_schema.sdl())?;
    eprintln!("Wrote schema.graphql and public.schema.graphql");
    Ok(())
}
