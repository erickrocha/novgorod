# Guia Passo a Passo: Como Testar o Pagamento com PagSeguro (Novgorod)

Este documento descreve detalhadamente a arquitetura de **abstração de gateways de pagamento multi-tenant** e como configurar e testar o **PagSeguro (PagBank)** de ponta a ponta no ecossistema **Novgorod**.

---

## 📌 Situação do Desenvolvimento

A integração com o **PagSeguro (PagBank)** e o sistema de seleção de gateway por lojista (*tenant*) estão totalmente implementados:

1. **Abstração Multi-Tenant no Backend (`kremlin`)**:
   - `tenant_payment_settings`: Tabela com isolamento por tenant contendo provedor ativo (`mercado_pago` ou `pagseguro`), credenciais criptografadas via **AES-256-GCM** com chave de chaveiro versionada (`payment:{tenant_id}:{version}`) e metadados.
   - `PaymentProviderGateway`: Trait comum implementada para `MercadoPago` e `PagSeguro`, padronizando `charge`, `status`, `search` e validação.
   - `GET /tenants/{tenantId}/payment-settings` e `PUT /tenants/{tenantId}/payment-settings`: Endpoints para gestão de credenciais e seleção de gateway com controle de acesso RBAC (`SysAdmin` ou `TenantOwner`).
   - `GET /checkout/payment-config?tenantId=X`: Retorna dinamicamente o provedor ativo do lojista (`pagseguro` ou `mercado_pago`) e a chave pública correspondente.
   - `POST /purchases/{id}/payments/submit`: Submissão de pagamento agnóstica a gateway (suporta tokens de cartão criptografados do PagSeguro / PagBank).
   - `POST /webhooks/pagseguro`: Endpoint para recebimento de notificações assíncronas do PagBank.
   - `spawn_payment_reconciliation`: Rotina periódica em background que consulta e concilia pagamentos pendentes para ambos os provedores.

2. **Storefront (`torg`)**:
   - `PagSeguroCardForm.tsx`: Componente com integração transparente do SDK do PagBank (`window.PagSeguro.encryptCard`), seleção de parcelas e criptografia de ponta a ponta de dados sensíveis de cartão de crédito.
   - `CheckoutPage.tsx`: Alternância dinâmica entre formulários (`MercadoPagoCardForm` vs `PagSeguroCardForm`) baseada na configuração retornada pelo lojista.

3. **Painel do Lojista (`veche`)**:
   - Menu **Operações > Configurações de Pagamento** (`/operations/payment-settings`).
   - Configuração do gateway preferido com suporte a credenciais Sandbox/Produção, chave pública e tokens com proteção visual e persistência segura.

---

## 📋 Sumário
1. [Testes Automatizados (Rápido)](#1-testes-automatizados-rápido)
2. [Configuração do Ambiente Local](#2-configuração-do-ambiente-local)
3. [Cartões de Teste (PagBank / PagSeguro Sandbox)](#3-cartões-de-teste-pagbank--pagseguro-sandbox)
4. [Cenário de Teste 1: Configurar Gateway PagSeguro no Painel (`veche`)](#4-cenário-de-teste-1-configurar-gateway-pagseguro-no-painel-veche)
5. [Cenário de Teste 2: Compra com PagSeguro no Storefront (`torg`)](#5-cenário-de-teste-2-compra-com-pagseguro-no-storefront-torg)
6. [Cenário de Teste 3: Cartão Recusado no PagSeguro](#6-cenário-de-teste-3-cartão-recusado-no-pagseguro)
7. [Cenário de Teste 4: Webhook do PagSeguro e Conciliação](#7-cenário-de-teste-4-webhook-do-pagseguro-e-conciliação)
8. [Cenário de Teste 5: Auditoria no Banco de Dados](#8-cenário-de-teste-5-auditoria-no-banco-de-dados)
9. [Cenário de Teste 6: Chamadas Diretas via API (cURL)](#9-cenário-de-teste-6-chamadas-diretas-via-api-curl)

---

## 1. Testes Automatizados (Rápido)

Para validar a integridade de todo o código sem necessidade de serviços externos ou navegadores:

### 1.1 Backend (`kremlin`)
```bash
cd kremlin
cargo test -p application --test pagseguro_tests --test payment_credentials_test --test payment_settings_endpoint_test
```
> **Esperado**: Todos os testes unitários do gateway PagSeguro, do chaveiro AES-256-GCM e dos endpoints de configuração de pagamento aprovados com sucesso.

### 1.2 Storefront (`torg`)
```bash
cd torg
npm test -- --run
```
> **Esperado**: Todos os 10 arquivos de teste passando (35 testes), incluindo `CheckoutPage` e renderização de formulários de pagamento.

### 1.3 Backoffice (`veche`)
```bash
cd veche
npm test -- --run
```
> **Esperado**: Todos os testes passando, incluindo `paymentSettingsSlice.test.ts` (operações assíncronas de busca e atualização de configuração de pagamento).

---

## 2. Configuração do Ambiente Local

### 2.1 Obter Credenciais Sandbox do PagBank (PagSeguro)
1. Acesse o painel: [PagBank Developers / Sandbox](https://developer.pagbank.com.br/).
2. Em **Credenciais de Integração**, gere ou copie:
   - **Token de Acesso Sandbox** (ex: `c8d1...`)
   - **Chave Pública** (utilizada pelo SDK de criptografia de cartão no frontend)

### 2.2 Variáveis de Ambiente no Backend (`kremlin/.env`)
Adicione ou configure no arquivo `kremlin/.env` (o backend possui fallback para variáveis de ambiente caso o tenant ainda não tenha configurado credenciais personalizadas no banco):

```dotenv
# PagSeguro Sandbox Padrão (Opcional - fallback)
PAGSEGURO_TOKEN=seu_token_sandbox_pagseguro
PAGSEGURO_PUBLIC_KEY=sua_chave_publica_pagseguro
PAGSEGURO_ENVIRONMENT=sandbox

# Chaveiro de Criptografia AES-256-GCM para configurações de pagamento no banco
PAYMENT_ENCRYPTION_KEYS="v1:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
PAYMENT_ACTIVE_KEY_VERSION="v1"
```
*(Se `PAYMENT_ENCRYPTION_KEYS` não for informada, o sistema reutilizará automaticamente `SHIPPING_ENCRYPTION_KEYS`).*

### 2.3 Mock offline do gateway no backend

Para validar o fluxo de pagamento do `kremlin` sem chamar gateways externos, configure:

```dotenv
PAYMENT_PROVIDER_MODE=mock
PAYMENT_MOCK_ALLOWED=true
PAYMENT_MOCK_PROVIDER=pagseguro
```

No payload de `POST /purchases/{id}/payments/submit`, use `token` com o cenário desejado: `mock:approved`, `mock:authorized`, `mock:pending`, `mock:declined` ou `mock:error`. Aliases como `mock:paid`, `mock:waiting` e `mock:rejected` também são aceitos. Um token JSON como `{"mockOutcome":"in_analysis"}` simula status pendente. A referência `mock:` retornada permite testar a consulta de status sem rede.

O mock é ativado somente com `PAYMENT_PROVIDER_MODE=mock`; sem essa variável, o backend mantém a resolução real. `PAYMENT_MOCK_PROVIDER` aceita `pagseguro` ou `mercado_pago`. Use somente em desenvolvimento/testes. O modo simula o backend do gateway, não a tokenização do SDK PagBank no navegador; para o checkout web, configure as chaves Sandbox do SDK.

### 2.4 Iniciar os Serviços
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

- **Terminal 3 — Painel do Lojista (`veche`)**:
  ```bash
  cd veche
  npm run dev
  ```

- **Terminal 4 — Storefront (`torg`)**:
  ```bash
  cd torg
  npm run dev
  ```

---

## 3. Cartões de Teste (PagBank / PagSeguro Sandbox)

Utilize os cartões oficiais do ambiente de Sandbox do PagSeguro:

| Cenário de Teste | Número do Cartão | Validade | CVV | Nome no Cartão | Resultado Esperado |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Aprovação Imediata (Mastercard)** | `5555 5555 5555 5555` | Data futura (ex: `12/28`) | `123` | `PAGSEGURO TESTE` | Pagamento aprovado (`captured` / `PAID`) |
| **Aprovação Imediata (Visa)** | `4111 1111 1111 1111` | Data futura (ex: `10/29`) | `123` | `CLIENTE APROVADO` | Pagamento aprovado (`captured` / `PAID`) |
| **Recusa / Cartão Inválido** | `4111 1111 1111 1112` | Data futura (ex: `08/28`) | `123` | `CARTAO RECUSADO` | Pagamento recusado (`failed` / `DECLINED`) |

---

## 4. Cenário de Teste 1: Configurar Gateway PagSeguro no Painel (`veche`)

1. Acesse o painel de lojista em `http://localhost:5173` (ou porta do Vite).
2. Faça login como **Tenant Owner** ou **Admin**.
3. No menu lateral, acesse **Operações** > **Configurações de Pagamento** (`/operations/payment-settings`).
4. Na tela:
   - Em **Provedor Ativo**, selecione **PagSeguro (PagBank)**.
   - Em **Ambiente**, selecione **Sandbox (Testes)**.
   - Preencha o **Token de Acesso (API)** com o token sandbox.
   - Preencha a **Chave Pública** com a chave pública do PagSeguro.
5. Clique em **"Salvar Configurações"**.
6. Uma notificação de sucesso verde será exibida: *"Configurações de pagamento salvas com sucesso"*.
7. O token é criptografado em repouso no banco de dados via chave AES-256-GCM.

---

## 5. Cenário de Teste 2: Compra com PagSeguro no Storefront (`torg`)

1. Acesse a loja em `http://localhost:5174`.
2. Adicione qualquer produto ao carrinho e avance para o Checkout (`/checkout`).
3. Conclua as etapas de **Dados do Comprador**, **Endereço** e **Frete**.
4. Na etapa de **Pagamento**:
   - O checkout consulta `GET /checkout/payment-config?tenantId=...`.
   - O formulário exibido será automaticamente o **`PagSeguroCardForm`** (indicado com o selo PagSeguro/PagBank).
5. Preencha os dados do cartão de teste:
   - **Número do Cartão**: `5555 5555 5555 5555`
   - **Nome Impresso no Cartão**: `PAGSEGURO TESTE`
   - **Validade**: `12/28`
   - **CVV**: `123`
   - **Parcelas**: `1x à vista`
6. Clique em **"Pagar com PagSeguro"**.
7. O SDK do PagSeguro criptografa os dados localmente (`window.PagSeguro.encryptCard`) e envia o payload criptografado ao backend.
8. **Resultado**:
   - Tela de confirmação com ícone verde: **"Pagamento confirmado"**.
   - Compra com status aprovado e pedidos gerados.

---

## 6. Cenário de Teste 3: Cartão Recusado no PagSeguro

1. Repita o processo até a etapa de Pagamento no checkout.
2. Utilize o número de cartão para simulação de recusa:
   - **Número**: `4111 1111 1111 1112`
   - **Nome**: `CARTAO RECUSADO`
   - **Validade**: `08/28`
   - **CVV**: `123`
3. Clique em **"Pagar com PagSeguro"**.
4. **Resultado**:
   - O PagSeguro retorna status `DECLINED`.
   - O backend registra o pagamento como `failed` e compra como `payment_failed`.
   - O checkout exibe o alerta: *"Pagamento recusado pelo PagSeguro. Confira os dados ou tente outro cartão."*
   - O comprador pode tentar novamente com outro cartão sem perder o carrinho ou a cotação.

---

## 7. Cenário de Teste 4: Webhook do PagSeguro e Conciliação

O PagSeguro envia notificações HTTP assíncronas com o ID do pedido ou da cobrança.

### 7.1 Simulação de Webhook via cURL
```bash
curl -X POST http://localhost:8080/webhooks/pagseguro \
  -H "Content-Type: application/json" \
  -d '{
    "id": "ORDE_1234567890ABCDEF",
    "reference_id": "novgorod-purchase-1",
    "charges": [
      {
        "id": "CHAR_9988776655",
        "status": "PAID",
        "amount": { "value": 15000 }
      }
    ]
  }'
```
**Resposta esperada**:
```json
{
  "status": "received"
}
```
A rotina interna atualiza a compra de referência para status pago e captura o pagamento associado.

---

## 8. Cenário de Teste 5: Auditoria no Banco de Dados

Conecte-se ao Postgres:
```bash
docker exec -it novgorod_postgres psql -U novgorod-dev -d novgorod-dev
```

### 8.1 Verificar a Configuração Criptografada do Lojista
```sql
SELECT 
    tenant_id, 
    provider, 
    active_version, 
    configuration, 
    key_version,
    length(encrypted_credentials) AS encrypted_payload_bytes,
    updated_at
FROM tenant_payment_settings;
```
*(Você verá que o segredo e o token nunca são salvos em texto puro, apenas o payload binário cifrado via AES-256-GCM).*

### 8.2 Verificar o Pagamento Processado via PagSeguro
```sql
SELECT 
    p.id AS purchase_id,
    p.status AS purchase_status,
    pay.id AS payment_id,
    pay.gateway_provider,
    pay.gateway_reference,
    pay.status AS payment_status,
    cc.brand,
    cc.last_four_digits,
    cc.cardholder_name
FROM purchase p
JOIN payment pay ON pay.purchase_id = p.id
LEFT JOIN credit_card_details cc ON cc.payment_id = pay.id
WHERE pay.gateway_provider = 'pagseguro'
ORDER BY p.id DESC
LIMIT 1;
```
**Resultado esperado**:
- `gateway_provider`: `'pagseguro'`
- `gateway_reference`: ID da ordem ou cobrança no PagSeguro (ex: `'ORDE_...'` ou `'CHAR_...'`)
- `payment_status`: `'captured'`
- `brand`: `'mastercard'` ou `'visa'`
- `last_four_digits`: `'5555'`
- `cardholder_name`: `'PAGSEGURO TESTE'`

---

## 9. Cenário de Teste 6: Chamadas Diretas via API (cURL)

### 9.1 Salvar Configuração de Pagamento de um Tenant (PUT)
```bash
curl -X PUT http://localhost:8080/tenants/1/payment-settings \
  -H "Authorization: Bearer <SEU_JWT_TENANT_OWNER>" \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "pagseguro",
    "configuration": {
      "environment": "sandbox",
      "publicKey": "PUBLIC_KEY_PAGSEGURO_AQUI"
    },
    "credentials": {
      "token": "TOKEN_SANDBOX_PAGSEGURO_AQUI"
    }
  }'
```

### 9.2 Obter Configuração de Pagamento do Tenant (GET)
```bash
curl -X GET http://localhost:8080/tenants/1/payment-settings \
  -H "Authorization: Bearer <SEU_JWT_TENANT_OWNER>"
```
**Resposta (200 OK)**:
```json
{
  "tenantId": 1,
  "provider": "pagseguro",
  "configuration": {
    "environment": "sandbox",
    "publicKey": "PUBLIC_KEY_PAGSEGURO_AQUI"
  },
  "credentialsConfigured": true,
  "updatedAt": "2026-09-30T14:00:00Z"
}
```

### 9.3 Consultar Configuração Pública para Checkout (GET)
```bash
curl -X GET "http://localhost:8080/checkout/payment-config?tenantId=1" \
  -H "Authorization: Bearer <SEU_JWT_CUSTOMER>"
```
**Resposta (200 OK)**:
```json
{
  "provider": "pagseguro",
  "publicKey": "PUBLIC_KEY_PAGSEGURO_AQUI"
}
```
