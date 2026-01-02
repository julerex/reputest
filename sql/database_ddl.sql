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

CREATE VIEW view_good_vibes_degree_one AS
SELECT
    sensor_id,
    emitter_id,
    COUNT(*) AS path_count
FROM good_vibes
WHERE emitter_id != sensor_id
GROUP BY sensor_id, emitter_id;

CREATE VIEW view_easy_good_vibes_degree_one AS
SELECT sensor.username  AS sensor_username,
    emitter.username AS emitter_username,
    vgvdt.path_count AS degree_one_path_count
FROM view_good_vibes_degree_one vgvdt
JOIN users emitter ON vgvdt.emitter_id = emitter.id
JOIN users sensor  ON vgvdt.sensor_id  = sensor.id
ORDER BY degree_one_path_count DESC;

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

CREATE VIEW view_good_vibes_degree_three AS
SELECT
    g1.sensor_id,
    g3.emitter_id,
    COUNT(*) AS path_count
FROM good_vibes g1
JOIN good_vibes g2 ON g1.emitter_id = g2.sensor_id
JOIN good_vibes g3 ON g2.emitter_id = g3.sensor_id
WHERE g1.emitter_id != g1.sensor_id
  AND g2.emitter_id != g2.sensor_id
  AND g3.emitter_id != g3.sensor_id
  AND g1.sensor_id  != g2.emitter_id
  AND g1.sensor_id  != g3.emitter_id
  AND g2.emitter_id != g3.emitter_id
GROUP BY g1.sensor_id, g3.emitter_id;

CREATE VIEW view_easy_good_vibes_degree_three AS
SELECT sensor.username  AS sensor_username,
    emitter.username AS emitter_username,
    vgvdt.path_count AS degree_three_path_count
FROM view_good_vibes_degree_three vgvdt
JOIN users emitter ON vgvdt.emitter_id = emitter.id
JOIN users sensor  ON vgvdt.sensor_id  = sensor.id
ORDER BY degree_three_path_count DESC;

CREATE VIEW view_good_vibes_degree_four AS
SELECT
    g1.sensor_id,
    g4.emitter_id,
    COUNT(*) AS path_count
FROM good_vibes g1
JOIN good_vibes g2 ON g1.emitter_id = g2.sensor_id
JOIN good_vibes g3 ON g2.emitter_id = g3.sensor_id
JOIN good_vibes g4 ON g3.emitter_id = g4.sensor_id
WHERE g1.emitter_id != g1.sensor_id
  AND g2.emitter_id != g2.sensor_id
  AND g3.emitter_id != g3.sensor_id
  AND g4.emitter_id != g4.sensor_id
  AND g1.sensor_id  != g2.emitter_id
  AND g1.sensor_id  != g3.emitter_id
  AND g1.sensor_id  != g4.emitter_id
  AND g2.emitter_id != g3.emitter_id
  AND g2.emitter_id != g4.emitter_id
  AND g3.emitter_id != g4.emitter_id
GROUP BY g1.sensor_id, g4.emitter_id;

CREATE VIEW view_easy_good_vibes_degree_four AS
SELECT sensor.username  AS sensor_username,
    emitter.username AS emitter_username,
    vgvdt.path_count AS degree_four_path_count
FROM view_good_vibes_degree_four vgvdt
JOIN users emitter ON vgvdt.emitter_id = emitter.id
JOIN users sensor  ON vgvdt.sensor_id  = sensor.id
ORDER BY degree_four_path_count DESC;

CREATE VIEW view_all_good_vibes_degrees AS
SELECT
    sensor.username AS sensor_username,
    emitter.username AS emitter_username,
    COALESCE(d1.path_count, 0) AS degree_one_path_count,
    COALESCE(d2.path_count, 0) AS degree_two_path_count,
    COALESCE(d3.path_count, 0) AS degree_three_path_count,
    COALESCE(d4.path_count, 0) AS degree_four_path_count
FROM (
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_one
    UNION
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_two
    UNION
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_three
    UNION
    SELECT sensor_id, emitter_id FROM view_good_vibes_degree_four
) all_pairs
JOIN users sensor ON all_pairs.sensor_id = sensor.id
JOIN users emitter ON all_pairs.emitter_id = emitter.id
LEFT JOIN view_good_vibes_degree_one d1 ON d1.sensor_id = all_pairs.sensor_id AND d1.emitter_id = all_pairs.emitter_id
LEFT JOIN view_good_vibes_degree_two d2 ON d2.sensor_id = all_pairs.sensor_id AND d2.emitter_id = all_pairs.emitter_id
LEFT JOIN view_good_vibes_degree_three d3 ON d3.sensor_id = all_pairs.sensor_id AND d3.emitter_id = all_pairs.emitter_id
LEFT JOIN view_good_vibes_degree_four d4 ON d4.sensor_id = all_pairs.sensor_id AND d4.emitter_id = all_pairs.emitter_id
ORDER BY sensor.username ASC;

-- Indexes
CREATE INDEX idx_good_vibes_emitter_id   ON good_vibes(emitter_id);
CREATE INDEX idx_good_vibes_sensor_id    ON good_vibes(sensor_id);
CREATE INDEX idx_good_vibes_created_at   ON good_vibes(created_at);
CREATE INDEX idx_users_username          ON users(username);
CREATE INDEX idx_users_id                ON users(id);
CREATE INDEX idx_refresh_tokens_token    ON refresh_tokens(token);
CREATE INDEX idx_access_tokens_token     ON access_tokens(token);