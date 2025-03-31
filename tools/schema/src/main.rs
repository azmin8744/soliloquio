use async_graphql::*;
use graphql::queries::Queries as QueryRoot;
use graphql::mutations::Mutations as MutationRoot;
use graphql::subscriptions::Subscriptions as SubscriptionRoot;

fn main() -> std::io::Result<()> {
    let schema = Schema::build(QueryRoot, MutationRoot::default(), SubscriptionRoot).finish();
    // Print the schema in SDL format
    println!("{}", &schema.sdl());
    Ok(())
}