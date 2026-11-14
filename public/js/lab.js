// The lab page: Postgres (PGlite) and Lua (Fengari) running in the browser,
// both loaded with the site's data from /api/data.json.
//
// `window.scrovLab` publishes both shells to the WebMCP tools registered by
// the Rust client (crates/client): `sql(query)` and `lua(code)` return a
// promise for the text the page shows, and reject when the run fails.

import { PGlite } from "/vendor/pglite/index.js";

const $ = (id) => document.getElementById(id);
const dataPromise = fetch("/api/data.json").then((response) => {
	if (!response.ok) throw new Error(`/api/data.json: HTTP ${response.status}`);
	return response.json();
});

function show(pane, text, failed) {
	pane.textContent = text;
	pane.classList.toggle("text-c3", failed);
}

// ── Postgres ─────────────────────────────────────────────────────────────────

const SCHEMA = `
create table projects (name text primary key, description text, repo text, stars text, updated text, url text);
create table posts (slug text primary key, title text, description text, date date, url text);
create table systems (slug text primary key, os text, kernel text, shell text, pkg text, wm text, terminal text, font text, accent text);
`;

/** Render rows as a psql-style text table. */
function formatTable(fields, rows) {
	if (fields.length === 0) return "(no columns)";
	const cell = (value) =>
		value === null || value === undefined
			? ""
			: value instanceof Date
				? value.toISOString().slice(0, 10)
				: typeof value === "object"
					? JSON.stringify(value)
					: String(value);
	const names = fields.map((field) => field.name);
	const table = rows.map((row) => names.map((name) => cell(row[name])));
	const widths = names.map((name, i) =>
		Math.max(name.length, ...table.map((row) => row[i].length)),
	);
	const line = (cells) =>
		cells.map((text, i) => ` ${text.padEnd(widths[i])} `).join("|");
	return [
		line(names),
		widths.map((width) => "-".repeat(width + 2)).join("+"),
		...table.map(line),
		`(${rows.length} row${rows.length === 1 ? "" : "s"})`,
	].join("\n");
}

async function startPostgres() {
	const db = new PGlite();
	await db.exec(SCHEMA);
	const data = await dataPromise;
	for (const [table, rows] of Object.entries(data)) {
		for (const row of rows) {
			const columns = Object.keys(row);
			const placeholders = columns.map((_, i) => `$${i + 1}`).join(", ");
			await db.query(
				`insert into ${table} (${columns.join(", ")}) values (${placeholders})`,
				columns.map((column) => row[column]),
			);
		}
	}
	return db;
}

const sqlStatus = $("sql-status");
const sqlOutput = $("sql-output");
const sqlRun = $("sql-run");
const postgres = startPostgres();

/** Run SQL, show the outcome in the page and return the text shown. */
async function runSql(query) {
	const db = await postgres;
	sqlRun.disabled = true;
	const started = performance.now();
	try {
		const results = await db.exec(query);
		const text =
			results
				.map((result) =>
					result.fields.length > 0
						? formatTable(result.fields, result.rows)
						: `OK (${result.affectedRows ?? 0} rows affected)`,
				)
				.join("\n\n") || "OK";
		show(sqlOutput, text, false);
		sqlStatus.textContent = `${(performance.now() - started).toFixed(1)} ms`;
		return text;
	} catch (error) {
		show(sqlOutput, `ERROR:  ${error.message}`, true);
		sqlStatus.textContent = "failed";
		throw error;
	} finally {
		sqlRun.disabled = false;
	}
}

postgres.then(
	() => {
		sqlStatus.textContent = "ready";
		sqlRun.disabled = false;
	},
	(error) => {
		sqlStatus.textContent = `PGlite failed to start: ${error.message}`;
	},
);

$("sql-form").addEventListener("submit", (event) => {
	event.preventDefault();
	// The failure is already shown in the output pane.
	runSql($("sql-input").value).catch(() => {});
});

// ── Lua ──────────────────────────────────────────────────────────────────────

const luaStatus = $("lua-status");
const luaOutput = $("lua-output");
const luaRun = $("lua-run");
let luaLines = [];

// `print` appends to the output pane; `site` is a plain Lua table built from
// the JSON, so scripts can use ipairs/pairs on it.
const LUA_PRELUDE = `
local js = require "js"
local function to_lua(value)
  -- JS strings, numbers and booleans arrive as Lua values; objects as userdata
  if type(value) ~= "userdata" then return value end
  if value == js.null then return nil end
  local out = {}
  if js.global.Array:isArray(value) then
    for i = 0, value.length - 1 do out[i + 1] = to_lua(value[i]) end
  else
    local keys = js.global.Object:keys(value)
    for i = 0, keys.length - 1 do out[keys[i]] = to_lua(value[keys[i]]) end
  end
  return out
end
site = to_lua(js.global.__scrovData)
function print(...)
  local parts = table.pack(...)
  for i = 1, parts.n do parts[i] = tostring(parts[i]) end
  js.global:__scrovPrint(table.concat(parts, "\\t"))
end
`;

const lua = dataPromise.then((data) => {
	if (!window.fengari) throw new Error("Fengari failed to load");
	window.__scrovData = data;
	window.__scrovPrint = (text) => luaLines.push(text);
	window.fengari.load(LUA_PRELUDE, "prelude")();
});

/** Run Lua, show the outcome in the page and return what it printed. */
function runLua(code) {
	luaLines = [];
	const started = performance.now();
	try {
		const result = window.fengari.load(code, "lab")();
		if (result !== undefined) luaLines.push(String(result));
	} catch (error) {
		const message = `lua: ${error?.message ?? error}`;
		show(luaOutput, [...luaLines, message].join("\n"), true);
		luaStatus.textContent = "failed";
		throw new Error(message);
	}
	const text = luaLines.join("\n") || "(no output)";
	show(luaOutput, text, false);
	luaStatus.textContent = `${(performance.now() - started).toFixed(1)} ms`;
	return text;
}

lua.then(
	() => {
		luaStatus.textContent = "ready";
		luaRun.disabled = false;
	},
	(error) => {
		luaStatus.textContent = `Lua failed to start: ${error?.message ?? error}`;
	},
);

$("lua-form").addEventListener("submit", (event) => {
	event.preventDefault();
	try {
		runLua($("lua-input").value);
	} catch {
		// The failure is already shown in the output pane.
	}
});

// ── for agents ───────────────────────────────────────────────────────────────

window.scrovLab = {
	async sql(query) {
		$("sql-input").value = query;
		return runSql(query);
	},
	async lua(code) {
		await lua;
		$("lua-input").value = code;
		return runLua(code);
	},
};
