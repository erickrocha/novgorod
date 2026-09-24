use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::{pk_auto, uuid_uniq};
use crate::m20260921_000001_create_person_table::Person;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Criar Tabela de Endereços (OrderAddress)
        manager
            .create_table(
                Table::create()
                    .table(OrderAddress::Table)
                    .if_not_exists()
                    .col(pk_auto(OrderAddress::Id).big_integer())
                    .col(uuid_uniq(OrderAddress::Id).not_null())
                    .col(ColumnDef::new(OrderAddress::AddressType).string().not_null()) // SHIPPING ou BILLING
                    .col(ColumnDef::new(OrderAddress::Street).string().not_null())
                    .col(ColumnDef::new(OrderAddress::Number).string().not_null())
                    .col(ColumnDef::new(OrderAddress::Complement).string())
                    .col(ColumnDef::new(OrderAddress::Neighborhood).string().not_null())
                    .col(ColumnDef::new(OrderAddress::City).string().not_null())
                    .col(ColumnDef::new(OrderAddress::State).string().not_null())
                    .col(ColumnDef::new(OrderAddress::Country).string().not_null())
                    .col(ColumnDef::new(OrderAddress::ZipCode).string().not_null())
                    .to_owned(),
            )
            .await?;

        // 2. Criar Tabela de Pedidos Principal (Order)
        manager
            .create_table(
                Table::create()
                    .table(Order::Table)
                    .if_not_exists()
                    .col(pk_auto(Order::Id).big_integer())
                    .col(uuid_uniq(Order::Id).not_null())
                    .col(ColumnDef::new(Order::CustomerId).uuid().not_null())
                    .col(ColumnDef::new(Order::CustomerName).string().not_null())
                    .col(ColumnDef::new(Order::CustomerEmail).string().not_null())
                    .col(ColumnDef::new(Order::CustomerTaxId).string().not_null()) // CPF/CNPJ
                    .col(ColumnDef::new(Order::CustomerPhone).string().not_null()) // CPF/CNPJ
                    .col(ColumnDef::new(Order::Status).string().not_null())
                    .col(ColumnDef::new(Order::SubtotalAmount).decimal_len(10, 2).not_null())
                    .col(ColumnDef::new(Order::ShippingAmount).decimal_len(10, 2).not_null())
                    .col(ColumnDef::new(Order::TotalAmount).decimal_len(10, 2).not_null())
                    .col(ColumnDef::new(Order::ShippingAddressId).uuid().not_null())
                    .col(ColumnDef::new(Order::BillingAddressId).uuid().not_null())
                    .col(ColumnDef::new(Order::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Order::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    // Chaves Estrangeiras para os Endereços
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order-shipping-address")
                            .from(Order::Table, Order::ShippingAddressId)
                            .to(OrderAddress::Table, OrderAddress::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order-billing-address")
                            .from(Order::Table, Order::BillingAddressId)
                            .to(OrderAddress::Table, OrderAddress::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. Criar Tabela de Itens do Pedido (OrderItem)
        manager
            .create_table(
                Table::create()
                    .table(OrderItem::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(OrderItem::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(OrderItem::OrderId).uuid().not_null())
                    .col(ColumnDef::new(OrderItem::ProductId).uuid().not_null())
                    .col(ColumnDef::new(OrderItem::Sku).string().not_null())
                    .col(ColumnDef::new(OrderItem::Name).string().not_null())
                    .col(ColumnDef::new(OrderItem::Quantity).integer().not_null())
                    .col(ColumnDef::new(OrderItem::UnitPrice).decimal_len(10, 2).not_null())
                    .col(ColumnDef::new(OrderItem::TotalPrice).decimal_len(10, 2).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-orderitem-order")
                            .from(OrderItem::Table, OrderItem::OrderId)
                            .to(Order::Table, Order::Id)
                            .on_delete(ForeignKeyAction::Cascade), // Se deletar o pedido, deleta os itens
                    )
                    .to_owned(),
            )
            .await?;

        // 4. Criar Tabela de Pagamentos (Payment)
        manager
            .create_table(
                Table::create()
                    .table(Payment::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Payment::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Payment::OrderId).uuid().not_null())
                    .col(ColumnDef::new(Payment::Method).string().not_null())
                    .col(ColumnDef::new(Payment::Status).string().not_null())
                    .col(ColumnDef::new(Payment::Installments).integer().not_null())
                    .col(ColumnDef::new(Payment::Amount).decimal_len(10, 2).not_null())
                    .col(ColumnDef::new(Payment::GatewayProvider).string().not_null())
                    .col(ColumnDef::new(Payment::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-payment-order")
                            .from(Payment::Table, Payment::OrderId)
                            .to(Order::Table, Order::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 5. Criar Tabela de Detalhes do Cartão Seguro (CreditCardDetails)
        manager
            .create_table(
                Table::create()
                    .table(CreditCardDetails::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CreditCardDetails::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(CreditCardDetails::PaymentId).uuid().not_null().unique_key()) // Relacionamento 1:1
                    .col(ColumnDef::new(CreditCardDetails::CardholderName).string().not_null())
                    .col(ColumnDef::new(CreditCardDetails::Brand).string().not_null())
                    .col(ColumnDef::new(CreditCardDetails::LastFourDigits).string_len(4).not_null())
                    .col(ColumnDef::new(CreditCardDetails::ExpirationMonth).integer().not_null())
                    .col(ColumnDef::new(CreditCardDetails::ExpirationYear).integer().not_null())
                    .col(ColumnDef::new(CreditCardDetails::CardToken).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-creditcard-payment")
                            .from(CreditCardDetails::Table, CreditCardDetails::PaymentId)
                            .to(Payment::Table, Payment::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 6. Criar Tabela de Transações (PaymentTransaction)
        manager
            .create_table(
                Table::create()
                    .table(PaymentTransaction::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PaymentTransaction::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(PaymentTransaction::PaymentId).uuid().not_null())
                    .col(ColumnDef::new(PaymentTransaction::GatewayTransactionId).string().not_null())
                    .col(ColumnDef::new(PaymentTransaction::AuthorizationCode).string())
                    .col(ColumnDef::new(PaymentTransaction::ResponseCode).string())
                    .col(ColumnDef::new(PaymentTransaction::ResponseMessage).string())
                    .col(ColumnDef::new(PaymentTransaction::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction-payment")
                            .from(PaymentTransaction::Table, PaymentTransaction::PaymentId)
                            .to(Payment::Table, Payment::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // A ordem de deleção deve ser inversa à ordem de criação para não violar as Foreign Keys
        manager.drop_table(Table::drop().table(PaymentTransaction::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(CreditCardDetails::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Payment::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(OrderItem::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Order::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(OrderAddress::Table).to_owned()).await?;

        Ok(())
    }
}

// -------------------------------------------------------------------------
// Enumerações DeriveIden (Identificadores para mapeamento de sintaxe do SeaORM)
// -------------------------------------------------------------------------

#[derive(DeriveIden)]
enum OrderAddress {
    Table,
    Id,
    AddressType,
    Street,
    Number,
    Complement,
    Neighborhood,
    City,
    State,
    Country,
    ZipCode,
}

#[derive(DeriveIden)]
enum Order {
    Table,
    Id,
    CustomerId,
    CustomerName,
    CustomerTaxId,
    CustomerPhone,
    CustomerEmail,
    Status,
    SubtotalAmount,
    ShippingAmount,
    TotalAmount,
    ShippingAddressId,
    BillingAddressId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum OrderItem {
    Table,
    Id,
    OrderId,
    ProductId,
    Sku,
    Name,
    Quantity,
    UnitPrice,
    TotalPrice,
}

#[derive(DeriveIden)]
enum Payment {
    Table,
    Id,
    OrderId,
    Method,
    Status,
    Installments,
    Amount,
    GatewayProvider,
    CreatedAt,
}

#[derive(DeriveIden)]
enum CreditCardDetails {
    Table,
    Id,
    PaymentId,
    CardholderName,
    Brand,
    LastFourDigits,
    ExpirationMonth,
    ExpirationYear,
    CardToken,
}

#[derive(DeriveIden)]
enum PaymentTransaction {
    Table,
    Id,
    PaymentId,
    GatewayTransactionId,
    AuthorizationCode,
    ResponseCode,
    ResponseMessage,
    CreatedAt,
}