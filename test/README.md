# Novgorod — Test Suite & Verification Guides

Este diretório contém os roteiros de teste, guias de validação passo a passo e documentações de verificação dos módulos do ecossistema **Novgorod**.

## 📖 Guias Disponíveis

- [**Guia de Teste de Frete (Passo a Passo)**](./how-to-test-shipping.md)
  - Roteiro completo de validação da funcionalidade de frete dos Correios e Frete Fixo.
  - Comandos para execução de testes unitários e de integração no backend (`kremlin`).
  - Passo a passo para testar a tela de configuração do lojista no painel (`veche`).
  - Passo a passo para testar a cotação e seleção de frete (PAC/SEDEX) na loja (`torg`).
  - Testes de casos de erro, limites de pacotes e chamadas diretas via cURL.

- [**Guia de Teste de Pagamento com Cartão de Crédito (Mercado Pago)**](./how-to-test-card-payment.md)
  - Roteiro completo de validação da integração com Mercado Pago (SDK transparente).
  - Tabela com números de cartões de teste do Mercado Pago Sandbox (Aprovação, Recusa, Análise).
  - Passo a passo para testar o checkout de ponta a ponta no Storefront (`torg`).
  - Validação de regras de negócio (isenção de cartão para total R$ 0,00, persistência de metadados).
  - Consultas SQL para auditoria no banco de dados e testes via cURL.

- [**Guia de Teste de Pagamento com PagSeguro / PagBank (Multi-Tenant)**](./how-to-test-pagseguro-payment.md)
  - Roteiro completo de validação da abstração multi-tenant de pagamento e do gateway PagSeguro.
  - Passo a passo para configurar o gateway preferido e credenciais no painel (`veche`).
  - Tabela com cartões de teste do PagSeguro / PagBank Sandbox (Aprovação, Recusa).
  - Passo a passo para testar o checkout com formulário do PagSeguro no Storefront (`torg`).
  - Simulação de Webhook assíncrono do PagBank e rotinas de conciliação.
  - Consultas SQL de auditoria de credenciais cifradas (AES-256-GCM) e pagamentos.
