create table posts (
    id uuid primary key,
    title text not null,
    markdown_content text,
    description text,
    slug text,
    user_id uuid not null,
    is_published boolean default false not null,
    first_published_at timestamp,
    created_at timestamp default current_timestamp not null,
    updated_at timestamp default current_timestamp not null
);

create table users (
    id uuid primary key,
    email text not null unique,
    password text not null,
    created_at timestamp default current_timestamp,
    updated_at timestamp default current_timestamp
);

create table refresh_tokens (
    id uuid primary key default gen_random_uuid(),
    user_id uuid not null references users(id) on delete cascade,
    token_hash varchar(255) unique not null,
    expires_at timestamp not null,
    device_info varchar(255),
    created_at timestamp not null default current_timestamp,
    last_used_at timestamp
);

-- Indexes for efficient queries and cleanup
create index idx_refresh_tokens_user_id on refresh_tokens(user_id);
create index idx_refresh_tokens_token_hash on refresh_tokens(token_hash);
create index idx_refresh_tokens_expires_at on refresh_tokens(expires_at);

alter table posts add constraint fk_user_id foreign key (user_id) references users (id);

create index idx_posts_user_pagination on posts (user_id, created_at desc, id desc);
create index idx_posts_user_updated_at on posts (user_id, updated_at desc, id desc);
create index idx_posts_user_title on posts (user_id, title asc, id asc);
create unique index idx_posts_user_slug on posts (user_id, slug);
