CREATE TABLE public.players
(
    user_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    CONSTRAINT players_pkey PRIMARY KEY (user_id),
    CONSTRAINT game_exists FOREIGN KEY (channel_id)
        REFERENCES public.games (channel_id) MATCH SIMPLE
        ON UPDATE CASCADE
        ON DELETE CASCADE
);
