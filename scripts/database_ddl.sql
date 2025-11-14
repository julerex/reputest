-- Connect using:
-- fly mpg connect
-- then run:
-- \connect reputest



CREATE TABLE refresh_tokens (
    id SERIAL PRIMARY KEY,
    token TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE access_tokens (
    id SERIAL PRIMARY KEY,
    token TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE good_vibes (
    tweet_id TEXT PRIMARY KEY,
    emitter_id TEXT NOT NULL REFERENCES users(id),
    sensor_id TEXT NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE vibe_requests (
    tweet_id TEXT PRIMARY KEY
);

CREATE VIEW good_vibes_view AS
SELECT
    gv.tweet_id,
    emitter.username AS emitter_username,
    sensor.username AS sensor_username,
    gv.created_at
FROM good_vibes gv
JOIN users emitter ON gv.emitter_id = emitter.id
JOIN users sensor ON gv.sensor_id = sensor.id
ORDER BY gv.created_at DESC;

CREATE INDEX idx_good_vibes_emitter_id ON good_vibes(emitter_id);
CREATE INDEX idx_good_vibes_sensor_id ON good_vibes(sensor_id);
CREATE INDEX idx_good_vibes_created_at ON good_vibes(created_at);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_id ON users(id);
CREATE INDEX idx_refresh_tokens_token ON refresh_tokens(token);
CREATE INDEX idx_access_tokens_token ON access_tokens(token);