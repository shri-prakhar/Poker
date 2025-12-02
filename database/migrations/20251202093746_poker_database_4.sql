-- Add migration script here
-- Add migration script here
-- Add migration script here
-- Add migration script here
-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users(
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    email TEXT NOT NULL,
    hashed_password TEXT NOT NULL,
    display_name TEXT, 
    created_at timestamptz NOT NULL DEFAULT now()     
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(lower(email));

CREATE TABLE IF NOT EXISTS user_sessions(
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_name TEXT , 
    created_at timestamptz NOT NULL DEFAULT now(),
    last_seen timestamptz
);

CREATE INDEX IF NOT EXISTS idx_sessions_user on user_sessions(user_id);

CREATE TABLE IF NOT EXISTS refresh_tokens(
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    expires_at timestamptz NOT NULL, 
    revoked boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_refresh_user on refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_token_hash on refresh_tokens(token_hash);

CREATE TABLE IF NOT EXISTS rooms (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    room_name TEXT,
    host_user_id uuid REFERENCES users(id),
    room_status TEXT NOT NULL DEFAULT 'waiting', --waiting | playing | finished
    max_players smallint DEFAULT 6,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_room_status ON rooms(room_status);

CREATE TABLE IF NOT EXISTS room_players(
    room_id uuid NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    seat smallint NOT NULL,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    chips BIGINT NOT NULL DEFAULT 0,
    connected boolean DEFAULT true,
    is_dealer boolean DEFAULT false,
    PRIMARY KEY(room_id , seat) 
);

CREATE INDEX IF NOT EXISTS idx_room_players_user ON room_players(user_id);

CREATE TABLE IF NOT EXISTS hands(
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    room_id uuid REFERENCES rooms(id),
    started_at timestamptz,
    finished_at timestamptz,
    pot BIGINT NOT NULL DEFAULT 0,
    board jsonb, --community cards  
    winner_user_id uuid,
    result jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_hands_room ON hands(room_id);

CREATE TABLE IF NOT EXISTS actions(
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    hand_id uuid REFERENCES hands(id) ON DELETE CASCADE,
    user_id uuid REFERENCES users(id),
    action_type TEXT NOT NULL, -- bet | call | fold | check | raise  | all-in | timeout
    amount BIGINT DEFAULT 0,
    created_at timestamptz DEFAULT now() 
);

CREATE INDEX IF NOT EXISTS idx_action_hand ON actions(hand_id);

CREATE TABLE IF NOT EXISTS hand_players(
    hand_id uuid REFERENCES hands(id) ON DELETE CASCADE,
    seat smallint NOT NULL,
    user_id uuid REFERENCES users(id),
    hole_cards jsonb,
    chips_before BIGINT,
    chips_after BIGINT,
    PRIMARY KEY (hand_id , seat)
);

CREATE INDEX IF NOT EXISTS idx_hands_players_user ON hand_players(user_id);
