import { EnvironmentProviders, inject } from '@angular/core';
import { Router } from '@angular/router';
import { firstValueFrom } from 'rxjs';
import { CartStore } from '../cart/cart.store';
import { OrderApi } from '../orders/order-api';
import type { WebMcpToolResult } from '../../shared/models/webmcp-tool-result.model';
import { provideWebMcpTools } from '@mcp-b/webmcp';
/**
 * WebMCP tools registered on the checkout route (`checkout.routes.ts`).
 *
 * That route already requires an authenticated user with a non-empty cart
 * (`authGuard`, `cartNotEmptyGuard` in `app.routes.ts` / `checkout.routes.ts`), so `place_order`
 * inherits that gating for free: it simply doesn't exist for an agent until the user has
 * navigated here.
 */
export const checkoutWebMcpTools: EnvironmentProviders[] = [
  provideWebMcpTools([
    {
      name: 'place_order',
      description:
        'Places an order for everything currently in the shopping cart, delivered to the ' +
        'given address. The order items and pizzeria come from the current cart (see ' +
        'get_cart), not from this call, so the cart total shown to the user always matches ' +
        'what gets ordered. On success the cart is cleared.',
      inputSchema: {
        type: 'object',
        properties: {
          deliveryStreet: { type: 'string', description: 'Delivery street address.' },
          deliveryCity: { type: 'string', description: 'Delivery city.' },
          deliveryCountry: { type: 'string', description: 'Delivery country.' },
          billingStreet: {
            type: 'string',
            description: 'Optional billing street address, if different from delivery.',
          },
          billingCity: { type: 'string', description: 'Optional billing city.' },
          billingCountry: { type: 'string', description: 'Optional billing country.' },
          notes: {
            type: 'string',
            description: 'Optional delivery notes for the pizzeria, up to 300 characters.',
          },
        },
        required: ['deliveryStreet', 'deliveryCity', 'deliveryCountry'],
        additionalProperties: false,
      },
      execute: async (args): Promise<WebMcpToolResult> => {
        const cartStore = inject(CartStore);
        const orderApi = inject(OrderApi);
        const router = inject(Router);

        const pizzeria = cartStore.pizzeria();
        const items = cartStore.items();
        if (!pizzeria || items.length === 0) {
          return {
            content: [
              { type: 'text', text: 'The cart is empty. Add items before placing an order.' },
            ],
            isError: true,
          };
        }

        const deliveryStreet = args.deliveryStreet?.trim();
        const deliveryCity = args.deliveryCity?.trim();
        const deliveryCountry = args.deliveryCountry?.trim();
        if (!deliveryStreet || !deliveryCity || !deliveryCountry) {
          return {
            content: [
              {
                type: 'text',
                text: 'deliveryStreet, deliveryCity, and deliveryCountry are all required.',
              },
            ],
            isError: true,
          };
        }

        const billingStreet = args.billingStreet?.trim();
        const billingCity = args.billingCity?.trim();
        const billingCountry = args.billingCountry?.trim();
        const hasBilling = !!(billingStreet || billingCity || billingCountry);
        if (hasBilling && (!billingStreet || !billingCity || !billingCountry)) {
          return {
            content: [
              {
                type: 'text',
                text:
                  'billingStreet, billingCity, and billingCountry must all be given together, ' +
                  'or all omitted.',
              },
            ],
            isError: true,
          };
        }

        try {
          const order = await firstValueFrom(
            orderApi.createOrder({
              pizzeriaId: pizzeria.id,
              deliveryAddress: {
                street: deliveryStreet,
                city: deliveryCity,
                country: deliveryCountry,
              },
              ...(hasBilling
                ? {
                    billingAddress: {
                      street: billingStreet as string,
                      city: billingCity as string,
                      country: billingCountry as string,
                    },
                  }
                : {}),
              notes: args.notes?.trim() || undefined,
              items: items.map((item) => ({
                pizzaId: item.pizzaId,
                quantity: item.quantity,
                selectedSizeId: item.selectedSizeId ?? undefined,
                selectedOptionIds: item.selectedOptionIds,
              })),
            }),
          );

          cartStore.clear();
          void router.navigate(['/orders', order.id]);

          return {
            content: [
              {
                type: 'text',
                text: `Order placed! Order ID: ${order.id}, status: ${order.status}, total: $${order.total}.`,
              },
            ],
          };
        } catch {
          return {
            content: [{ type: 'text', text: 'Failed to place the order. Please try again.' }],
            isError: true,
          };
        }
      },
    },
  ]),
];
