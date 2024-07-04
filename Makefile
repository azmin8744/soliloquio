SRCS = schema.sql

entity:
	PGPASSWORD=${POSTGRES_PASSWORD} sea-orm-cli generate entity -u "postgres://postgres@localhost:5432/$(DB_NAME)" -o ./packages/models/src -l

restore_database:
	PGPASSWORD=$(POSTGRES_PASSWORD) psql -U postgres -h localhost -c "DROP DATABASE IF EXISTS $(DB_NAME);"
	PGPASSWORD=$(POSTGRES_PASSWORD) psql -U postgres -h localhost -c "CREATE DATABASE $(DB_NAME);"

restore_schema:
	psqldef -U postgres -h localhost $(DB_NAME) < schema.sql

reset_database:
	restore_database restore_schema
