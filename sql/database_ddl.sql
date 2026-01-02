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

-- Tracks which tweets have been processed for vibe requests
CREATE TABLE vibe_requests (
    tweet_id TEXT PRIMARY KEY  -- Tweet ID that has been processed
);

COMMENT ON TABLE vibe_requests IS 'Tracks which tweets have been processed for vibe requests';
COMMENT ON COLUMN vibe_requests.tweet_id IS 'Tweet ID that has been processed for vibe requests';

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
CREATE VIEW view_good_vibes_degree_one AS
SELECT
    sensor_id,      -- Receiver of good vibes
    emitter_id,     -- Sender of good vibes
    COUNT(*) AS path_count  -- Number of direct connections (usually 1, but counts duplicates)
FROM good_vibes
WHERE emitter_id != sensor_id  -- Exclude self-loops
GROUP BY sensor_id, emitter_id;

COMMENT ON VIEW view_good_vibes_degree_one IS 'Direct connections (paths of length 1): emitter -> sensor. Counts how many times each direct relationship appears, excluding self-loops';

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
CREATE VIEW view_good_vibes_degree_two AS
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

COMMENT ON VIEW view_good_vibes_degree_two IS 'Paths of length 2: sensor -> intermediate -> emitter. Finds all acyclic paths where emitter has good vibes for someone who has good vibes for sensor';

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
CREATE VIEW view_good_vibes_degree_three AS
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

COMMENT ON VIEW view_good_vibes_degree_three IS 'Paths of length 3: sensor -> intermediate1 -> intermediate2 -> emitter. Finds all acyclic paths with two intermediate nodes';

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
CREATE VIEW view_good_vibes_degree_four AS
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

COMMENT ON VIEW view_good_vibes_degree_four IS 'Paths of length 4: sensor -> intermediate1 -> intermediate2 -> intermediate3 -> emitter. Finds all acyclic paths with three intermediate nodes';

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

-- Combined view showing all degree paths (1-4) for each sensor-emitter pair
-- This is the main view used by the web application to display the full relationship graph
CREATE VIEW view_all_good_vibes_degrees AS
SELECT
    sensor.username AS sensor_username,      -- Username of the receiver
    sensor.name AS sensor_name,              -- Display name of the receiver
    emitter.username AS emitter_username,    -- Username of the sender
    emitter.name AS emitter_name,            -- Display name of the sender
    COALESCE(d1.path_count, 0) AS degree_one_path_count,    -- Direct connections (length 1)
    COALESCE(d2.path_count, 0) AS degree_two_path_count,   -- Two-hop paths (length 2)
    COALESCE(d3.path_count, 0) AS degree_three_path_count,  -- Three-hop paths (length 3)
    COALESCE(d4.path_count, 0) AS degree_four_path_count    -- Four-hop paths (length 4)
FROM (
    -- Collect all unique sensor-emitter pairs from all degree views
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_one
    UNION
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_two
    UNION
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_three
    UNION
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_four
) all_pairs
JOIN users sensor ON all_pairs.sensor_id = sensor.id      -- Get sensor user details
JOIN users emitter ON all_pairs.emitter_id = emitter.id  -- Get emitter user details
-- Left join each degree view to get path counts (NULL if no path exists at that degree)
LEFT JOIN view_good_vibes_degree_one d1 ON d1.sensor_id = all_pairs.sensor_id AND d1.emitter_id = all_pairs.emitter_id
LEFT JOIN view_good_vibes_degree_two d2 ON d2.sensor_id = all_pairs.sensor_id AND d2.emitter_id = all_pairs.emitter_id
LEFT JOIN view_good_vibes_degree_three d3 ON d3.sensor_id = all_pairs.sensor_id AND d3.emitter_id = all_pairs.emitter_id
LEFT JOIN view_good_vibes_degree_four d4 ON d4.sensor_id = all_pairs.sensor_id AND d4.emitter_id = all_pairs.emitter_id
ORDER BY sensor.username ASC, emitter.username ASC;  -- Alphabetical by sensor, then emitter

COMMENT ON VIEW view_all_good_vibes_degrees IS 'Combined view showing all degree paths (1-4) for each sensor-emitter pair. This is the main view used by the web application to display the full relationship graph, ordered alphabetically by sensor username then emitter username';

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