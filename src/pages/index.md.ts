import type { APIRoute } from "astro";
import { homeMarkdown, markdownResponse } from "@/lib/agent-content";

export const prerender = true;

export const GET: APIRoute = () => markdownResponse(homeMarkdown());
