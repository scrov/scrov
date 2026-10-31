import { EnvironmentProviders, inject } from '@angular/core';
import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import { Page } from '../../core/models/pagination.model';
import { Order } from './order.models';
import { OrderApi } from './order-api';
import type { WebMcpToolResult } from '../../shared/models/webmcp-tool-result.model';
import { provideWebMcpTools } from '@mcp-b/webmcp';
/**
 * WebMCP tools registered on the order list route (`order.routes.ts`, path `''`).
 * The parent `orders` route already requires an authenticated user (`authGuard` in
 * `app.routes.ts`), so this tool only exists for an agent once the user is signed in.
 */
export const orderListWebMcpTools: EnvironmentProviders[] = [
  provideWebMcpTools([
    {
      name: 'list_my_orders',
      description:
        "Lists the signed-in user's orders, paginated. Each entry includes the order ID, " +
        'status, pizzeria, and total. Call get_order with an order ID for full details.',
      inputSchema: {
        type: 'object',
        properties: {
          page: { type: 'number', description: 'Page number, 1-indexed. Defaults to 1.' },
          limit: { type: 'number', description: 'Results per page. Defaults to 10.' },
        },
        additionalProperties: false,
      },
      execute: async (args): Promise<WebMcpToolResult> => {
        const http = inject(HttpClient);
        const page =
          Number.isInteger(args.page) && (args.page as number) > 0 ? (args.page as number) : 1;
        const limit =
          Number.isInteger(args.limit) && (args.limit as number) > 0
            ? (args.limit as number)
            : 10;

        const result = await firstValueFrom(
          http.get<Page<Order>>('/api/orders', { params: { page, limit } }),
        );

        if (result.items.length === 0) {
          return { content: [{ type: 'text', text: 'No orders found.' }] };
        }

        const lines = result.items
          .map(
            (order) =>
              `- [Order ID: ${order.id}] ${order.pizzeria.name} - status: ${order.status}, ` +
              `total: $${order.total}, placed: ${order.createdAt}`,
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
 * WebMCP tools registered on the order detail route (`order.routes.ts`, path `':id'`).
 */
export const orderDetailWebMcpTools: EnvironmentProviders[] = [
  provideWebMcpTools([
    {
      name: 'get_order',
      description:
        'Gets the current status and full details of a specific order by ID: items, ' +
        'addresses, notes, and total. Reads the same live status feed the order tracking ' +
        'page uses.',
      inputSchema: {
        type: 'object',
        properties: {
          orderId: {
            type: 'string',
            description: 'The order ID, as returned by list_my_orders or place_order.',
          },
        },
        required: ['orderId'],
        additionalProperties: false,
      },
      execute: (args, client): Promise<WebMcpToolResult> =>
        new Promise((resolve) => {
          const orderId = args.orderId;
          const source = new EventSource(`/api/orders/${orderId}/subscribe`, {
            withCredentials: true,
          });
          let settled = false;

          const finish = (result: WebMcpToolResult): void => {
            if (settled) return;
            settled = true;
            source.close();
            resolve(result);
          };

          client.signal.addEventListener(
            'abort',
            () => finish({ content: [{ type: 'text', text: 'Cancelled.' }], isError: true }),
            { once: true },
          );

          source.onmessage = (event: MessageEvent<string>): void => {
            try {
              const order: Order = JSON.parse(event.data);
              const itemLines = order.items
                .map(
                  (item) => `  - ${item.pizza.name} x${item.quantity} ($${item.unitPrice} each)`,
                )
                .join('\n');

              finish({
                content: [
                  {
                    type: 'text',
                    text:
                      `Order ${order.id} - status: ${order.status}\n` +
                      `Pizzeria: ${order.pizzeria.name} (${order.pizzeria.city}, ${order.pizzeria.country})\n` +
                      `Total: $${order.total}\n` +
                      `Delivery address: ${order.deliveryAddress.street}, ${order.deliveryAddress.city}, ${order.deliveryAddress.country}\n` +
                      (order.notes ? `Notes: ${order.notes}\n` : '') +
                      `Items:\n${itemLines}`,
                  },
                ],
              });
            } catch {
              finish({
                content: [{ type: 'text', text: 'Could not read order data.' }],
                isError: true,
              });
            }
          };

          source.onerror = (): void => {
            finish({
              content: [{ type: 'text', text: `Could not load order "${orderId}".` }],
              isError: true,
            });
          };
        }),
    },
  ]),
  provideWebMcpTools([
    {
      name: 'cancel_order',
      description:
        'Cancels an order by ID. Only orders that have not yet started preparation can ' +
        'typically be cancelled; the server rejects the request otherwise.',
      inputSchema: {
        type: 'object',
        properties: {
          orderId: { type: 'string', description: 'The order ID to cancel.' },
        },
        required: ['orderId'],
        additionalProperties: false,
      },
      execute: async (args): Promise<WebMcpToolResult> => {
        const orderApi = inject(OrderApi);
        try {
          const order = await firstValueFrom(orderApi.cancelOrder(args.orderId));
          return {
            content: [{ type: 'text', text: `Order ${order.id} is now ${order.status}.` }],
          };
        } catch (err: unknown) {
          const message =
            err instanceof HttpErrorResponse && typeof err.error?.message === 'string'
              ? err.error.message
              : 'Could not cancel this order.';
          return { content: [{ type: 'text', text: message }], isError: true };
        }
      },
    },
  ]),
];
