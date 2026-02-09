-- OAuth 2.0 refresh tokens used for obtaining access tokens
CREATE TABLE refresh_tokens (
    id         SERIAL                    PRIMARY KEY,  -- Auto-incrementing ID
    token      TEXT                      NOT NULL,     -- The refresh token value
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL DEFAULT NOW()  -- When the token was created
);

COMMENT ON TABLE refresh_tokens IS 'OAuth 2.0 refresh tokens used for obtaining access tokens';
COMMENT ON COLUMN refresh_tokens.id IS 'Auto-incrementing primary key';
COMMENT ON COLUMN refresh_tokens.token IS 'The OAuth 2.0 refresh token value';
COMMENT ON COLUMN refresh_tokens.created_at IS 'Timestamp when the refresh token was created';

-- OAuth 2.0 access tokens used for Twitter API requests
CREATE TABLE access_tokens (
    id         SERIAL                    PRIMARY KEY,  -- Auto-incrementing ID
    token      TEXT                      NOT NULL,     -- The access token value
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL DEFAULT NOW()  -- When the token was created
);

COMMENT ON TABLE access_tokens IS 'OAuth 2.0 access tokens used for Twitter API requests';
COMMENT ON COLUMN access_tokens.id IS 'Auto-incrementing primary key';
COMMENT ON COLUMN access_tokens.token IS 'The OAuth 2.0 access token value';
COMMENT ON COLUMN access_tokens.created_at IS 'Timestamp when the access token was created';

-- Web login sessions (OAuth 2.0 user context per session)
CREATE TABLE sessions (
    id            UUID                     PRIMARY KEY,
    user_id       TEXT                     NOT NULL,
    username      TEXT                     NOT NULL,
    access_token  TEXT                     NOT NULL,
    refresh_token TEXT,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at    TIMESTAMP WITH TIME ZONE NOT NULL
);

COMMENT ON TABLE sessions IS 'Web login sessions: per-user OAuth 2.0 tokens (encrypted)';
COMMENT ON COLUMN sessions.id IS 'Session ID (UUID), stored in cookie';
COMMENT ON COLUMN sessions.user_id IS 'X/Twitter user ID';
COMMENT ON COLUMN sessions.username IS 'X/Twitter username';
COMMENT ON COLUMN sessions.access_token IS 'Encrypted OAuth 2.0 access token';
COMMENT ON COLUMN sessions.refresh_token IS 'Encrypted OAuth 2.0 refresh token (optional)';
COMMENT ON COLUMN sessions.created_at IS 'When the session was created';
COMMENT ON COLUMN sessions.expires_at IS 'When the session expires';

CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

COMMENT ON INDEX idx_sessions_expires_at IS 'Speed up cleanup of expired sessions';

-- Twitter users who have given or received good vibes
CREATE TABLE users (
    id         TEXT                      PRIMARY KEY,  -- Twitter user ID (used as primary key)
    username   TEXT                      NOT NULL,     -- Twitter username/handle
    name       TEXT                      NOT NULL,     -- Twitter display name
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL      -- When the Twitter account was created
);

COMMENT ON TABLE users IS 'Twitter users who have given or received good vibes';
COMMENT ON COLUMN users.id IS 'Twitter user ID, used as primary key';
COMMENT ON COLUMN users.username IS 'Twitter username/handle (e.g., @username)';
COMMENT ON COLUMN users.name IS 'Twitter display name';
COMMENT ON COLUMN users.created_at IS 'Timestamp when the Twitter account was created';

-- Records of good vibes relationships: emitter sends good vibes to sensor
-- Represents a directed edge in the good vibes graph (emitter -> sensor)
CREATE TABLE good_vibes (
    tweet_id   TEXT,                                    -- ID of the tweet containing the good vibes
    emitter_id TEXT                      NOT NULL REFERENCES users(id),  -- User sending good vibes
    sensor_id  TEXT                      NOT NULL REFERENCES users(id), -- User receiving good vibes
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL,      -- When the tweet was created
    PRIMARY KEY (emitter_id, sensor_id)                  -- One relationship per emitter-sensor pair
);

COMMENT ON TABLE good_vibes IS 'Records of good vibes relationships: emitter sends good vibes to sensor. Represents a directed edge in the good vibes graph (emitter -> sensor)';
COMMENT ON COLUMN good_vibes.tweet_id IS 'ID of the tweet containing the good vibes';
COMMENT ON COLUMN good_vibes.emitter_id IS 'User ID of the person sending good vibes (emitter)';
COMMENT ON COLUMN good_vibes.sensor_id IS 'User ID of the person receiving good vibes (sensor)';
COMMENT ON COLUMN good_vibes.created_at IS 'Timestamp when the tweet was created';

-- Records of megajoule transfers: sender sends amount megajoules to receiver
CREATE TABLE megajoule (
    tweet_id   TEXT,                                    -- ID of the tweet containing the megajoules
    sender_id  TEXT                      NOT NULL REFERENCES users(id),  -- User sending megajoules
    receiver_id TEXT                     NOT NULL REFERENCES users(id),  -- User receiving megajoules
    amount     INTEGER                   NOT NULL,      -- Amount of megajoules
    is_accepted BOOLEAN                  NOT NULL DEFAULT FALSE,  -- Whether the transfer has been accepted (to be implemented later)
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL,      -- When the tweet was created
    PRIMARY KEY (tweet_id)                              -- One record per tweet
);

COMMENT ON TABLE megajoule IS 'Records of megajoule transfers: sender sends amount megajoules to receiver';
COMMENT ON COLUMN megajoule.tweet_id IS 'ID of the tweet containing the megajoules transfer';
COMMENT ON COLUMN megajoule.sender_id IS 'User ID of the person sending megajoules';
COMMENT ON COLUMN megajoule.receiver_id IS 'User ID of the person receiving megajoules';
COMMENT ON COLUMN megajoule.amount IS 'Amount of megajoules transferred';
COMMENT ON COLUMN megajoule.is_accepted IS 'Whether the megajoule transfer has been accepted (to be implemented later)';
COMMENT ON COLUMN megajoule.created_at IS 'Timestamp when the tweet was created';

CREATE INDEX idx_megajoule_sender_id ON megajoule(sender_id);
CREATE INDEX idx_megajoule_receiver_id ON megajoule(receiver_id);
CREATE INDEX idx_megajoule_created_at ON megajoule(created_at);

COMMENT ON INDEX idx_megajoule_sender_id IS 'Index on sender_id column to speed up queries filtering by sender';
COMMENT ON INDEX idx_megajoule_receiver_id IS 'Index on receiver_id column to speed up queries filtering by receiver';
COMMENT ON INDEX idx_megajoule_created_at IS 'Index on created_at column to speed up time-based queries';

-- Tracks which tweets have been processed for vibe requests
CREATE TABLE vibe_requests (
    tweet_id TEXT PRIMARY KEY  -- Tweet ID that has been processed
);

COMMENT ON TABLE vibe_requests IS 'Tracks which tweets have been processed for vibe requests';
COMMENT ON COLUMN vibe_requests.tweet_id IS 'Tweet ID that has been processed for vibe requests';

-- Tracks materialized view refresh performance
CREATE TABLE vibe_materialize_time (
    id SERIAL PRIMARY KEY,
    degree INTEGER,  -- 1, 2, 3, 4, 5, or 6 (or NULL for combined view)
    refresh_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    time_taken_ms INTEGER NOT NULL  -- Time taken to refresh in milliseconds
);

COMMENT ON TABLE vibe_materialize_time IS 'Tracks materialized view refresh performance metrics';
COMMENT ON COLUMN vibe_materialize_time.degree IS 'Vibe degree (1, 2, 3, 4, 5, or 6) for the respective view, or NULL for combined view';
COMMENT ON COLUMN vibe_materialize_time.refresh_time IS 'Timestamp when the materialized view refresh completed';
COMMENT ON COLUMN vibe_materialize_time.time_taken_ms IS 'Time taken to refresh the materialized view in milliseconds';

CREATE INDEX idx_vibe_materialize_time_degree ON vibe_materialize_time(degree);
CREATE INDEX idx_vibe_materialize_time_refresh_time ON vibe_materialize_time(refresh_time);

COMMENT ON INDEX idx_vibe_materialize_time_degree IS 'Index on degree column to speed up queries filtering by degree';
COMMENT ON INDEX idx_vibe_materialize_time_refresh_time IS 'Index on refresh_time column to speed up time-based queries';

-- Human-readable view of good vibes with usernames instead of IDs
CREATE VIEW view_easy_good_vibes AS
SELECT
    gv.tweet_id,
    sensor.username  AS sensor_username,   -- Username of the receiver
    emitter.username AS emitter_username,  -- Username of the sender
    gv.created_at
FROM good_vibes gv
JOIN users emitter ON gv.emitter_id = emitter.id
JOIN users sensor  ON gv.sensor_id  = sensor.id
ORDER BY gv.created_at DESC;  -- Most recent first

COMMENT ON VIEW view_easy_good_vibes IS 'Human-readable view of good vibes with usernames instead of user IDs, ordered by creation date (most recent first)';

-- Direct connections (paths of length 1): emitter -> sensor
-- Counts how many times each direct relationship appears
CREATE MATERIALIZED VIEW view_good_vibes_degree_one AS
SELECT
    sensor_id,      -- Receiver of good vibes
    emitter_id,     -- Sender of good vibes
    COUNT(*) AS path_count  -- Number of direct connections (usually 1, but counts duplicates)
FROM good_vibes
WHERE emitter_id != sensor_id  -- Exclude self-loops
GROUP BY sensor_id, emitter_id;

COMMENT ON MATERIALIZED VIEW view_good_vibes_degree_one IS 'Direct connections (paths of length 1): emitter -> sensor. Counts how many times each direct relationship appears, excluding self-loops';

-- Human-readable view of degree one paths with usernames
CREATE VIEW view_easy_good_vibes_degree_one AS
SELECT sensor.username  AS sensor_username,
    emitter.username AS emitter_username,
    vgvdt.path_count AS degree_one_path_count
FROM view_good_vibes_degree_one vgvdt
JOIN users emitter ON vgvdt.emitter_id = emitter.id
JOIN users sensor  ON vgvdt.sensor_id  = sensor.id
ORDER BY degree_one_path_count DESC;  -- Highest counts first

COMMENT ON VIEW view_easy_good_vibes_degree_one IS 'Human-readable view of degree one paths with usernames, ordered by path count (highest first)';

-- Paths of length 2: sensor -> intermediate -> emitter
-- Finds all acyclic paths where emitter has good vibes for someone who has good vibes for sensor
CREATE MATERIALIZED VIEW view_good_vibes_degree_two AS
SELECT
    g1.sensor_id,   -- Starting point (sensor)
    g2.emitter_id,  -- End point (emitter)
    COUNT(*) AS path_count  -- Number of distinct 2-hop paths
FROM good_vibes g1
JOIN good_vibes g2 ON g1.emitter_id = g2.sensor_id  -- Connect: g1.emitter -> g2.sensor
WHERE g1.emitter_id != g1.sensor_id   -- No self-loops in first edge
  AND g2.emitter_id != g2.sensor_id   -- No self-loops in second edge
  AND g1.sensor_id  != g2.emitter_id  -- Ensure path is acyclic (sensor != emitter)
GROUP BY g1.sensor_id, g2.emitter_id;

COMMENT ON MATERIALIZED VIEW view_good_vibes_degree_two IS 'Paths of length 2: sensor -> intermediate -> emitter. Finds all acyclic paths where emitter has good vibes for someone who has good vibes for sensor';

-- Human-readable view of degree two paths with usernames
CREATE VIEW view_easy_good_vibes_degree_two AS
 SELECT sensor.username  AS sensor_username,
    emitter.username AS emitter_username,
    vgvdt.path_count AS degree_two_path_count
FROM view_good_vibes_degree_two vgvdt
JOIN users emitter ON vgvdt.emitter_id = emitter.id
JOIN users sensor  ON vgvdt.sensor_id  = sensor.id
ORDER BY degree_two_path_count DESC;  -- Highest counts first

COMMENT ON VIEW view_easy_good_vibes_degree_two IS 'Human-readable view of degree two paths with usernames, ordered by path count (highest first)';

-- Paths of length 3: sensor -> intermediate1 -> intermediate2 -> emitter
-- Finds all acyclic paths with two intermediate nodes
CREATE MATERIALIZED VIEW view_good_vibes_degree_three AS
SELECT
    g1.sensor_id,   -- Starting point (sensor)
    g3.emitter_id,  -- End point (emitter)
    COUNT(*) AS path_count  -- Number of distinct 3-hop paths
FROM good_vibes g1
JOIN good_vibes g2 ON g1.emitter_id = g2.sensor_id      -- First hop: g1.emitter -> g2.sensor
JOIN good_vibes g3 ON g2.emitter_id = g3.sensor_id      -- Second hop: g2.emitter -> g3.sensor
WHERE g1.emitter_id != g1.sensor_id   -- No self-loops in first edge
  AND g2.emitter_id != g2.sensor_id   -- No self-loops in second edge
  AND g3.emitter_id != g3.sensor_id   -- No self-loops in third edge
  AND g1.sensor_id  != g2.emitter_id  -- Ensure path is acyclic
  AND g1.sensor_id  != g3.emitter_id  -- Ensure path is acyclic
  AND g2.emitter_id != g3.emitter_id -- Ensure path is acyclic
GROUP BY g1.sensor_id, g3.emitter_id;

COMMENT ON MATERIALIZED VIEW view_good_vibes_degree_three IS 'Paths of length 3: sensor -> intermediate1 -> intermediate2 -> emitter. Finds all acyclic paths with two intermediate nodes';

-- Human-readable view of degree three paths with usernames
CREATE VIEW view_easy_good_vibes_degree_three AS
SELECT sensor.username  AS sensor_username,
    emitter.username AS emitter_username,
    vgvdt.path_count AS degree_three_path_count
FROM view_good_vibes_degree_three vgvdt
JOIN users emitter ON vgvdt.emitter_id = emitter.id
JOIN users sensor  ON vgvdt.sensor_id  = sensor.id
ORDER BY degree_three_path_count DESC;  -- Highest counts first

COMMENT ON VIEW view_easy_good_vibes_degree_three IS 'Human-readable view of degree three paths with usernames, ordered by path count (highest first)';

-- Paths of length 4: sensor -> intermediate1 -> intermediate2 -> intermediate3 -> emitter
-- Finds all acyclic paths with three intermediate nodes
CREATE MATERIALIZED VIEW view_good_vibes_degree_four AS
SELECT
    g1.sensor_id,   -- Starting point (sensor)
    g4.emitter_id,  -- End point (emitter)
    COUNT(*) AS path_count  -- Number of distinct 4-hop paths
FROM good_vibes g1
JOIN good_vibes g2 ON g1.emitter_id = g2.sensor_id      -- First hop: g1.emitter -> g2.sensor
JOIN good_vibes g3 ON g2.emitter_id = g3.sensor_id      -- Second hop: g2.emitter -> g3.sensor
JOIN good_vibes g4 ON g3.emitter_id = g4.sensor_id      -- Third hop: g3.emitter -> g4.sensor
WHERE g1.emitter_id != g1.sensor_id   -- No self-loops in first edge
  AND g2.emitter_id != g2.sensor_id   -- No self-loops in second edge
  AND g3.emitter_id != g3.sensor_id   -- No self-loops in third edge
  AND g4.emitter_id != g4.sensor_id   -- No self-loops in fourth edge
  AND g1.sensor_id  != g2.emitter_id   -- Ensure path is acyclic
  AND g1.sensor_id  != g3.emitter_id   -- Ensure path is acyclic
  AND g1.sensor_id  != g4.emitter_id   -- Ensure path is acyclic
  AND g2.emitter_id != g3.emitter_id   -- Ensure path is acyclic
  AND g2.emitter_id != g4.emitter_id   -- Ensure path is acyclic
  AND g3.emitter_id != g4.emitter_id   -- Ensure path is acyclic
GROUP BY g1.sensor_id, g4.emitter_id;

COMMENT ON MATERIALIZED VIEW view_good_vibes_degree_four IS 'Paths of length 4: sensor -> intermediate1 -> intermediate2 -> intermediate3 -> emitter. Finds all acyclic paths with three intermediate nodes';

-- Human-readable view of degree four paths with usernames
CREATE VIEW view_easy_good_vibes_degree_four AS
SELECT sensor.username  AS sensor_username,
    emitter.username AS emitter_username,
    vgvdt.path_count AS degree_four_path_count
FROM view_good_vibes_degree_four vgvdt
JOIN users emitter ON vgvdt.emitter_id = emitter.id
JOIN users sensor  ON vgvdt.sensor_id  = sensor.id
ORDER BY degree_four_path_count DESC;  -- Highest counts first

COMMENT ON VIEW view_easy_good_vibes_degree_four IS 'Human-readable view of degree four paths with usernames, ordered by path count (highest first)';

-- Paths of length 5: sensor -> intermediate1 -> intermediate2 -> intermediate3 -> intermediate4 -> emitter
-- Finds all acyclic paths with four intermediate nodes
CREATE MATERIALIZED VIEW view_good_vibes_degree_five AS
SELECT
    g1.sensor_id,   -- Starting point (sensor)
    g5.emitter_id,  -- End point (emitter)
    COUNT(*) AS path_count  -- Number of distinct 5-hop paths
FROM good_vibes g1
JOIN good_vibes g2 ON g1.emitter_id = g2.sensor_id      -- First hop: g1.emitter -> g2.sensor
JOIN good_vibes g3 ON g2.emitter_id = g3.sensor_id      -- Second hop: g2.emitter -> g3.sensor
JOIN good_vibes g4 ON g3.emitter_id = g4.sensor_id      -- Third hop: g3.emitter -> g4.sensor
JOIN good_vibes g5 ON g4.emitter_id = g5.sensor_id      -- Fourth hop: g4.emitter -> g5.sensor
WHERE g1.emitter_id != g1.sensor_id   -- No self-loops in first edge
  AND g2.emitter_id != g2.sensor_id   -- No self-loops in second edge
  AND g3.emitter_id != g3.sensor_id   -- No self-loops in third edge
  AND g4.emitter_id != g4.sensor_id   -- No self-loops in fourth edge
  AND g5.emitter_id != g5.sensor_id   -- No self-loops in fifth edge
  AND g1.sensor_id  != g2.emitter_id   -- Ensure path is acyclic
  AND g1.sensor_id  != g3.emitter_id   -- Ensure path is acyclic
  AND g1.sensor_id  != g4.emitter_id   -- Ensure path is acyclic
  AND g1.sensor_id  != g5.emitter_id   -- Ensure path is acyclic
  AND g2.emitter_id != g3.emitter_id   -- Ensure path is acyclic
  AND g2.emitter_id != g4.emitter_id   -- Ensure path is acyclic
  AND g2.emitter_id != g5.emitter_id   -- Ensure path is acyclic
  AND g3.emitter_id != g4.emitter_id   -- Ensure path is acyclic
  AND g3.emitter_id != g5.emitter_id   -- Ensure path is acyclic
  AND g4.emitter_id != g5.emitter_id   -- Ensure path is acyclic
GROUP BY g1.sensor_id, g5.emitter_id;

COMMENT ON MATERIALIZED VIEW view_good_vibes_degree_five IS 'Paths of length 5: sensor -> intermediate1 -> intermediate2 -> intermediate3 -> intermediate4 -> emitter. Finds all acyclic paths with four intermediate nodes';

-- Human-readable view of degree five paths with usernames
CREATE VIEW view_easy_good_vibes_degree_five AS
SELECT sensor.username  AS sensor_username,
    emitter.username AS emitter_username,
    vgvdt.path_count AS degree_five_path_count
FROM view_good_vibes_degree_five vgvdt
JOIN users emitter ON vgvdt.emitter_id = emitter.id
JOIN users sensor  ON vgvdt.sensor_id  = sensor.id
ORDER BY degree_five_path_count DESC;  -- Highest counts first

COMMENT ON VIEW view_easy_good_vibes_degree_five IS 'Human-readable view of degree five paths with usernames, ordered by path count (highest first)';

-- Paths of length 6: sensor -> intermediate1 -> intermediate2 -> intermediate3 -> intermediate4 -> intermediate5 -> emitter
-- Finds all acyclic paths with five intermediate nodes
CREATE MATERIALIZED VIEW view_good_vibes_degree_six AS
SELECT
    g1.sensor_id,   -- Starting point (sensor)
    g6.emitter_id,  -- End point (emitter)
    COUNT(*) AS path_count  -- Number of distinct 6-hop paths
FROM good_vibes g1
JOIN good_vibes g2 ON g1.emitter_id = g2.sensor_id      -- First hop: g1.emitter -> g2.sensor
JOIN good_vibes g3 ON g2.emitter_id = g3.sensor_id      -- Second hop: g2.emitter -> g3.sensor
JOIN good_vibes g4 ON g3.emitter_id = g4.sensor_id      -- Third hop: g3.emitter -> g4.sensor
JOIN good_vibes g5 ON g4.emitter_id = g5.sensor_id      -- Fourth hop: g4.emitter -> g5.sensor
JOIN good_vibes g6 ON g5.emitter_id = g6.sensor_id      -- Fifth hop: g5.emitter -> g6.sensor
WHERE g1.emitter_id != g1.sensor_id   -- No self-loops in first edge
  AND g2.emitter_id != g2.sensor_id   -- No self-loops in second edge
  AND g3.emitter_id != g3.sensor_id   -- No self-loops in third edge
  AND g4.emitter_id != g4.sensor_id   -- No self-loops in fourth edge
  AND g5.emitter_id != g5.sensor_id   -- No self-loops in fifth edge
  AND g6.emitter_id != g6.sensor_id   -- No self-loops in sixth edge
  AND g1.sensor_id  != g2.emitter_id   -- Ensure path is acyclic
  AND g1.sensor_id  != g3.emitter_id   -- Ensure path is acyclic
  AND g1.sensor_id  != g4.emitter_id   -- Ensure path is acyclic
  AND g1.sensor_id  != g5.emitter_id   -- Ensure path is acyclic
  AND g1.sensor_id  != g6.emitter_id   -- Ensure path is acyclic
  AND g2.emitter_id != g3.emitter_id   -- Ensure path is acyclic
  AND g2.emitter_id != g4.emitter_id   -- Ensure path is acyclic
  AND g2.emitter_id != g5.emitter_id   -- Ensure path is acyclic
  AND g2.emitter_id != g6.emitter_id   -- Ensure path is acyclic
  AND g3.emitter_id != g4.emitter_id   -- Ensure path is acyclic
  AND g3.emitter_id != g5.emitter_id   -- Ensure path is acyclic
  AND g3.emitter_id != g6.emitter_id   -- Ensure path is acyclic
  AND g4.emitter_id != g5.emitter_id   -- Ensure path is acyclic
  AND g4.emitter_id != g6.emitter_id   -- Ensure path is acyclic
  AND g5.emitter_id != g6.emitter_id   -- Ensure path is acyclic
GROUP BY g1.sensor_id, g6.emitter_id;

COMMENT ON MATERIALIZED VIEW view_good_vibes_degree_six IS 'Paths of length 6: sensor -> intermediate1 -> intermediate2 -> intermediate3 -> intermediate4 -> intermediate5 -> emitter. Finds all acyclic paths with five intermediate nodes';

-- Human-readable view of degree six paths with usernames
CREATE VIEW view_easy_good_vibes_degree_six AS
SELECT sensor.username  AS sensor_username,
    emitter.username AS emitter_username,
    vgvdt.path_count AS degree_six_path_count
FROM view_good_vibes_degree_six vgvdt
JOIN users emitter ON vgvdt.emitter_id = emitter.id
JOIN users sensor  ON vgvdt.sensor_id  = sensor.id
ORDER BY degree_six_path_count DESC;  -- Highest counts first

COMMENT ON VIEW view_easy_good_vibes_degree_six IS 'Human-readable view of degree six paths with usernames, ordered by path count (highest first)';

-- Combined view showing all degree paths (1-6) for each sensor-emitter pair
-- This is the main view used by the web application to display the full relationship graph
CREATE MATERIALIZED VIEW view_all_good_vibes_degrees AS
SELECT
    sensor.username AS sensor_username,      -- Username of the receiver
    sensor.name AS sensor_name,              -- Display name of the receiver
    emitter.username AS emitter_username,    -- Username of the sender
    emitter.name AS emitter_name,            -- Display name of the sender
    COALESCE(d1.path_count, 0) AS degree_one_path_count,    -- Direct connections (length 1)
    COALESCE(d2.path_count, 0) AS degree_two_path_count,   -- Two-hop paths (length 2)
    COALESCE(d3.path_count, 0) AS degree_three_path_count,  -- Three-hop paths (length 3)
    COALESCE(d4.path_count, 0) AS degree_four_path_count,    -- Four-hop paths (length 4)
    COALESCE(d5.path_count, 0) AS degree_five_path_count,    -- Five-hop paths (length 5)
    COALESCE(d6.path_count, 0) AS degree_six_path_count    -- Six-hop paths (length 6)
FROM (
    -- Collect all unique sensor-emitter pairs from all degree views
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_one
    UNION
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_two
    UNION
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_three
    UNION
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_four
    UNION
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_five
    UNION
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_six
) all_pairs
JOIN users sensor ON all_pairs.sensor_id = sensor.id      -- Get sensor user details
JOIN users emitter ON all_pairs.emitter_id = emitter.id  -- Get emitter user details
-- Left join each degree view to get path counts (NULL if no path exists at that degree)
LEFT JOIN view_good_vibes_degree_one d1 ON d1.sensor_id = all_pairs.sensor_id AND d1.emitter_id = all_pairs.emitter_id
LEFT JOIN view_good_vibes_degree_two d2 ON d2.sensor_id = all_pairs.sensor_id AND d2.emitter_id = all_pairs.emitter_id
LEFT JOIN view_good_vibes_degree_three d3 ON d3.sensor_id = all_pairs.sensor_id AND d3.emitter_id = all_pairs.emitter_id
LEFT JOIN view_good_vibes_degree_four d4 ON d4.sensor_id = all_pairs.sensor_id AND d4.emitter_id = all_pairs.emitter_id
LEFT JOIN view_good_vibes_degree_five d5 ON d5.sensor_id = all_pairs.sensor_id AND d5.emitter_id = all_pairs.emitter_id
LEFT JOIN view_good_vibes_degree_six d6 ON d6.sensor_id = all_pairs.sensor_id AND d6.emitter_id = all_pairs.emitter_id
ORDER BY sensor.username ASC, emitter.username ASC;  -- Alphabetical by sensor, then emitter

COMMENT ON MATERIALIZED VIEW view_all_good_vibes_degrees IS 'Combined view showing all degree paths (1-6) for each sensor-emitter pair. This is the main view used by the web application to display the full relationship graph, ordered alphabetically by sensor username then emitter username';

-- Create a database function to refresh all materialized views
-- This function uses SECURITY DEFINER to run with the privileges of the function owner,
-- allowing the reputest-rust-app user to refresh views owned by another role.
--
-- INSTRUCTIONS:
-- 1. Connect to your database as a superuser (postgres) or the owner of the materialized views
-- 2. Run this script to create the function
-- 3. The function will be owned by the user who creates it (should have permission to refresh the views)
-- 4. Grant EXECUTE permission to reputest-rust-app user
--
-- Usage from Rust code:
--   sqlx::query("SELECT refresh_all_materialized_views()").execute(pool).await?;

-- Drop the function if it already exists (for idempotency)
DROP FUNCTION IF EXISTS refresh_all_materialized_views();

-- Create the function with SECURITY DEFINER
-- This means the function runs with the privileges of the user who created it,
-- not the user who calls it
CREATE OR REPLACE FUNCTION refresh_all_materialized_views()
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    start_time TIMESTAMP WITH TIME ZONE;
    elapsed_ms INTEGER;
BEGIN
    -- Refresh degree 1 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_one;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (1, NOW(), elapsed_ms);

    -- Refresh degree 2 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_two;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (2, NOW(), elapsed_ms);

    -- Refresh degree 3 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_three;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (3, NOW(), elapsed_ms);

    -- Refresh degree 4 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_four;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (4, NOW(), elapsed_ms);

    -- Refresh degree 5 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_five;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (5, NOW(), elapsed_ms);

    -- Refresh degree 6 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_six;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (6, NOW(), elapsed_ms);

    -- Refresh combined view (record with degree=NULL)
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_all_good_vibes_degrees;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (NULL, NOW(), elapsed_ms);
END;
$$;

-- Grant execute permission to the application user
-- Replace 'reputest-rust-app' with your actual application database username if different
GRANT EXECUTE ON FUNCTION refresh_all_materialized_views() TO "reputest-rust-app";

-- Add a comment explaining the function
COMMENT ON FUNCTION refresh_all_materialized_views() IS 
'Refreshes all materialized views (degree 1-6 and combined view) and records timing metrics. '
'Uses SECURITY DEFINER to allow the application user to refresh views owned by another role.';

-- Indexes for performance optimization
CREATE INDEX idx_good_vibes_emitter_id   ON good_vibes(emitter_id);   -- Speed up queries filtering by emitter
CREATE INDEX idx_good_vibes_sensor_id    ON good_vibes(sensor_id);    -- Speed up queries filtering by sensor
CREATE INDEX idx_good_vibes_created_at   ON good_vibes(created_at);   -- Speed up time-based queries
CREATE INDEX idx_users_username          ON users(username);          -- Speed up username lookups
CREATE INDEX idx_users_id                ON users(id);                -- Speed up user ID lookups (though PK already indexed)
CREATE INDEX idx_refresh_tokens_token    ON refresh_tokens(token);    -- Speed up token lookups
CREATE INDEX idx_access_tokens_token     ON access_tokens(token);     -- Speed up token lookups

COMMENT ON INDEX idx_good_vibes_emitter_id IS 'Index on emitter_id column to speed up queries filtering by emitter';
COMMENT ON INDEX idx_good_vibes_sensor_id IS 'Index on sensor_id column to speed up queries filtering by sensor';
COMMENT ON INDEX idx_good_vibes_created_at IS 'Index on created_at column to speed up time-based queries';
COMMENT ON INDEX idx_users_username IS 'Index on username column to speed up username lookups';
COMMENT ON INDEX idx_users_id IS 'Index on id column to speed up user ID lookups (though primary key is already indexed)';
COMMENT ON INDEX idx_refresh_tokens_token IS 'Index on token column to speed up refresh token lookups';
COMMENT ON INDEX idx_access_tokens_token IS 'Index on token column to speed up access token lookups';