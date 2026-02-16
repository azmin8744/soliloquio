# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Database Management
- `make reset_database` - Drops and recreates the database, then applies schema
- `make restore_database` - Only drops and recreates the database 
- `make restore_schema` - Only applies schema.sql to existing database
- `make migrate` - Uses psqldef to apply schema changes (safer for production)
- `make entity` - Regenerates SeaORM entity models from database schema

### Build and Run
- `cargo build` - Build the main application and all workspace packages
- `cargo run` - Start the GraphQL server at http://localhost:8000
- `cargo test` - Run all tests across workspace packages
- `make graphql_schema` - Generate schema.graphql from GraphQL types

### Database Setup
- `docker-compose up -d` - Start PostgreSQL container
- Default connection: `postgres://postgres:password@localhost:5432/soliloquio`

## Architecture

### Rust Workspace Structure
This is a Rust workspace with multiple packages organized by domain:

- **Main Application** (`src/main.rs`): Actix-web server with GraphQL endpoint
- **graphql**: GraphQL schema, resolvers, queries, mutations, subscriptions
- **models**: SeaORM entity models generated from database schema
- **services**: Business logic and authentication services
- **data-access-objects**: Database access layer (in development)
- **repositories**: Repository pattern implementation (in development)

### GraphQL Architecture
- **Queries**: Read operations in `packages/graphql/src/queries/`
- **Mutations**: Write operations in `packages/graphql/src/mutations/`
- **Subscriptions**: Real-time operations in `packages/graphql/src/subscriptions/`
- **Types**: GraphQL types and resolvers in `packages/graphql/src/types/`

### Authentication System
- JWT-based authentication with refresh tokens
- Multi-device support with device-specific refresh tokens
- Tokens stored as SHA256 hashes in `refresh_tokens` table
- Access tokens expire in 1 hour, refresh tokens in 7 days
- GraphiQL IDE available at http://localhost:8000 for testing

### Database Schema
- **users**: User accounts with email/password
- **posts**: User-generated content with markdown support
- **refresh_tokens**: Multi-device session management

### Key Dependencies
- **async-graphql**: GraphQL server implementation
- **actix-web**: Web framework
- **sea-orm**: Database ORM with PostgreSQL
- **uuid**: UUID generation and handling
- **jsonwebtoken**: JWT token handling
- **pulldown-cmark**: Markdown processing with caching
- **tracing / tracing-subscriber**: Structured logging with env-filter and JSON support
- **tracing-actix-web**: Per-request HTTP logging middleware

### Environment Variables
Required environment variables (see `.env` file):
- `POSTGRES_PASSWORD`: Database password
- `DB_NAME`: Database name (default: soliloquio)
- `DATABASE_URL`: Full PostgreSQL connection string
- `TOKEN_SECRET`: JWT signing secret
- `TOKEN_EXPIRATION_SECONDS`: Access token expiration
- `REFRESH_TOKEN_EXPIRATION_DAYS`: Refresh token expiration
- `HOST_NAME`: Token issuer hostname
- `RUST_LOG`: Log level filter (default: `info`). Use `RUST_LOG=debug` for DB queries, `RUST_LOG=sqlx=debug` for SQL only
- `LOG_FORMAT`: `json` for structured JSON output, anything else for pretty (default: `pretty`)

### Testing
- Run `cargo test` to execute all tests
- Tests are located alongside source files following Rust conventions
- Authentication system has comprehensive test coverage

### Package Dependencies
Packages depend on each other in this order:
- `models` (base layer)
- `services` (depends on models)
- `graphql` (depends on models, services)
- Main application (depends on graphql, services, models)

### Frontend

See @./client/README.md .

When making changes, ensure dependencies are updated in the correct order and that the workspace builds successfully with `cargo build`.