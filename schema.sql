create table posts (
    id uuid primary key,
    title text not null,
    body text not null,
    user_id uuid not null,
    published_at timestamp,
    created_at timestamp default current_timestamp,
    updated_at timestamp default current_timestamp
);

create table users (
    id uuid primary key,
    email text not null,
    password text not null,
    created_at timestamp default current_timestamp,
    updated_at timestamp default current_timestamp
);

alter table posts add constraint fk_user_id foreign key (user_id) references users (id);
