use sea_orm::entity::prelude::*;
use sea_orm::entity::LinkDef;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bot")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub broadcaster_id: String,
    pub bot_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::BroadcasterId",
        to = "super::user::Column::Id"
    )]
    Broadcaster,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::BotId",
        to = "super::user::Column::Id"
    )]
    Bot,
}

// Can't use "impl Related" here, since there are two relations to the "user" table.
impl Linked for Entity {
    type FromEntity = Entity;
    type ToEntity = super::user::Entity;

    fn link(&self) -> Vec<LinkDef> {
        vec![Relation::Broadcaster.def(), Relation::Bot.def()]
    }
}

impl ActiveModelBehavior for ActiveModel {}
