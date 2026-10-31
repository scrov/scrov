import { Injectable, signal, computed } from '@angular/core';
import { httpResource } from '@angular/common/http';
import { declareWebMcpTool } from '@mcp-b/webmcp'; 

export interface CartPizzeria {
  id: string;
}

export interface CartItem {
  id: string;
  pizzaId: string;
  quantity: number;
  selectedSizeId: string | null;
  selectedOptionIds: string[];
}

export interface CartPizza {
  id: string;
  name: string;
  image: string;
  basePrice: number;
}

export interface CartOption {
  id: string;
  label: string;
  price: number;
}

export interface CartItemDetail {
  id: string;
  pizza: CartPizza;
  quantity: number;
  size: CartOption | null;
  extraToppings: CartOption[];
  totalPrice: number;
}

export interface CartData {
  pizzeria: { id: string; name: string; image: string };
  items: CartItemDetail[];
  total: number;
}

@Injectable({ providedIn: 'root' })
export class CartStore {
  public readonly pizzeria = signal<CartPizzeria | null>(null);
  public readonly items = signal<CartItem[]>([]);

  private readonly cartResource = httpResource<CartData>(() => {
    const currentItems = this.items();
    const currentPizzeria = this.pizzeria();

    if (currentItems.length === 0 || !currentPizzeria) {
      return undefined;
    }

    return {
      url: '/api/orders/cart',
      method: 'POST',
      body: {
        pizzeriaId: currentPizzeria.id,
        items: currentItems.map((item) => ({
          pizzaId: item.pizzaId,
          quantity: item.quantity,
          selectedSizeId: item.selectedSizeId ?? undefined,
          selectedOptionIds: item.selectedOptionIds,
        })),
      },
    };
  });

  public readonly cart = computed<CartData | null>(() => this.cartResource.value() ?? null);
  public readonly isLoading = computed<boolean>(() => this.cartResource.isLoading());

  public readonly totalPrice = computed<number>(() => this.cart()?.total ?? 0);

  public readonly itemCount = computed<number>(
    () => this.cart()?.items.reduce((sum, item) => sum + item.quantity, 0) ?? 0,
  );

  public readonly isEmpty = computed<boolean>(() => this.items().length === 0);

  public constructor() {
    declareWebMcpTool({
      name: 'get_cart',
      description:
        'Returns the current contents of the shopping cart, including the pizzeria, each ' +
        'item (with its ID, pizza name, size, extra toppings, quantity, and price), and the ' +
        'order total. Item IDs are needed to call update_cart_item_quantity or remove_cart_item.',
      inputSchema: { type: 'object', properties: {} },
      execute: () => {
        const cart = this.cart();

        if (!cart || this.isEmpty()) {
          return { content: [{ type: 'text', text: 'The cart is currently empty.' }] };
        }

        const itemLines = cart.items
          .map((item) => {
            const size = item.size
              ? `Size: ${item.size.label} (+$${item.size.price})`
              : 'No size selected';
            const toppings = item.extraToppings.length
              ? `Extra toppings: ${item.extraToppings.map((t) => `${t.label} (+$${t.price})`).join(', ')}`
              : 'No extra toppings';
            return `- [Item ID: ${item.id}] ${item.pizza.name} x${item.quantity} | ${size} | ${toppings} | Item total: $${item.totalPrice}`;
          })
          .join('\n');

        return {
          content: [
            {
              type: 'text',
              text: `Cart for ${cart.pizzeria.name}:\n\n${itemLines}\n\nTotal: $${cart.total}`,
            },
          ],
        };
      },
    });

    declareWebMcpTool({
      name: 'add_pizza_to_cart',
      description:
        'Adds a pizza to the shopping cart. pizzaId, selectedSizeId, and selectedOptionIds ' +
        'come from list_pizzas and list_pizza_options. Adding a pizza from a different ' +
        "pizzeria than what's already in the cart replaces the cart's contents.",
      inputSchema: {
        type: 'object',
        properties: {
          pizzeriaId: {
            type: 'string',
            description: 'ID of the pizzeria this pizza belongs to.',
          },
          pizzaId: { type: 'string', description: 'ID of the pizza to add, from list_pizzas.' },
          quantity: {
            type: 'number',
            minimum: 1,
            description: 'How many of this pizza to add.',
          },
          selectedSizeId: {
            type: 'string',
            description:
              'ID of the chosen size, from list_pizza_options. Omit if the pizza has no sizes.',
          },
          selectedOptionIds: {
            type: 'array',
            items: { type: 'string' },
            description: 'IDs of chosen extra toppings, from list_pizza_options.',
          },
        },
        required: ['pizzeriaId', 'pizzaId', 'quantity'],
        additionalProperties: false,
      },
      execute: (args) => {
        const quantity = Math.trunc(args.quantity);
        if (!Number.isFinite(quantity) || quantity < 1) {
          return {
            content: [{ type: 'text', text: 'quantity must be a positive integer.' }],
            isError: true,
          };
        }

        const switchedPizzeria = this.hasItemsForOtherPizzeria(args.pizzeriaId);
        this.addItem(
          args.pizzaId,
          quantity,
          args.selectedSizeId ?? null,
          args.selectedOptionIds ?? [],
          args.pizzeriaId,
        );

        const note = switchedPizzeria
          ? ' Note: the cart contained items from a different pizzeria, so it was replaced with this item.'
          : '';
        return {
          content: [
            {
              type: 'text',
              text: `Added ${quantity} x pizza ${args.pizzaId} to the cart.${note}`,
            },
          ],
        };
      },
    });

    declareWebMcpTool({
      name: 'update_cart_item_quantity',
      description:
        'Changes the quantity of an existing cart item. Use quantity 0 to remove it. Item ' +
        'IDs come from get_cart.',
      inputSchema: {
        type: 'object',
        properties: {
          itemId: { type: 'string', description: 'The cart item ID, from get_cart.' },
          quantity: {
            type: 'number',
            minimum: 0,
            description: 'New quantity. 0 removes the item.',
          },
        },
        required: ['itemId', 'quantity'],
        additionalProperties: false,
      },
      execute: (args) => {
        const quantity = Math.trunc(args.quantity);
        if (!Number.isFinite(quantity) || quantity < 0) {
          return {
            content: [{ type: 'text', text: 'quantity must be zero or a positive integer.' }],
            isError: true,
          };
        }
        if (!this.items().some((item) => item.id === args.itemId)) {
          return {
            content: [
              {
                type: 'text',
                text: `No cart item with ID "${args.itemId}". Call get_cart for current item IDs.`,
              },
            ],
            isError: true,
          };
        }

        this.updateQuantity(args.itemId, quantity);
        return {
          content: [
            {
              type: 'text',
              text:
                quantity === 0
                  ? `Removed item ${args.itemId} from the cart.`
                  : `Updated item ${args.itemId} to quantity ${quantity}.`,
            },
          ],
        };
      },
    });

    declareWebMcpTool({
      name: 'remove_cart_item',
      description: 'Removes an item from the shopping cart entirely. Item IDs come from get_cart.',
      inputSchema: {
        type: 'object',
        properties: {
          itemId: { type: 'string', description: 'The cart item ID, from get_cart.' },
        },
        required: ['itemId'],
        additionalProperties: false,
      },
      execute: (args) => {
        if (!this.items().some((item) => item.id === args.itemId)) {
          return {
            content: [{ type: 'text', text: `No cart item with ID "${args.itemId}".` }],
            isError: true,
          };
        }
        this.removeItem(args.itemId);
        return { content: [{ type: 'text', text: `Removed item ${args.itemId} from the cart.` }] };
      },
    });

    declareWebMcpTool({
      name: 'clear_cart',
      description: 'Empties the shopping cart completely.',
      inputSchema: { type: 'object', properties: {} },
      execute: () => {
        this.clear();
        return { content: [{ type: 'text', text: 'Cart cleared.' }] };
      },
    });
  }

  public hasItemsForOtherPizzeria(pizzeriaId: string): boolean {
    const current = this.pizzeria();
    return current !== null && current.id !== pizzeriaId;
  }

  public addItem(
    pizzaId: string,
    quantity: number,
    selectedSizeId: string | null,
    selectedOptionIds: string[],
    pizzeriaId: string,
  ): void {
    if (this.hasItemsForOtherPizzeria(pizzeriaId)) {
      this.clear();
    }

    this.pizzeria.set({ id: pizzeriaId });

    const itemId = this.generateItemId(pizzaId, selectedSizeId, selectedOptionIds);

    this.items.update((items) => {
      const existing = items.find((item) => item.id === itemId);
      if (existing) {
        return items.map((item) =>
          item.id === itemId ? { ...item, quantity: item.quantity + quantity } : item,
        );
      }

      return [
        ...items,
        {
          id: itemId,
          pizzaId,
          quantity,
          selectedSizeId,
          selectedOptionIds,
        },
      ];
    });
  }

  public updateQuantity(itemId: string, quantity: number): void {
    if (quantity === 0) {
      this.removeItem(itemId);
    } else {
      this.items.update((items) =>
        items.map((item) => (item.id === itemId ? { ...item, quantity } : item)),
      );
    }
  }

  public removeItem(itemId: string): void {
    this.items.update((items) => items.filter((item) => item.id !== itemId));
    if (this.isEmpty()) {
      this.clear();
    }
  }

  public clear(): void {
    this.items.set([]);
    this.pizzeria.set(null);
  }

  private generateItemId(
    pizzaId: string,
    selectedSizeId: string | null,
    selectedOptionIds: string[],
  ): string {
    const sizeId = selectedSizeId ?? '';
    const toppingIds = [...selectedOptionIds].sort().join(',');
    return `${pizzaId}:${sizeId}:${toppingIds}`;
  }
}
