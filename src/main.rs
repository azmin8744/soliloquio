use actix_web::{
    guard, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use actix_cors::Cors;
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

fn get_token_from_request(req: &HttpRequest) -> Option<Token> {
    // First check Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            return Some(Token(auth_str.to_string()));
        }
    }
    
    // Then check for access_token cookie
    if let Some(cookie_header) = req.headers().get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            // Parse cookies manually to find access_token
            for cookie_pair in cookie_str.split(';') {
                let cookie_pair = cookie_pair.trim();
                if cookie_pair.starts_with("access_token=") {
                    let token = cookie_pair.trim_start_matches("access_token=");
                    return Some(Token(token.to_string()));
                }
            }
        }
    }
    
    None
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
    if let Some(token) = get_token_from_request(&req) {
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
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:8001")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec!["content-type", "authorization"])
                    .supports_credentials()
            )
            .app_data(web::Data::new(schema.clone()))
            .service(web::resource("/").guard(guard::Get()).to(graphiql))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/ws").to(index_ws))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}