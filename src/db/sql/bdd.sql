-- Create the chat table
CREATE TABLE chat (
    id BIGINT PRIMARY KEY
);

-- Create the demands table with a composite primary key and foreign key reference to chat
CREATE TABLE demands (
    chat_id BIGINT NOT NULL,
    type_of VARCHAR NOT NULL,
    token VARCHAR,
    percentage SMALLINT,
    interval VARCHAR,
    CONSTRAINT fk_chat
        FOREIGN KEY (chat_id)
        REFERENCES chat(id)
        ON DELETE CASCADE,
    PRIMARY KEY (chat_id, type_of, token, percentage, interval)  -- Composite primary key with all columns
);

-- Create the tokens_at table with times as VARCHAR[] instead of TEXT[]
CREATE TABLE tokens_at (
    timestamp_in_min INTEGER PRIMARY KEY,
    times VARCHAR[],  -- Changed to VARCHAR[] for faster lookup and indexing
    tokens JSONB
);

-- Create an index on chat_id for demands for faster lookups
CREATE INDEX idx_demands_chat_id ON demands(chat_id);
