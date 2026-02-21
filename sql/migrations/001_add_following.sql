-- Migration: Add following table and follower_count column to users
-- Run this on existing databases that were created before the following feature.
-- For fresh installs, database_ddl.sql already includes these changes.

-- Add follower_count to users if not present (e.g. from older schema)
ALTER TABLE users ADD COLUMN IF NOT EXISTS follower_count INTEGER NOT NULL DEFAULT 0;

-- Create following table if not exists
CREATE TABLE IF NOT EXISTS following (
    follower  TEXT                      NOT NULL REFERENCES users(id),
    followed  TEXT                      NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (follower, followed)
);

CREATE INDEX IF NOT EXISTS idx_following_follower ON following(follower);
CREATE INDEX IF NOT EXISTS idx_following_followed ON following(followed);
