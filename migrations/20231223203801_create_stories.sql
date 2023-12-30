CREATE TABLE stories
(
    id uuid default gen_random_uuid() primary key,
    name varchar(100) not null,
    owner varchar(100) not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    deleted_at timestamptz
);

CREATE INDEX stories_owner_index 
    ON stories 
    USING btree(owner);
