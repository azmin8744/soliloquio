mod setup;

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema
};

use async_graphql_rocket::*;
use setup::set_up_db;
use graphql::queries::Queries as QueryRoot;
use graphql::mutations::Mutations as MutationRoot;

use rocket::{response::content, *};
type SchemaType = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[get("/")]
async fn index() -> &'static str {
    "Hello, soliloquio!"
}

#[rocket::get("/graphql")]
fn graphql_playground() -> content::RawHtml<String> {
    content::RawHtml(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[rocket::post("/graphql", data = "<request>", format = "application/json")]
async fn graphql_request(schema: &State<SchemaType>, request: GraphQLRequest) -> GraphQLResponse {
    request.execute(schema.inner()).await
}

#[launch] // The "main" function of the program
async fn rocket() -> _ {
    let db = match set_up_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };

        // Build the Schema
    let schema = Schema::build(QueryRoot, MutationRoot::default(), EmptySubscription)
        .data(db) // Add the database connection to the GraphQL global context
        .finish();
    rocket::build()
        .manage(schema)
        .mount("/", routes![index, graphql_playground, graphql_request])
}

