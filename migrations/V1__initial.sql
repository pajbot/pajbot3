CREATE TABLE "user"
(
    id                 TEXT                     NOT NULL PRIMARY KEY,
    login              TEXT                     NOT NULL,
    display_name       TEXT                     NOT NULL,
    login_last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

CREATE FUNCTION trigger_user_update_login_last_updated()
    RETURNS trigger AS
$$
BEGIN
    NEW.login_last_updated = now();
    RETURN NEW;
END
$$
    LANGUAGE plpgsql;

CREATE TRIGGER user_login_update
    AFTER UPDATE OF login
    ON "user"
    FOR EACH ROW
EXECUTE PROCEDURE trigger_user_update_login_last_updated();

CREATE TYPE authorization_purpose AS ENUM ('bot_v1', 'broadcaster_v1');

CREATE TABLE special_twitch_authorization
(
    access_token  TEXT                     NOT NULL PRIMARY KEY,
    refresh_token TEXT                     NOT NULL,
    valid_until   TIMESTAMP WITH TIME ZONE NOT NULL,
    user_id       TEXT                     NOT NULL REFERENCES "user" (id),
    purpose       authorization_purpose    NOT NULL
);

-- for the purpose of letting users log into the website.
CREATE TABLE user_authorization
(
    access_token         TEXT                     NOT NULL PRIMARY KEY,
    twitch_access_token  TEXT                     NOT NULL,
    twitch_refresh_token TEXT                     NOT NULL,
    valid_until          TIMESTAMP WITH TIME ZONE NOT NULL,
    user_id              TEXT                     NOT NULL REFERENCES "user" (id)
);

CREATE TABLE bot
(
    broadcaster_id TEXT NOT NULL PRIMARY KEY REFERENCES "user" (id),
    name           TEXT NOT NULL
);
