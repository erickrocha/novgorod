# Novgorod — Arquitetura e Codinomes do Projeto

Este documento define a arquitetura conceitual e a estrutura de módulos do projeto **Novgorod**, utilizando a história da cidade medieval homônima como metáfora organizacional para os componentes do sistema.

## 🏛️ Visão Geral

O projeto **Novgorod** é estruturado de forma modular, separando responsabilidades entre o núcleo de processamento de regras de negócios, a interface administrativa de gestão e a vitrine de vendas voltada para o cliente final.

A escolha do nome remete ao histórico centro comercial medieval que conectava diferentes frentes de forma autônoma, resiliente e altamente estruturada.

---

## 🧩 Estrutura de Módulos

| Módulo / Subprojeto | Nome Histórico | Papel Arquitetural | Descrição |
| :--- | :--- | :--- | :--- |
| **Backend Core** | `kremlin` | Cidadela / Núcleo | O coração do sistema. Abriga as regras de negócios críticas, transações, fluxos de dados, lógica de backend e integrações de serviços. |
| **Admin Panel** | `veché` | Assembleia / Gestão | O painel de controle administrativo. Onde operadores e administradores gerenciam catálogos, pedidos, usuários e parâmetros do sistema. |
| **Storefront** | `torg` | O Grande Mercado | A vitrine digital e loja voltada para o cliente final, focada na experiência de navegação, carrinho de compras e conversão. |

---

## 📂 Estrutura de Diretórios (Monorepo / Organização)

```text
novgorod/
├── kremlin/          # Backend (APIs, regras de negócio e serviços core)
├── veché/            # Frontend Admin (Painel de gestão e operações)
└── torg/             # Storefront (Interface de compras e experiência do cliente)
```

## 📜 Justificativa Histórica dos Nomes

1. **Kremlin (Detinets):** Na antiga Novgorod, o Kremlin era a fortaleza central e o centro administrativo de onde emanavam as decisões mais importantes, protegendo o núcleo estrutural da cidade.
2. **Veché:** A assembleia pública medieval onde as leis eram discutidas e deliberadas pelos líderes e cidadãos. Representa perfeitamente o espaço onde os administradores governam o ecossistema do e-commerce.
3. **Torg:** Significa mercado. Historicamente, a *Torgovaya Storona* (Lado do Mercado) era a margem da cidade onde o comércio ativo e as trocas diárias aconteciam de forma vibrante com o público.