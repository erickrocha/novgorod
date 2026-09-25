# Torg — Vitrine & Storefront Novgorod

> O Grande Mercado (*Torgovaya Storona*) do ecossistema Novgorod. Vitrine digital de e-commerce moderna voltada ao consumidor final.

---

## 🏛️ Papel na Arquitetura Novgorod

| Subprojeto | Papel | Descrição |
| :--- | :--- | :--- |
| **`kremlin`** | Cidadela / Backend Core | API central em Rust (port 8080), regras de negócio e persistência. |
| **`veche`** | Assembleia / Admin Panel | Painel de controle e operações para operadores e administradores. |
| **`torg`** | Grande Mercado / Storefront | Vitrine voltada ao cliente final: catálogo, busca, carrinho, cupons e checkout. |

---

## 🛠️ Stack Tecnológica

- **React 19** + **TypeScript**
- **Vite** (bundler ultra-rápido)
- **Redux Toolkit (`@reduxjs/toolkit` + `react-redux`)**:
  - `cartSlice`: Gestão do carrinho, persistência em `localStorage`, aplicação de cupons e cálculo de frete.
  - `catalogSlice`: Catálogo de produtos, filtros de categoria, faixa de preço, ordenação e busca.
  - `authSlice`: Sessão do cliente e tokens de autenticação.
  - `uiSlice`: Modais de visualização rápida (Quick View), menu responsivo e notificações toast.
- **Axios (`src/api/client.ts`)**:
  - Cliente HTTP centralizado com interceptores de requisição/resposta, tratamento unificado de erros e fallback resiliente para mock data em modo de desenvolvimento.
- **Tailwind CSS v4**:
  - Design contemporâneo, tipografia Plus Jakarta Sans, glassmorphism e micro-interações fluidas.
- **Lucide Icons**:
  - Ícones elegantes e consistentes.

---

## 🚀 Como Executar com Yarn

Na raiz da pasta `torg`:

```bash
# 1. Instalar as dependências
yarn install

# 2. Iniciar o servidor de desenvolvimento
yarn dev

# 3. Compilar bundle de produção
yarn build

# 4. Pré-visualizar o build
yarn preview
```

O servidor iniciará por padrão em `http://localhost:5174`.

---

## 📂 Estrutura de Pastas

```text
torg/
├── src/
│   ├── api/             # Instância do Axios e interceptors
│   ├── components/      # Componentes UI reutilizáveis
│   │   ├── common/      # Botões, badges, toasts
│   │   ├── layout/      # Header, Footer, CartDrawer, QuickViewModal
│   │   └── product/     # ProductCard, ProductGrid, ProductFilters
│   ├── pages/           # Rotas da vitrine (Catalog, ProductDetail, Cart, Checkout)
│   ├── routes/          # Definições do React Router
│   ├── services/        # catalogService, cartService e mockData de alta fidelidade
│   ├── store/           # Redux store, hooks tipados e slices
│   ├── types/           # Interfaces TypeScript para produtos, carrinho e API
│   ├── App.tsx          # Router principal
│   ├── main.tsx         # Ponto de entrada com Redux Provider
│   └── index.css        # Tailwind CSS v4 & design tokens
├── index.html
├── package.json
└── vite.config.ts
```
