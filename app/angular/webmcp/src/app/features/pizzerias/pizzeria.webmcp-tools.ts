import { EnvironmentProviders, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import { Page } from '../../core/models/pagination.model';
import { PizzeriaDetail, PizzeriaSummary } from './models/pizzeria.models';
import { Pizza, PizzaOption } from './models/pizza.models';
import type { WebMcpToolResult } from '../../shared/models/webmcp-tool-result.model';
import { provideWebMcpTools } from '@mcp-b/webmcp';
/**
 * WebMCP tools registered on the pizzeria list route (`pizzeria.routes.ts`, path `''`).
 * Available to an agent whenever the user is browsing the pizzeria list.
 */
export const pizzeriaListWebMcpTools: EnvironmentProviders[] = [
  provideWebMcpTools([
    {
      name: 'search_pizzerias',
      description:
        'Searches or lists pizzerias available to order from, paginated. Returns each ' +
        "pizzeria's ID, name, city, country, and how many pizzas are on its menu. Call " +
        'get_pizzeria with a returned ID to see full details, or list_pizzas to see its menu.',
      inputSchema: {
        type: 'object',
        properties: {
          search: { type: 'string', description: 'Optional text to filter pizzerias by name.' },
          page: { type: 'number', description: 'Page number, 1-indexed. Defaults to 1.' },
          limit: { type: 'number', description: 'Results per page, up to 50. Defaults to 12.' },
        },
        additionalProperties: false,
      },
      execute: async (args): Promise<WebMcpToolResult> => {
        const http = inject(HttpClient);
        const page = Number.isInteger(args.page) && (args.page as number) > 0 ? (args.page as number) : 1;
        const limit =
          Number.isInteger(args.limit) && (args.limit as number) > 0
            ? Math.min(args.limit as number, 50)
            : 12;
        const search = args.search?.trim();

        const result = await firstValueFrom(
          http.get<Page<PizzeriaSummary>>('/api/pizzerias', {
            params: { page, limit, ...(search ? { search } : {}) },
          }),
        );

        if (result.items.length === 0) {
          return { content: [{ type: 'text', text: 'No pizzerias matched.' }] };
        }

        const lines = result.items
          .map(
            (pizzeria) =>
              `- [ID: ${pizzeria.id}] ${pizzeria.name} - ${pizzeria.city}, ${pizzeria.country} ` +
              `(${pizzeria._count.pizzas} pizzas on the menu)`,
          )
          .join('\n');

        return {
          content: [
            {
              type: 'text',
              text: `Page ${result.page} of ${result.totalPages} (${result.total} total):\n\n${lines}`,
            },
          ],
        };
      },
    },
  ]),
];

/**
 * WebMCP tools registered on the pizzeria detail route (`pizzeria.routes.ts`, path `':id'`).
 * Available to an agent whenever the user is viewing a specific pizzeria's page.
 */
export const pizzeriaDetailWebMcpTools: EnvironmentProviders[] = [
  provideWebMcpTools([
    {
      name: 'get_pizzeria',
      description:
        'Gets full details for a single pizzeria by ID: name, location, owner, staff, and ' +
        'how many pizzas are on its menu.',
      inputSchema: {
        type: 'object',
        properties: {
          pizzeriaId: {
            type: 'string',
            description: 'The pizzeria ID, as returned by search_pizzerias.',
          },
        },
        required: ['pizzeriaId'],
        additionalProperties: false,
      },
      execute: async (args): Promise<WebMcpToolResult> => {
        const http = inject(HttpClient);
        const pizzeria = await firstValueFrom(
          http.get<PizzeriaDetail>(`/api/pizzerias/${args.pizzeriaId}`),
        );
        const staff = pizzeria.staff.length
          ? pizzeria.staff.map((member) => member.name).join(', ')
          : 'none';

        return {
          content: [
            {
              type: 'text',
              text:
                `${pizzeria.name} (ID: ${pizzeria.id}) - ${pizzeria.city}, ${pizzeria.country}\n` +
                `Owner: ${pizzeria.owner.name}\n` +
                `Staff: ${staff}\n` +
                `${pizzeria._count.pizzas} pizzas on the menu. Call list_pizzas with this ` +
                'pizzeria ID to see them.',
            },
          ],
        };
      },
    },
  ]),
  provideWebMcpTools([
    {
      name: 'list_pizzas',
      description:
        "Lists the pizzas on a pizzeria's menu, with optional name/max-price filtering and " +
        'pagination. Each pizza includes its base price and included toppings. Call ' +
        'list_pizza_options to see the available sizes and extra-topping add-ons, whose IDs ' +
        'are used when calling add_pizza_to_cart.',
      inputSchema: {
        type: 'object',
        properties: {
          pizzeriaId: {
            type: 'string',
            description: 'The pizzeria ID, as returned by search_pizzerias or get_pizzeria.',
          },
          name: { type: 'string', description: 'Optional text to filter pizzas by name.' },
          maxPrice: { type: 'number', description: 'Optional maximum base price filter.' },
          page: { type: 'number', description: 'Page number, 1-indexed. Defaults to 1.' },
          limit: { type: 'number', description: 'Results per page, up to 50. Defaults to 8.' },
        },
        required: ['pizzeriaId'],
        additionalProperties: false,
      },
      execute: async (args): Promise<WebMcpToolResult> => {
        const http = inject(HttpClient);
        const params: Record<string, string | number> = {};
        const name = args.name?.trim();
        if (name) params['name'] = name;
        if (typeof args.maxPrice === 'number') params['maxPrice'] = args.maxPrice;
        if (Number.isInteger(args.page) && (args.page as number) > 0) {
          params['page'] = args.page as number;
        }
        if (Number.isInteger(args.limit) && (args.limit as number) > 0) {
          params['limit'] = Math.min(args.limit as number, 50);
        }

        const pizzas = await firstValueFrom(
          http.get<Pizza[]>(`/api/pizzerias/${args.pizzeriaId}/pizzas`, { params }),
        );

        if (pizzas.length === 0) {
          return { content: [{ type: 'text', text: 'No pizzas matched.' }] };
        }

        const lines = pizzas
          .map((pizza) => {
            const included = pizza.toppings.length
              ? pizza.toppings.map((topping) => topping.label).join(', ')
              : 'none';
            return (
              `- [Pizza ID: ${pizza.id}] ${pizza.name} - base price $${pizza.basePrice}\n` +
              `  Included toppings: ${included}`
            );
          })
          .join('\n');

        return { content: [{ type: 'text', text: lines }] };
      },
    },
  ]),
  provideWebMcpTools([
    {
      name: 'list_pizza_options',
      description:
        'Lists the available pizza sizes and extra-topping add-ons, with their IDs and ' +
        'prices. These IDs are used as selectedSizeId and selectedOptionIds when calling ' +
        'add_pizza_to_cart. The same options apply to every pizza at every pizzeria.',
      inputSchema: { type: 'object', properties: {} },
      execute: async (): Promise<WebMcpToolResult> => {
        const http = inject(HttpClient);
        const [sizes, toppings] = await Promise.all([
          firstValueFrom(http.get<PizzaOption[]>('/api/options/sizes')),
          firstValueFrom(http.get<PizzaOption[]>('/api/options/toppings')),
        ]);

        const format = (options: PizzaOption[]): string =>
          options.length
            ? options
                .map((option) => `- [ID: ${option.id}] ${option.label} (+$${option.price})`)
                .join('\n')
            : 'none';

        return {
          content: [
            {
              type: 'text',
              text: `Sizes:\n${format(sizes)}\n\nExtra toppings:\n${format(toppings)}`,
            },
          ],
        };
      },
    },
  ]),
];
