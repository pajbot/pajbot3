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

-- Twitch Authorization existing for the purpose of:
-- (if bot_scope_version IS NOT NULL) joining Twitch chat as this user (i.e. operating under this bot username)
-- (if broadcaster_scope_version IS NOT NULL) operating as a Bot in this Twitch channel.
-- This determines what scope the token has. Each scope version maps
-- to a set of requested scopes. The token is expected to have the scope
-- (bot_scope_version IS NOT NULL ? bot_scope(bot_scope_version) : {}) UNION
-- (broadcaster_scope_version IS NOT NULL ? broadcaster_scope(broadcaster_scope_version) : {})
--
-- The version is maintained, since Twitch tends to evolve their API and likes to introduce
-- new scopes sometimes which we need new authorization for, even if we want to keep
-- the amount of data accessed via the Twitch API constant.
--
-- At least one of bot_scope_version or broadcaster_scope_version has to be NOT NULL.
CREATE TABLE special_twitch_authorization
(
    user_id                   TEXT                     NOT NULL REFERENCES "user" (id) PRIMARY KEY,
    bot_scope_version         SMALLINT, -- intentionally nullable
    broadcaster_scope_version SMALLINT, -- intentionally nullable
    twitch_access_token       TEXT                     NOT NULL,
    twitch_refresh_token      TEXT                     NOT NULL,
    valid_until               TIMESTAMP WITH TIME ZONE NOT NULL,
    CHECK(bot_scope_version IS NOT NULL OR broadcaster_scope_version IS NOT NULL)
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

-- TODO fill out more
CREATE TABLE bot
(
    broadcaster_id TEXT NOT NULL PRIMARY KEY REFERENCES "user" (id),
    bot_id         TEXT NOT NULL REFERENCES "user"(id)
);
