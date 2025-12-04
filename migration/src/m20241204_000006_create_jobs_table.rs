use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Jobs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Jobs::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Jobs::FileId).uuid().not_null())
                    .col(ColumnDef::new(Jobs::Status).string().not_null())
                    .col(ColumnDef::new(Jobs::Payload).json().not_null())
                    .col(ColumnDef::new(Jobs::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Jobs::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_jobs_file_id")
                            .from(Jobs::Table, Jobs::FileId)
                            .to(Files::Table, Files::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Jobs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Jobs {
    Table,
    Id,
    FileId,
    Status,
    Payload,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Files {
    Table,
    Id,
}
