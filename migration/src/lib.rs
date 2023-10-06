pub use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct RawSqlMigration {
    up_sql: &'static str,
    down_sql: &'static str,
}

#[async_trait::async_trait]
impl MigrationTrait for RawSqlMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(self.up_sql)
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(self.down_sql)
            .await?;
        Ok(())
    }
}

macro_rules! raw_sql_migration {
    ($e:expr) => {
        Box::new(RawSqlMigration {
            up_sql: include_str!(concat!($e, "_up.sql")),
            down_sql: include_str!(concat!($e, "_down.sql")),
        })
    };
}

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![raw_sql_migration!("m20231003_143225_initial")]
    }
}
