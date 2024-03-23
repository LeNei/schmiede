CREATE TABLE post (
  id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
  title VARCHAR(64) NOT NULL,
  content TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('post');
