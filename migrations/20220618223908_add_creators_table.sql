create table creators (
    id UUID not null PRIMARY KEY default gen_random_uuid(),
    name TEXT UNIQUE not null,
    email TEXT UNIQUE not null,
    pfp TEXT null,
    pw_hash TEXT not null,
    created timestamp with time zone not null default current_timestamp
  );
