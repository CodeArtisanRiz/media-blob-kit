use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RefreshToken::RefreshTokens)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RefreshToken::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RefreshToken::UserId).uuid().not_null())
                    .col(ColumnDef::new(RefreshToken::TokenHash).string().not_null().unique_key())
                    .col(ColumnDef::new(RefreshToken::ExpiresAt).timestamp().not_null())
                    .col(ColumnDef::new(RefreshToken::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(RefreshToken::Revoked).boolean().not_null().default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_refresh_token_user")
                            .from(RefreshToken::RefreshTokens, RefreshToken::UserId)
                            .to(User::Users, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RefreshToken::RefreshTokens).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RefreshToken {
    RefreshTokens,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    CreatedAt,
    Revoked,
}

#[derive(DeriveIden)]
enum User {
    Users,
    Id,
}
