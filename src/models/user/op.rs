use crate::models::user;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ConnectionTrait, EntityTrait};

pub async fn upsert_user(
    user_basics: impl Into<user::ActiveModel>,
    db: &impl ConnectionTrait,
) -> anyhow::Result<()> {
    let user_model = user_basics.into();
    user::Entity::insert(user_model)
        .on_conflict(
            OnConflict::column(user::Column::Id)
                .update_columns([user::Column::Login, user::Column::DisplayName])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}
