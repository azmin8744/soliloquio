create table posts (
    id uuid primary key,
    title text not null,
    markdown_content text,
    user_id uuid not null,
    is_published boolean default false not null,
    first_published_at timestamp,
    created_at timestamp default current_timestamp,
    updated_at timestamp default current_timestamp
);

create table users (
    id uuid primary key,
    email text not null unique,
    password text not null,
    refresh_token text,
    created_at timestamp default current_timestamp,
    updated_at timestamp default current_timestamp
);

alter table posts add constraint fk_user_id foreign key (user_id) references users (id);
