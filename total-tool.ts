import { installWebMCP } from '@mcp-b/webmcp-polyfill';
import { normalizeInputSchema } from '@mcp-b/webmcp-ts-sdk/schema';
import { z } from 'zod';

const totalInput = z.object({
  count: z.string().regex(/^\d+$/).transform(Number),
  limit: z.number().default(10),
});

export async function registerTotalTool() {
  installWebMCP();
  const context = document.modelContext;
  if (!context) throw new Error('WebMCP is unavailable');
  const registration = new AbortController();

  await context.registerTool(
    {
      name: 'calculate_total',
      description: 'Add a numeric count to a limit, which defaults to 10',
      // Keep vendor validation in this handler, including when called through MCP.
      inputSchema: { ...normalizeInputSchema(totalInput).inputSchema },
      async execute(input) {
        const result = await totalInput['~standard'].validate(input);
        if (result.issues) {
          const text = result.issues.map((issue) => issue.message).join('; ');
          return { content: [{ type: 'text', text }], isError: true };
        }
        return { total: result.value.count + result.value.limit };
      },
    },
    { signal: registration.signal }
  );

  return () => registration.abort();
}
