import '@mcp-b/global';
import type { JsonSchemaForInference, ToolDescriptorFromSchema } from '@mcp-b/webmcp-ts-sdk';

const inputSchema = {
  type: 'object',
  properties: {
    query: { type: 'string' },
    limit: { type: 'integer', minimum: 1 },
  },
  required: ['query'],
  additionalProperties: false,
} as const satisfies JsonSchemaForInference;

const outputSchema = {
  type: 'object',
  properties: {
    total: { type: 'integer' },
    items: { type: 'array', items: { type: 'string' } },
  },
  required: ['total'],
  additionalProperties: false,
} as const satisfies JsonSchemaForInference;

const tool = {
  name: 'search_summary',
  description: 'Search with summary',
  inputSchema,
  outputSchema,
  execute: ({ query }) => ({ total: 1, items: [query] }),
} satisfies ToolDescriptorFromSchema<typeof inputSchema, typeof outputSchema>;

const context = document.modelContext;
if (!context) throw new Error('WebMCP is unavailable');
await context.registerTool(tool);
