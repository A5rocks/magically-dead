CREATE TABLE games
(
    channel_id bigint NOT NULL,
    owner_id bigint NOT NULL,
    CONSTRAINT games_pkey PRIMARY KEY (channel_id)
);
