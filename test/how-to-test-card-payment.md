# Guia Passo a Passo: Como Testar o Pagamento com Cartão de Crédito (Novgorod)

Este documento descreve detalhadamente a arquitetura e como testar de ponta a ponta o fluxo de **pagamento com cartão de crédito** no ecossistema **Novgorod**.

---

## 📌 Situação do Desenvolvimento

**Sim, o pagamento com cartão de crédito está totalmente desenvolvido!**

A implementação conta com:
- **Backend (`kremlin`)**:
  - `GET /checkout/payment-config`: Retorna a chave pública (`MP_PUBLIC_KEY`) para inicialização segura do SDK.
  - `POST /purchases`: Criação atômica do registro de compra (`purchase`), reserva de estoque e pedidos associados (`orders`).
  - `POST /purchases/{id}/payments/submit`: Recebe o token seguro gerado pelo SDK, processa a cobrança via API do Mercado Pago, persiste os metadados do cartão (`credit_card_details`), atualiza status de pagamento e pedidos para `paid`/`captured`, e efetiva resgate de cupons.
  - `GET /purchases/{id}/payments/status`: Polling para conciliação de status de pagamento em análise.
  - `POST /webhooks/mercado-pago`: Endpoint de webhook com validação de assinatura criptográfica HMAC-SHA256 (`x-signature` e `x-request-id`).
  - `spawn_payment_reconciliation`: Rotina periódica em background que consulta e concilia pagamentos pendentes ou que sofreram timeout de rede.
- **Storefront (`torg`)**:
  - `MercadoPagoCardForm.tsx`: Integração com o SDK V2 do Mercado Pago via iframes seguros (`cardNumber`, `expirationDate`, `securityCode`), seleção de parcelas e emissor.
  - `CheckoutPage.tsx`: Fluxo completo guiado em 5 etapas (Dados do cliente, Endereço de entrega, Seleção de frete, Resumo com cupons e Pagamento com cartão).

---

## 📋 Sumário
1. [Testes Automatizados (Rápido)](#1-testes-automatizados-rápido)
2. [Configuração do Ambiente Local](#2-configuração-do-ambiente-local)
3. [Cartões de Teste (Mercado Pago Sandbox)](#3-cartões-de-teste-mercado-pago-sandbox)
4. [Cenário de Teste 1: Pagamento Aprovado no Storefront (`torg`)](#4-cenário-de-teste-1-pagamento-aprovado-no-storefront-torg)
5. [Cenário de Teste 2: Cartão Recusado / Saldo Insuficiente](#5-cenário-de-teste-2-cartão-recusado--saldo-insuficiente)
6. [Cenário de Teste 3: Compra com Total R$ 0,00 (Sem Cartão)](#6-cenário-de-teste-3-compra-com-total-r-000-sem-cartão)
7. [Cenário de Teste 4: Verificação no Banco de Dados](#7-cenário-de-teste-4-verificação-no-banco-de-dados)
8. [Cenário de Teste 5: Chamada Direta via API (cURL)](#8-cenário-de-teste-5-chamada-direta-via-api-curl)

---

## 1. Testes Automatizados (Rápido)

Para validar a integridade de todo o código sem precisar de credenciais reais ou abrir navegadores:

### 1.1 Storefront (`torg`)
```bash
cd torg
npm test -- src/pages/__tests__/CheckoutPage.test.tsx
```
> **Esperado**: 7 testes passando com sucesso, cobrindo cotação com parcelamento, formulário de cartão, compra com valor zero, troca de frete e validação de CPF.

### 1.2 Backend (`kremlin`)
```bash
cargo test --manifest-path kremlin/Cargo.toml -p business --test marketplace_postgres -- --nocapture
```
> **Esperado**: Testes de integração de `purchase`, `payment`, `payment_transaction` e `payment_allocation` aprovados.

---

## 2. Configuração do Ambiente Local

Para testar no navegador de forma real (com comunicação com o gateway em modo Sandbox):

### 2.1 Obter Credenciais de Teste do Mercado Pago
1. Acesse o painel de desenvolvedores: [Mercado Pago Developers](https://www.mercadopago.com.br/developers/panel).
2. Em **Suas integrações**, crie ou selecione uma aplicação de teste.
3. Copie as **Credenciais de Teste**:
   - `Public Key` (inicia com `TEST-...`)
   - `Access Token` (inicia com `TEST-...`)
   - `Collector ID` (ID numérico do usuário vendedor, visível nos detalhes da conta)
   - `Webhook Secret` (opcional para testes locais básicos, gerado na aba de Webhooks)

### 2.2 Configurar o `.env` do Backend (`kremlin/.env`)
Adicione as seguintes variáveis no arquivo `kremlin/.env`:
```dotenv
MP_PUBLIC_KEY=TEST-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
MP_ACCESS_TOKEN=TEST-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
MP_COLLECTOR_ID=123456789
MP_WEBHOOK_SECRET=sua_chave_secreta_webhook
```

### 2.3 Iniciar os Serviços
Em terminais separados:

- **Terminal 1 — Banco de Dados**:
  ```bash
  docker compose -f infra/dev/docker-compose.yml up -d
  ```

- **Terminal 2 — Backend (`kremlin`)**:
  ```bash
  cd kremlin
  cargo run -p application
  ```
  *(Servidor escutando em `http://localhost:8080`)*

- **Terminal 3 — Storefront (`torg`)**:
  ```bash
  cd torg
  npm run dev
  ```
  *(Acesse `http://localhost:5174` ou a porta indicada pelo Vite)*

---

## 3. Cartões de Teste (Mercado Pago Sandbox)

Utilize os dados oficiais do ambiente de testes do Mercado Pago para simular cada resultado:

| Cenário de Teste | Número do Cartão | Validade | CVV | Nome no Cartão | Resultado Esperado |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Aprovação Imediata** | `4066 6901 0203 0405` | Mês/Ano futuro (ex: `12/29`) | `123` | `APRO` | Pagamento aprovado (`captured`) |
| **Aprovação Alternativa** | `4242 4242 4242 4242` | Mês/Ano futuro (ex: `06/28`) | `123` | `APRO` | Pagamento aprovado (`captured`) |
| **Saldo Insuficiente** | `4066 6901 0203 0413` | Mês/Ano futuro (ex: `10/28`) | `123` | `FUND` | Pagamento recusado (`failed`) |
| **Recusado Geral** | `4066 6901 0203 0411` | Mês/Ano futuro (ex: `08/28`) | `123` | `CALL` | Pagamento recusado (`failed`) |
| **Em Processamento / Análise**| `4066 6901 0203 0421` | Mês/Ano futuro (ex: `05/29`) | `123` | `OTHE` | Status pendente (`pending`) |

*Nota: No campo CPF, informe qualquer CPF válido com 11 dígitos.*

---

## 4. Cenário de Teste 1: Pagamento Aprovado no Storefront (`torg`)

### Passo a Passo:
1. Abra o navegador em `http://localhost:5174`.
2. Faça login com uma conta de comprador (`Customer`) ou registre-se.
3. Navegue no catálogo, escolha um produto e clique em **"Adicionar ao carrinho"**.
4. Acesse o **Carrinho** e clique em **"Finalizar Compra"** (redireciona para `/checkout`).
5. **Etapa 1 (Seus dados)**:
   - Se o cliente já tiver CPF no cadastro, o campo estará preenchido. Caso contrário, informe o CPF e confirme.
6. **Etapa 2 (Endereço)**:
   - Selecione um endereço existente ou cadastre um novo (ex: CEP `01310-100`).
7. **Etapa 3 (Cotação e Frete)**:
   - Clique em **"Calcular total com frete"**.
   - As opções de frete (ex: SEDEX, PAC ou Fixo) serão calculadas e exibidas.
8. **Etapa 4 (Pagamento com Cartão)**:
   - A opção **Cartão de crédito** estará selecionada.
   - O formulário seguro do Mercado Pago será renderizado:
     - **Número do Cartão**: Digite `4066 6901 0203 0405`.
     - **Validade**: Digite `12/29`.
     - **Código de Segurança**: Digite `123`.
     - **Titular do cartão**: Digite `APRO`.
     - **Banco emissor**: Selecione o banco sugerido na lista suspensa.
     - **Parcelas**: Selecione `1x` (ou a quantidade de parcelas desejada).
9. Clique no botão **"Pagar com cartão de crédito"**.

### Resultado Esperado:
- O botão exibirá *"Enviando pagamento…"*.
- O SDK gera o token criptografado e envia ao backend em `/purchases/{id}/payments/submit`.
- O backend processa a transação com a API do Mercado Pago e recebe confirmação (`approved`).
- A tela exibe imediatamente o bloco de confirmação:
  - Ícone verde de confirmação com título **"Pagamento confirmado"**.
  - Texto: *"Compra #X confirmada com sucesso. Pedidos: ORD-Y."*.
  - Botão *"Voltar ao catálogo"*.

---

## 5. Cenário de Teste 2: Cartão Recusado / Saldo Insuficiente

### Passo a Passo:
1. Siga os passos 1 a 7 do Cenário 1.
2. No formulário de cartão, preencha:
   - **Número do Cartão**: `4066 6901 0203 0413` *(Cartão de teste para rejeição)*.
   - **Validade**: `12/29`.
   - **Código de Segurança**: `123`.
   - **Titular do cartão**: `FUND`.
3. Clique em **"Pagar com cartão de crédito"**.

### Resultado Esperado:
- O gateway rejeita a transação.
- O backend marca o status do pagamento como `failed` e atualiza a compra para `payment_failed`.
- O checkout exibe o aviso em vermelho:
  > **"Pagamento recusado. Confira os dados e tente outro cartão."**
- Os campos do formulário são reabilitados para permitir nova tentativa com outro cartão.

---

## 6. Cenário de Teste 3: Compra com Total R$ 0,00 (Sem Cartão)

O sistema possui uma regra de negócio inteligente para pedidos 100% cobertos por cupom promocional:

1. No checkout, aplique um cupom de desconto que cubra o valor total dos produtos e frete.
2. O total a pagar é recalculado para **R$ 0,00**.
3. O formulário do Mercado Pago **não é exibido**.
4. É apresentado o botão **"Concluir pedido gratuito"**.
5. Ao clicar, o pedido é criado diretamente com status `paid`/`captured`, sem necessidade de dados de pagamento.

---

## 7. Cenário de Teste 4: Verificação no Banco de Dados

Após concluir uma compra paga com sucesso, conecte-se ao banco de dados:

```bash
docker exec -it novgorod_postgres psql -U novgorod-dev -d novgorod-dev
```

Execute a consulta abaixo para auditar os registros gerados:

```sql
SELECT 
    p.id AS purchase_id,
    p.status AS purchase_status,
    p.total_cents,
    pay.id AS payment_id,
    pay.status AS payment_status,
    pay.method,
    pay.gateway_provider,
    pay.gateway_reference,
    cc.brand,
    cc.last_four_digits,
    cc.cardholder_name
FROM purchase p
JOIN payment pay ON pay.purchase_id = p.id
LEFT JOIN credit_card_details cc ON cc.payment_id = pay.id
ORDER BY p.id DESC
LIMIT 1;
```

### O que você deve ver:
- `purchase_status`: `'paid'`
- `payment_status`: `'captured'`
- `method`: `'credit_card'`
- `gateway_provider`: `'mercado_pago'`
- `gateway_reference`: ID da transação no Mercado Pago (ex: `'1234567890'`)
- `brand`: `'visa'` ou `'master'`
- `last_four_digits`: `'0405'`
- `cardholder_name`: `'APRO'`

Para verificar a geração dos pedidos por lojista:
```sql
SELECT id, number, tenant_id, status, payment_status, total_cents
FROM orders
WHERE purchase_id = (SELECT MAX(id) FROM purchase);
```
- Cada pedido terá `status = 'paid'` e `payment_status = 'captured'`.

---

## 8. Cenário de Teste 5: Chamada Direta via API (cURL)

Se desejar testar a camada de endpoints REST diretamente:

### 8.1 Consultar Configuração da Chave Pública
```bash
curl -s -X GET http://localhost:8080/checkout/payment-config \
  -H "Authorization: Bearer <SEU_JWT_CUSTOMER>"
```
**Resposta esperada (200 OK):**
```json
{
  "publicKey": "TEST-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
}
```

### 8.2 Submeter Pagamento de uma Compra Existente
```bash
curl -s -X POST http://localhost:8080/purchases/1/payments/submit \
  -H "Authorization: Bearer <SEU_JWT_CUSTOMER>" \
  -H "Content-Type: application/json" \
  -d '{
    "token": "token_gerado_pelo_sdk_mp",
    "paymentMethodId": "visa",
    "issuerId": "310",
    "installments": 1
  }'
```
**Resposta esperada (200 OK):**
```json
{
  "purchaseId": 1,
  "status": "captured",
  "reference": "9876543210"
}
```
