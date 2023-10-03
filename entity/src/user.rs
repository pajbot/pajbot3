use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub login_last_updated: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::special_twitch_authorization::Entity")]
    SpecialTwitchAuthorization,
    #[sea_orm(has_many = "super::user_authorization::Entity")]
    UserAuthorization,
}

impl Related<super::special_twitch_authorization::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SpecialTwitchAuthorization.def()
    }
}

impl Related<super::user_authorization::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAuthorization.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
