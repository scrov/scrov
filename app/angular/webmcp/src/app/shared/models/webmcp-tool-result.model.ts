/**
 * Return shape for a WebMCP tool's `execute` function: a list of content blocks for the
 * agent, plus an optional error flag. `execute`'s declared return type is `unknown`, so this
 * annotation is purely for our own type-checking (and to satisfy
 * `@typescript-eslint/explicit-function-return-type`, which - unlike a direct
 * `declareWebMcpTool({...})` call - isn't inferred automatically when the tool object is
 * nested inside the array literal passed to `provideWebMcpTools([...])`).
 */
export interface WebMcpToolResult {
  content: { type: 'text'; text: string }[];
  isError?: boolean;
}
