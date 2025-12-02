use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApiKey::ApiKeys)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ApiKey::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ApiKey::ProjectId).uuid().not_null())
                    .col(ColumnDef::new(ApiKey::Name).string().not_null())
                    .col(ColumnDef::new(ApiKey::KeyHash).string().not_null().unique_key())
                    .col(ColumnDef::new(ApiKey::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(ApiKey::ExpiresAt).timestamp())
                    .col(
                        ColumnDef::new(ApiKey::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_api_key_project")
                            .from(ApiKey::ApiKeys, ApiKey::ProjectId)
                            .to(Project::Projects, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiKey::ApiKeys).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ApiKey {
    ApiKeys,
    Id,
    ProjectId,
    Name,
    KeyHash,
    CreatedAt,
    ExpiresAt,
    IsActive,
}

#[derive(DeriveIden)]
enum Project {
    Projects,
    Id,
}
