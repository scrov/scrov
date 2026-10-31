![RealWorld Angular banner](public/images/realworld-angular-banner.png)

# Realworld Angular playground

The frontend single-page application for Realworld Angular, built with [Angular v21](https://angular.dev/). This is a learning and reference implementation that demonstrates modern Angular patterns — standalone components, signals, reactive forms, lazy loading, route guards, and server-sent events — within a cohesive, opinionated codebase. It is part of the [RealWorld Angular](https://github.com/realworld-angular/realworld-angular) initiative.

Customers can browse pizzerias, build their cart, place orders, and track their delivery. Pizzeria admins and kitchen staff have a dedicated admin panel to manage menus, orders, and staff.

> **AI agent integration:** This app exposes live application state to AI agents via Angular's experimental WebMCP API. See [WebMCP Tools](#webmcp-tools).

## Table of Contents

- [Overview](#overview)
- [WebMCP Tools](#webmcp-tools)
- [Tech Stack](#tech-stack)
- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [Available Scripts](#available-scripts)
- [Application Structure](#application-structure)
- [Roles and Access](#roles-and-access)
- [Contributing](#contributing)
- [License](#license)

## Overview

Key features of the frontend:

- **Browse** pizzerias and their menus without an account
- **Cart and checkout** — add pizzas with options (size, crust, toppings) and place an order
- **Order tracking** — view the real-time status of your orders
- **Authentication** — register, log in, and manage your profile
- **Staff invite flow** — accept a pizzeria invite via a token link to join as kitchen staff
- **Admin panel** — manage pizzerias, pizza menus, orders, and staff (role-gated)

> **This is a playground application for developers learning Angular and full-stack patterns.** It is not a real marketplace — no actual orders are fulfilled, no payment is processed, and mock data is encouraged.

## WebMCP Tools

This app uses Angular's experimental [`provideWebMcpTools`](https://github.com/angular/angular/commit/3b0ae5fef0328477ee0f5d51980217e7c583a606) API to expose live application state to AI agents via the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/).

### How it works

`provideWebMcpTools` is an Angular provider factory that registers MCP tools with the browser's `navigator.modelContext` API (via the `@mcp-b/webmcp` polyfill). Each tool has:

- **`name`** — a unique identifier the AI agent uses to call the tool
- **`description`** — a natural-language description that helps the AI decide when to use the tool
- **`inputSchema`** — a JSON Schema describing the tool's input parameters
- **`execute`** — a function invoked in the Angular injection context, so it can call `inject()` to access any service or store

Under the hood, each tool is registered via `declareWebMcpTool` when the associated injector initializes, and automatically unregistered when the injector is destroyed. When declared at the application root (via `bootstrapApplication` providers), tools are available for the lifetime of the app. When declared in route `providers` alongside `withExperimentalAutoCleanupInjectors()`, tools register when a route is activated and unregister on navigation away.

```typescript
// app.config.ts
provideWebMcpTools([
  {
    name: 'get_cart',
    description: 'Returns the current contents of the shopping cart.',
    inputSchema: { type: 'object', properties: {} },
    execute: () => {
      const cartStore = inject(CartStore);
      // read signals, format response...
      return { content: [{ type: 'text', text: '...' }] };
    },
  },
]);
```

### Exposed tools

Tools are registered close to the state or page they expose. App-wide tools (cart, current
user) are declared via `declareWebMcpTool` in the `CartStore` and `Auth` service constructors,
so they're always available. Page-scoped tools are registered via `provideWebMcpTools` in the
relevant route's `providers` (see each feature's `*.webmcp-tools.ts` file) - they inherit that
route's guards for free, so e.g. `place_order` only exists for an agent once the user has
navigated to checkout with items in their cart while signed in.

| Tool                        | Scope                    | Description                                                                                                                      |
| ---------------------------- | ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `get_cart`                   | app-wide (`CartStore`)    | Current cart contents: pizzeria, each item (with ID, pizza, size, extra toppings, quantity, price), and the order total.                  |
| `add_pizza_to_cart`          | app-wide (`CartStore`)    | Adds a pizza to the cart by pizza/size/topping IDs and quantity.                                                                           |
| `update_cart_item_quantity`  | app-wide (`CartStore`)    | Changes an existing cart item's quantity, or removes it with quantity 0.                                                                   |
| `remove_cart_item`           | app-wide (`CartStore`)    | Removes an item from the cart.                                                                                                             |
| `clear_cart`                 | app-wide (`CartStore`)    | Empties the cart.                                                                                                                          |
| `get_current_user`           | app-wide (`Auth`)         | Whether someone is signed in, and their name/email/role if so.                                                                             |
| `logout`                     | app-wide (`Auth`)         | Signs the current user out.                                                                                                                |
| `search_pizzerias`           | `/pizzerias`              | Paginated pizzeria search/listing.                                                                                                         |
| `get_pizzeria`                | `/pizzerias/:id`          | Full details for one pizzeria by ID.                                                                                                       |
| `list_pizzas`                 | `/pizzerias/:id`          | A pizzeria's menu, with name/max-price filtering.                                                                                          |
| `list_pizza_options`          | `/pizzerias/:id`          | Available pizza sizes and extra-topping add-ons (IDs and prices) for `add_pizza_to_cart`.                                                  |
| `place_order`                 | `/checkout`               | Places an order for the current cart contents at the given delivery address. Requires sign-in and a non-empty cart (route-guard enforced). |
| `list_my_orders`              | `/orders`                 | The signed-in user's orders, paginated.                                                                                                    |
| `get_order`                   | `/orders/:id`             | Full status/details for one order by ID.                                                                                                   |
| `cancel_order`                | `/orders/:id`             | Cancels an order by ID, if it hasn't started preparation yet.                                                                              |

Deliberately **not** exposed as tools: `login` / `register`. Those would require an AI agent to
be handed a plaintext password as a tool argument, which risks the credential being logged or
replayed by whatever is driving the agent - a user should type their own password into the login
form, not hand it to a tool call.

## API

The app connects to a deployed backend. Browse the [API documentation](https://api.realworldangular.org/api/docs) for available endpoints.

## Tech Stack

| Layer           | Technology                                                  |
| --------------- | ----------------------------------------------------------- |
| Framework       | [Angular v21](https://angular.dev/) — standalone components |
| Language        | TypeScript 5.9                                              |
| Routing         | `@angular/router` with lazy-loaded routes and guards        |
| HTTP            | `HttpClient` with a credentials interceptor                 |
| Testing         | [Vitest](https://vitest.dev/) with jsdom                    |
| Linting         | ESLint + `angular-eslint`                                   |
| Formatting      | Prettier                                                    |
| Package manager | pnpm                                                        |

## Prerequisites

- **Node.js** >= 20
- **pnpm** >= 9 — install with `npm install -g pnpm`

## Getting Started

### 1. Clone and install dependencies

```bash
git clone <repository-url>
cd realworld-angular
pnpm install
```

### 2. Start the development server

```bash
pnpm start
```

The application will be available at **http://localhost:4200** and automatically reloads on file changes.

## Available Scripts

| Script           | Description                                     |
| ---------------- | ----------------------------------------------- |
| `pnpm start`     | Start the dev server at `http://localhost:4200` |
| `pnpm run build` | Build for production into `dist/`               |
| `pnpm run watch` | Build in watch mode (development)               |
| `pnpm run test`  | Run unit tests with Vitest                      |
| `pnpm run lint`  | Lint the codebase with ESLint                   |

## Application Structure

```
src/
├── app/                        # Root config, shell component, and route definitions
├── core/
│   ├── guards/                 # authGuard, guestGuard, roleGuard
│   ├── interceptors/           # credentialsInterceptor — attaches cookies to requests
│   ├── models/                 # TypeScript interfaces (User, Pizza, Order, etc.)
│   └── services/               # Auth, cart store, interceptors-backed HTTP, …
├── design-system/              # Global CSS reset and design tokens
├── features/
│   ├── auth/                   # Login, register, accept-invite pages
│   ├── home/                   # Homepage — browse all pizzerias
│   ├── pizzerias/              # Pizzeria detail page with menu
│   ├── cart/                   # Shopping cart
│   ├── checkout/               # Checkout flow
│   ├── orders/                 # Order list and order detail pages
│   ├── profile/                # User profile page
│   ├── admin/                  # Admin panel (pizzerias, pizzas, staff, orders)
│   ├── not-found/              # 404 page
│   └── unauthorized/           # 403 page
└── shared/                     # Reusable UI components
```

### Route Map

| Route                                | Access                                                              |
| ------------------------------------ | ------------------------------------------------------------------- |
| `/`                                  | Public                                                              |
| `/pizzerias/:id`                     | Public                                                              |
| `/auth/login`                        | Guests only (redirected if already logged in)                       |
| `/auth/register`                     | Guests only                                                         |
| `/invite/:token`                     | Public                                                              |
| `/cart`                              | Public — build a cart without an account; checkout requires sign-in |
| `/checkout`                          | Authenticated                                                       |
| `/orders`                            | Authenticated                                                       |
| `/orders/:id`                        | Authenticated                                                       |
| `/profile`                           | Authenticated                                                       |
| `/admin/pizzerias`                   | `PIZZERIA_ADMIN`                                                    |
| `/admin/pizzerias/new`               | `PIZZERIA_ADMIN`                                                    |
| `/admin/pizzerias/:id`               | `PIZZERIA_ADMIN` — layout shell; redirects to a default tab         |
| `/admin/pizzerias/:id/orders`        | `PIZZERIA_ADMIN`                                                    |
| `/admin/pizzerias/:id/pizzas`        | `PIZZERIA_ADMIN`                                                    |
| `/admin/pizzerias/:id/configuration` | `PIZZERIA_ADMIN`                                                    |

## Roles and Access

The application supports two user roles:

| Role             | Description                                              |
| ---------------- | -------------------------------------------------------- |
| `CUSTOMER`       | Default role — can browse, order, and track their orders |
| `PIZZERIA_ADMIN` | Can create and manage pizzerias, menus, and orders       |

Role guards (`roleGuard`) protect admin routes client-side. The API also enforces roles server-side.

## Contributing

> **Note on scope:** This project is a focused learning reference — it is not meant to add third-party libraries, to serve as a perfect template stack, or to add features just for the sake of covering more aspects of the marketplace. If you have ideas that fall outside this scope, no contribution is lost — a fork demonstrating those aspects is a great standalone project.

Contributions are welcome! Please follow these guidelines:

- **Fork** the repository and create your branch from `main`
- **Write tests** for new components or services (Vitest)
- **Follow** the existing Angular conventions (standalone components, `inject()`, signals where applicable)
- **Run lint and tests** before submitting: `pnpm run lint && pnpm run test`
- Open a **pull request** with a clear description of the changes and motivation

### Reporting Issues

If you find a bug or have a feature request, please [open an issue](../../issues) with:

- A clear and descriptive title
- Steps to reproduce (for bugs)
- Expected vs. actual behavior
- Browser and OS details

## License

This project is licensed under the [MIT License](LICENSE).
