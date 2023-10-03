use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "special_twitch_authorization")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: String,

    pub bot_scope_version: Option<i16>,
    pub broadcaster_scope_version: Option<i16>,

    pub twitch_access_token: String,
    pub twitch_refresh_token: String,
    pub valid_until: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
