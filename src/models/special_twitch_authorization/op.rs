use crate::models::special_twitch_authorization;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ConnectionTrait, EntityTrait};

pub async fn upsert(
    special_twitch_authorization: impl Into<special_twitch_authorization::ActiveModel>,
    db: &impl ConnectionTrait,
) -> anyhow::Result<()> {
    let active_model = special_twitch_authorization.into();
    special_twitch_authorization::Entity::insert(active_model)
        .on_conflict(
            OnConflict::column(special_twitch_authorization::Column::UserId)
                .update_columns([
                    special_twitch_authorization::Column::BotScopeVersion,
                    special_twitch_authorization::Column::BroadcasterScopeVersion,
                    special_twitch_authorization::Column::TwitchAccessToken,
                    special_twitch_authorization::Column::TwitchRefreshToken,
                    special_twitch_authorization::Column::ValidUntil,
                ])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}
