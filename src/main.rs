use actix_cors::Cors;
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use tracing_actix_web::TracingLogger;
mod setup;
use graphql::config::SingleUserMode;
use graphql::mutations::Mutations as MutationRoot;
use graphql::public::{build_public_schema, PublicApiKey, PublicSchema};
use graphql::queries::Queries as QueryRoot;
use graphql::subscriptions::{on_connection_init, Subscriptions as SubscriptionRoot};
use graphql::utilities::MarkdownCache;
use services::authentication::Token;
use services::email::EmailService;
use setup::set_up_db;

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

fn get_api_key_from_request(req: &HttpRequest) -> Option<PublicApiKey> {
    // Check X-API-Key header first
    if let Some(v) = req.headers().get("X-API-Key") {
        if let Ok(s) = v.to_str() {
            return Some(PublicApiKey(s.to_string()));
        }
    }
    // Fall back to Authorization: Bearer slq_...
    if let Some(v) = req.headers().get("Authorization") {
        if let Ok(s) = v.to_str() {
            let stripped = s.strip_prefix("Bearer ").unwrap_or(s);
            if stripped.starts_with("slq_") {
                return Some(PublicApiKey(stripped.to_string()));
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

async fn public_index(
    schema: web::Data<PublicSchema>,
    req: HttpRequest,
    gql_request: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = gql_request.into_inner();
    if let Some(key) = get_api_key_from_request(&req) {
        request = request.data(key);
    }
    schema.execute(request).await.into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_default();
    if log_format.eq_ignore_ascii_case("json") {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .init();
    }

    let db = match set_up_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };

    let allowed_origins: Vec<String> = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:8001".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let public_cors_origins: Option<Vec<String>> = {
        let raw = std::env::var("PUBLIC_CORS_ORIGINS")
            .unwrap_or_else(|_| "*".to_string());
        if raw.trim() == "*" {
            None // wildcard
        } else {
            Some(raw.split(',').map(|s| s.trim().to_string()).collect())
        }
    };

    let markdown_cache = MarkdownCache::new();
    let email_service = EmailService::from_env();
    let single_user_mode = SingleUserMode(
        std::env::var("SINGLE_USER_MODE")
            .unwrap_or_default()
            .eq_ignore_ascii_case("true"),
    );

    let schema = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot,
    )
    .data(db.clone())
    .data(markdown_cache.clone())
    .data(email_service)
    .data(single_user_mode)
    .finish();

    let public_schema = build_public_schema(db, markdown_cache);

    tracing::info!("GraphiQL IDE: http://localhost:8000");

    HttpServer::new(move || {
        let mut main_cors = Cors::default()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec!["content-type", "authorization", "x-api-key"])
            .supports_credentials();
        for origin in &allowed_origins {
            main_cors = main_cors.allowed_origin(origin);
        }

        let public_cors = match &public_cors_origins {
            None => Cors::default()
                .allowed_methods(vec!["POST"])
                .allowed_headers(vec!["content-type", "authorization", "x-api-key"])
                .allow_any_origin(),
            Some(origins) => {
                let mut c = Cors::default()
                    .allowed_methods(vec!["POST"])
                    .allowed_headers(vec!["content-type", "authorization", "x-api-key"]);
                for o in origins {
                    c = c.allowed_origin(o);
                }
                c
            }
        };

        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(schema.clone()))
            .app_data(web::Data::new(public_schema.clone()))
            .service(
                web::scope("/public")
                    .wrap(public_cors)
                    .service(web::resource("").guard(guard::Post()).to(public_index)),
            )
            .service(
                web::scope("")
                    .wrap(main_cors)
                    .service(web::resource("/").guard(guard::Get()).to(graphiql))
                    .service(web::resource("/").guard(guard::Post()).to(index))
                    .service(web::resource("/ws").to(index_ws)),
            )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
