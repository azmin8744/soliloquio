use actix_web::{
    guard, http::header::HeaderMap, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
mod setup;
use setup::set_up_db;
use graphql::queries::Queries as QueryRoot;
use graphql::mutations::Mutations as MutationRoot;
use graphql::subscriptions::{Subscriptions as SubscriptionRoot, on_connection_init};
use graphql::utilities::MarkdownCache;
use services::authentication::Token;

type SchemaType = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

fn get_token_from_headers(headers: &HeaderMap) -> Option<Token> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().map(|s| Token(s.to_string())).ok())
}

async fn graphiql() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            GraphiQLSource::build()
                .endpoint("/")
                .subscription_endpoint("/ws")
                .finish(),
        )
}

async fn index(
    schema: web::Data<SchemaType>,
    req: HttpRequest,
    gql_request: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = gql_request.into_inner();
    if let Some(token) = get_token_from_headers(req.headers()) {
        request = request.data(token);
    }
    schema.execute(request).await.into()
}

async fn index_ws(
    schema: web::Data<SchemaType>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema))
        .on_connection_init(on_connection_init)
        .start(&req, payload)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = match set_up_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };

    let markdown_cache = MarkdownCache::new();

    let schema = Schema::build(QueryRoot, MutationRoot::default(), SubscriptionRoot)
    .data(db) // Add the database connection to the GraphQL global context
    .data(markdown_cache) // Add the markdown cache to the GraphQL global context
    .finish();

    println!("GraphiQL IDE: http://localhost:8000");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .service(web::resource("/").guard(guard::Get()).to(graphiql))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/ws").to(index_ws))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}