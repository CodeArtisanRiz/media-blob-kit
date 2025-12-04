use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Files::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Files::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Files::ProjectId).uuid().not_null())
                    .col(ColumnDef::new(Files::S3Key).string().not_null().unique_key())
                    .col(ColumnDef::new(Files::Filename).string().not_null())
                    .col(ColumnDef::new(Files::MimeType).string().not_null())
                    .col(ColumnDef::new(Files::Size).big_integer().not_null())
                    .col(ColumnDef::new(Files::Status).string().not_null())
                    .col(ColumnDef::new(Files::VariantsJson).json().not_null())
                    .col(ColumnDef::new(Files::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Files::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_files_project_id")
                            .from(Files::Table, Files::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Files::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Files {
    Table,
    Id,
    ProjectId,
    S3Key,
    Filename,
    MimeType,
    Size,
    Status,
    VariantsJson,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}
