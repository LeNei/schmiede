-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

create table users (
  id UUID not null primary key default (uuid_generate_v4()),
  first_name varchar(100) not null,
  last_name varchar(100) not null,
  email varchar(255) not null unique,
  verified boolean not null default false,
  password varchar(100) not null,
  role varchar(50) not null default 'user',
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

CREATE INDEX user_email_idx ON users (email);
SELECT diesel_manage_updated_at('users');
