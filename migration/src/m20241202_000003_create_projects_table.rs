use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Project::Projects)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Project::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Project::OwnerId).uuid().not_null())
                    .col(ColumnDef::new(Project::Name).string().not_null())
                    .col(ColumnDef::new(Project::Description).string())
                    .col(
                        ColumnDef::new(Project::Settings)
                            .json()
                            .not_null()
                            .default(SimpleExpr::Custom("'{}'".to_owned())),
                    )
                    .col(ColumnDef::new(Project::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Project::UpdatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Project::DeletedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_owner")
                            .from(Project::Projects, Project::OwnerId)
                            .to(User::Users, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Project::Projects).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Project {
    Projects,
    Id,
    OwnerId,
    Name,
    Description,
    Settings,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
enum User {
    Users,
    Id,
}
