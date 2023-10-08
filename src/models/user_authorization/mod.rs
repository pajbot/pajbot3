mod op;

pub use op::*;

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_authorization")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub access_token: String,
    pub twitch_access_token: String,
    pub twitch_refresh_token: String,
    pub valid_until: ChronoDateTimeUtc,
    pub user_id: String,
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

/// This type is used whenever a user authorization is handled that might be expired,
/// to ensure that the two are not accidentally mixed through the type system.
pub struct PossiblyExpired(pub Model);
