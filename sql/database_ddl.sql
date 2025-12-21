CREATE TABLE refresh_tokens (
    id         SERIAL                    PRIMARY KEY,
    token      TEXT                      NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL DEFAULT NOW()
);

CREATE TABLE access_tokens (
    id         SERIAL                    PRIMARY KEY,
    token      TEXT                      NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL DEFAULT NOW()
);

CREATE TABLE users (
    id         TEXT                      PRIMARY KEY,
    username   TEXT                      NOT NULL,
    name       TEXT                      NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL
);

CREATE TABLE good_vibes (
    tweet_id   TEXT,
    emitter_id TEXT                      NOT NULL REFERENCES users(id),
    sensor_id  TEXT                      NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE  NOT NULL,
    PRIMARY KEY (emitter_id, sensor_id)
);

CREATE TABLE vibe_requests (
    tweet_id TEXT PRIMARY KEY
);

CREATE VIEW view_easy_good_vibes AS
SELECT
    gv.tweet_id,
    sensor.username  AS sensor_username,
    emitter.username AS emitter_username,
    gv.created_at
FROM good_vibes gv
JOIN users emitter ON gv.emitter_id = emitter.id
JOIN users sensor  ON gv.sensor_id  = sensor.id
ORDER BY gv.created_at DESC;

CREATE VIEW view_good_vibes_degree_two AS
SELECT
    g1.sensor_id,
    g2.emitter_id,
    COUNT(*) AS path_count
FROM good_vibes g1
JOIN good_vibes g2 ON g1.emitter_id = g2.sensor_id
WHERE g1.emitter_id != g1.sensor_id
  AND g2.emitter_id != g2.sensor_id
  AND g1.sensor_id  != g2.emitter_id
GROUP BY g1.sensor_id, g2.emitter_id;

CREATE VIEW view_easy_good_vibes_degree_two AS
 SELECT sensor.username  AS sensor_username,
    emitter.username AS emitter_username,
    vgvdt.path_count AS degree_two_path_count
FROM view_good_vibes_degree_two vgvdt
JOIN users emitter ON vgvdt.emitter_id = emitter.id
JOIN users sensor  ON vgvdt.sensor_id  = sensor.id
ORDER BY degree_two_path_count DESC;

-- Indexes
CREATE INDEX idx_good_vibes_emitter_id   ON good_vibes(emitter_id);
CREATE INDEX idx_good_vibes_sensor_id    ON good_vibes(sensor_id);
CREATE INDEX idx_good_vibes_created_at   ON good_vibes(created_at);
CREATE INDEX idx_users_username          ON users(username);
CREATE INDEX idx_users_id                ON users(id);
CREATE INDEX idx_refresh_tokens_token    ON refresh_tokens(token);
CREATE INDEX idx_access_tokens_token     ON access_tokens(token);