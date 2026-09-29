# Guia Passo a Passo: Como Testar a Implementação de Frete (Novgorod)

Este documento descreve como testar de ponta a ponta a funcionalidade de frete em todos os módulos do projeto **Novgorod**:
- **`kremlin`**: Regras de negócio, cálculo de frete (Fixo e Correios), criptografia AES-256-GCM de credenciais e endpoints de cotação/seleção.
- **`veche`**: Painel administrativo do lojista (configuração de modo, serviços, embalagem e credenciais dos Correios).
- **`torg`**: Loja/Storefront do comprador (cálculo de frete, visualização de opções PAC/SEDEX, seleção de serviço e atualização do resumo do pedido).

---

## 📋 Sumário
1. [Testes Automatizados (Verificação Rápida)](#1-testes-automatizados-verificação-rápida)
2. [Preparação do Ambiente Local](#2-preparação-do-ambiente-local)
3. [Cenário de Teste 1: Configuração no Painel do Lojista (`veche`)](#3-cenário-de-teste-1-configuração-no-painel-do-lojista-veche)
4. [Cenário de Teste 2: Cotação e Seleção no Storefront (`torg`)](#4-cenário-de-teste-2-cotação-e-seleção-no-storefront-torg)
5. [Cenário de Teste 3: Validação de Erros e Casos de Borda](#5-cenário-de-teste-3-validação-de-erros-e-casos-de-borda)
6. [Cenário de Teste 4: Verificação Direta via API (cURL)](#6-cenário-de-teste-4-verificação-direta-via-api-curl)

---

## 1. Testes Automatizados (Verificação Rápida)

Para validar a integridade de todo o código sem necessidade de iniciar os servidores web:

### 1.1 Backend (`kremlin`)

Execute todos os testes unitários do workspace Rust:
```bash
cargo test --workspace --manifest-path kremlin/Cargo.toml
```
> **Esperado**: 102+ testes passando (incluindo testes de domínio de frete, credenciais criptografadas e caso de uso de cotação).

Para executar os testes de integração com PostgreSQL (requer banco rodando):
```bash
KREMLIN_TEST_DATABASE_URL=postgresql://novgorod-dev:9e374511@localhost:5432/shipping_test \
  cargo test -p business --manifest-path kremlin/Cargo.toml \
  --test shipping_postgres --test marketplace_postgres -- --ignored --nocapture
```

### 1.2 Painel do Lojista (`veche`)

Entre no diretório e execute os testes unitários e o build de produção:
```bash
cd veche
npm test
npm run build
```
> **Esperado**: 28/28 testes passando (incluindo `shippingSettingsSlice.test.ts` e `ShippingSettings.test.tsx`), e build TypeScript/Vite concluído sem erros.

### 1.3 Storefront (`torg`)

Entre no diretório e execute os testes unitários e o build de produção:
```bash
cd torg
npm test
npm run build
```
> **Esperado**: 35/35 testes passando (incluindo testes de cotação e seleção de frete no `CheckoutPage.test.tsx`), e build Vite concluído sem erros.

---

## 2. Preparação do Ambiente Local

### 2.1 Subir Banco de Dados e Serviços Auxiliares

Inicie o PostgreSQL e o LocalStack via Docker Compose:
```bash
docker compose -f infra/dev/docker-compose.yml up -d
```
Verifique se o container `novgorod_postgres` está saudável:
```bash
docker ps --filter "name=novgorod_postgres"
```

### 2.2 Configurar Variáveis de Ambiente no Backend (`kremlin/.env`)

Certifique-se de que o arquivo `kremlin/.env` contenha as chaves de criptografia para credenciais de frete:
```dotenv
SHIPPING_ENCRYPTION_KEYS={"v1":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}
SHIPPING_ACTIVE_KEY_VERSION=v1
CORREIOS_ENVIRONMENT=homologation
```
> *Nota*: A chave hexadecimal de 64 caracteres acima serve para desenvolvimento e testes locais.

### 2.3 Iniciar as Aplicações

Abra 3 terminais separados:

- **Terminal 1 — Backend (`kremlin`)**:
  ```bash
  cd kremlin
  cargo run -p application
  ```
  *(Servidor rodando em `http://localhost:8080`)*

- **Terminal 2 — Painel Lojista (`veche`)**:
  ```bash
  cd veche
  npm run dev
  ```
  *(Acesse `http://localhost:5173` ou a porta indicada pelo Vite)*

- **Terminal 3 — Storefront (`torg`)**:
  ```bash
  cd torg
  npm run dev
  ```
  *(Acesse `http://localhost:5174` ou a porta indicada pelo Vite)*

---

## 3. Cenário de Teste 1: Configuração no Painel do Lojista (`veche`)

### Objetivo
Configurar os parâmetros de frete de um lojista (modo de cálculo, CEP de origem, margem de embalagem, serviços ativos e credenciais dos Correios).

### Passo a Passo:
1. Acesse o painel **Veché** no navegador.
2. Faça login com uma conta de lojista (`TenantOwner`) ou administrador (`SysAdmin`).
3. No menu lateral, acesse **Operações** → **Configurações de Frete** (rota `/operacoes/frete`).
4. **Testar Modo Fixo**:
   - Selecione a opção **"Tabela Fixa (por Estado/UF)"**.
   - Note que os campos específicos dos Correios ficam ocultos.
   - Clique em **"Salvar configurações"** e verifique a mensagem de sucesso.
5. **Testar Modo Correios**:
   - Alterne o seletor para **"Correios (API Oficial)"**.
   - **CEP de Origem**: Preencha um CEP válido de remetente (ex: `01001-000`).
   - **Tolerâncias de Embalagem**:
     - Peso adicional: `100` g
     - Comprimento: `10` mm
     - Largura: `10` mm
     - Altura: `10` mm
   - **Serviços Contratados**:
     - Mantenha ou adicione:
       - Código: `04510` | Nome: `PAC`
       - Código: `04014` | Nome: `SEDEX`
   - **Credenciais da API Correios**:
     - Usuário Meu Correios: `usuario_teste`
     - Código de Acesso às APIs: `token_acesso_123`
     - Cartão de Postagem: `0076543210`
     - Número do Contrato: `9912345678`
     - Código Regional: `08`
   - Clique em **"Salvar configurações"**.
6. **Verificação**:
   - A notificação de sucesso deve aparecer no topo.
   - O selo de status deve exibir **"Configuradas (protegidas)"** na seção de credenciais.
   - Recarregue a página (F5): confirme que a configuração (modo, CEP, serviços) persiste e os campos de credenciais permanecem limpos por segurança (campos write-only).

---

## 4. Cenário de Teste 2: Cotação e Seleção no Storefront (`torg`)

### Objetivo
Simular o fluxo de compra de um cliente, calculando o frete via Correios e selecionando entre PAC e SEDEX com atualização imediata dos valores.

### Passo a Passo:
1. Acesse o **Torg** no navegador.
2. Adicione ao carrinho um produto que possua dimensões e peso configurados (ex: Vinho Tinto Reserva).
3. Vá para o carrinho e clique em **"Finalizar compra"** (rota `/checkout`).
4. **Passo 1: Seus dados**:
   - Certifique-se de que o CPF e dados de contato estão preenchidos.
5. **Passo 2: Entrega**:
   - Escolha um endereço salvo ou informe um CEP de destino (ex: `90000-000` - Porto Alegre/RS).
6. **Passo 3: Produtos e cupons**:
   - Clique no botão **"Calcular total com frete"**.
7. **Passo 4: Opções de entrega (NOVO)**:
   - Uma nova seção **"4. Opções de entrega"** será exibida.
   - Devem constar as opções disponíveis retornadas pelo cálculo:
     - **PAC**: Preço (ex: `R$ 15,00`) e prazo (`Em até 5 dias úteis`).
     - **SEDEX**: Preço (ex: `R$ 28,00`) e prazo (`Em até 2 dias úteis`).
   - Por padrão, o serviço de menor custo (PAC) vem pré-selecionado.
   - No painel lateral **"Resumo da compra"**, verifique a linha:
     ```text
     Frete (PAC): R$ 15,00
     Total a pagar: R$ ...
     ```
8. **Alteração do Serviço de Frete**:
   - Clique no radio button de **SEDEX**.
   - Observe o indicador de atualização (*"Atualizando frete…"*) enquanto a requisição `POST /checkout/quotes/{id}/shipping-selection` é enviada.
   - Assim que a resposta retornar:
     - O radio button do SEDEX passa a estar selecionado.
     - O resumo lateral é atualizado em tempo real para:
       ```text
       Frete (SEDEX): R$ 28,00
       Total a pagar: [novo total recalculado com o frete do SEDEX]
       ```
9. **Passo 5: Pagamento**:
   - O formulário de pagamento com cartão de crédito (Mercado Pago) ou conclusão de pedido é exibido com o valor total atualizado correspondente à opção escolhida.

---

## 5. Cenário de Teste 3: Validação de Erros e Casos de Borda

| Caso de Teste | Ação | Comportamento Esperado |
| :--- | :--- | :--- |
| **CEP Inexistente ou Inválido** | Informar um CEP com formato inválido (ex: `00000-000` ou menos de 8 dígitos). | O sistema bloqueia o envio ou retorna erro de validação (HTTP 400). |
| **Dimensões Acima do Limite** | Adicionar um volume com peso > 30kg ou soma das dimensões > 200cm. | A API rejeita a cotação com a mensagem explicativa de que os Correios não aceitam pacotes com essas medidas. |
| **Lojista sem Serviço Disponível** | Tentar cotar para um lojista que desmarcou todos os serviços ou credenciais inválidas. | O sistema exibe o aviso claro: *"O vendedor não possui serviço de entrega disponível para este endereço"*. |
| **Expiração da Cotação (15 min)** | Deixar a cotação aberta por mais de 15 minutos e tentar submeter o pedido. | O backend rejeita com status 409 (conflito/expiração) e solicita recálculo do frete. |
| **Alteração de Itens/Endereço** | Modificar a quantidade no carrinho ou trocar o endereço de entrega após a cotação. | A cotação anterior é automaticamente invalidada e a tela solicita que o botão *"Calcular total com frete"* seja acionado novamente. |

---

## 6. Cenário de Teste 4: Verificação Direta via API (cURL)

Caso queira testar a API REST diretamente no terminal via `curl`:

### 6.1 Consultar Configuração de Frete
```bash
curl -X GET http://localhost:8080/tenants/1/shipping-settings \
  -H "Authorization: Bearer <SELLER_TOKEN>" \
  -H "Accept: application/json"
```

### 6.2 Atualizar Configuração e Credenciais
```bash
curl -X PUT http://localhost:8080/tenants/1/shipping-settings \
  -H "Authorization: Bearer <SELLER_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{
    "configuration": {
      "mode": "correios",
      "originCep": "01001000",
      "services": [
        {"code": "04510", "name": "PAC"},
        {"code": "04014", "name": "SEDEX"}
      ],
      "packaging": {"weightG": 50, "lengthMm": 10, "widthMm": 10, "heightMm": 10}
    },
    "credentials": {
      "username": "usuario_teste",
      "apiAccessCode": "token_acesso_123",
      "postingCard": "0076543210",
      "contract": "9912345678",
      "regionalIdentifier": "08"
    }
  }'
```

### 6.3 Selecionar Opção de Frete na Cotação
```bash
curl -X POST http://localhost:8080/checkout/quotes/55/shipping-selection \
  -H "Authorization: Bearer <CUSTOMER_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '[
    {"tenantId": 1, "optionId": "d1e7c5b0-2b1a-4c28-98e3-0c15d48a1234"}
  ]'
```
> Retorna status `201 Created` com o novo ID de cotação e os totais recalculados.
