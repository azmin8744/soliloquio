SRCS = schema.sql


.PHONY: restore_database restore_schema reset_database graphql_schema

entity:
	PGPASSWORD=${POSTGRES_PASSWORD} sea-orm-cli generate entity -u "postgres://postgres@localhost:5432/$(DB_NAME)" -o ./packages/models/src -l

restore_database:
	PGPASSWORD=$(POSTGRES_PASSWORD) psql -U postgres -h localhost -c "DROP DATABASE IF EXISTS $(DB_NAME);"
	PGPASSWORD=$(POSTGRES_PASSWORD) psql -U postgres -h localhost -c "CREATE DATABASE $(DB_NAME);"

restore_schema:
	PGPASSWORD=$(POSTGRES_PASSWORD) psql -U postgres -h localhost $(DB_NAME) < schema.sql

reset_database: restore_database restore_schema

migrate:
	PGPASSWORD=$(POSTGRES_PASSWORD) psqldef -U postgres -h localhost $(DB_NAME) < schema.sql

graphql_schema:
	cd tools/schema && cargo run > ../../schema.graphql
