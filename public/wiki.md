# scrov — spec

> **Status: planned.** Nothing described here ships yet. This file is the design, and it changes as the work does.
> Landing page: https://scrov.app/platform/ · Code: https://github.com/scrov/bot · Site: https://github.com/scrov/scrov
> Last updated: 2026-11-07

## 1. What scrov is

scrov is a voice agent that turns one spoken request into many tool calls, one per device, and sends them at the same time ("multicast"). Devices take part by registering **WebMCP tools**, functions with a description and a JSON schema, from a page or a desktop window. The agent picks the tools, calls them in parallel, collects the results and answers once, out loud.

Example: *"Dim the living room lights, cast my focus playlist to the TV, and pin a 20-minute timer in my glasses"* becomes `ha_light_set`, `cast_play` and `glasses_timer_pin`, sent together.

## 2. Architecture

```
 reSpeaker Clip / browser mic
        │  16 kHz mono 16-bit PCM over WebSocket
        ▼
 Voice agent  (Cloudflare Agents, withVoice, one Durable Object per user)
   STT ─► onTurn() ─► LLM planner ─► dispatcher ─► TTS reply
                                       │  parallel tool calls
               ┌───────────────────────┼────────────────────────┐
               ▼                       ▼                        ▼
      Browser bridge             Desktop hub              Discord bot
   (scrov extension)        (Tauri v2 + Leptos SSR)   (scrov/bot, Workers)
               │                       │
     pages with WebMCP tools   its own pages with tools
               │                       │
   Home Assistant · Cast ·      notifications · URLs ·
   VITURE glasses · Clip        clipboard · glasses
```

## 3. Components

### 3.1 Input: reSpeaker Clip or a browser mic
- reSpeaker Clip (Seeed): two PDM mics, 360° pickup up to 3 m, BLE 5.3 and Wi-Fi 6, a 0.5" OLED, vibration feedback, 2 GB storage, open-source firmware (Apache-2.0) and an SDK.
- Planned firmware role: stream 16 kHz mono 16-bit PCM to the voice agent over Wi-Fi, the format the Cloudflare voice client already sends. BLE handles pairing and setup.
- The Clip is also an output: `clip_oled_show` and `clip_vibrate` confirm results without speech.
- Any browser mic works through the Agents voice client (`VoiceClient` or the `useVoiceAgent` React hook).

### 3.2 Voice agent
- `withVoice(Agent)` from the `agents` package (`agents/voice`; this replaces the deprecated `@cloudflare/voice`).
- Default providers: `WorkersAIFluxSTT` for speech-to-text and `WorkersAITTS` for text-to-speech. Planned alternative: an Azure Speech adapter for both.
- Built-in behavior used as is: continuous STT with model-driven turn detection, sentence-chunked streaming TTS, interruption handling, and conversation persistence in the Durable Object's SQLite storage.
- `onTurn(transcript, context)` runs the planner (§3.4) and returns the spoken summary.

### 3.3 Tool registry
- Each bridge opens a WebSocket to the user's agent and announces its tools: `{ hostId, device, tools: [{ name, description, inputSchema }] }`.
- The agent keeps the registry in Durable Object state. A tool is removed when its host disconnects or when the page aborts the tool's `AbortSignal`.
- scrov keeps extra metadata on its side, not in WebMCP: `confirm: true` for tools that need a spoken "yes" first, and `timeoutMs` for each tool.

### 3.4 Planner and multicast dispatcher
1. The LLM receives the transcript plus the current tool list and returns a plan: a list of `{ callId, tool, args }`.
2. The dispatcher groups calls by host and sends them **in parallel**. Each call carries an idempotency key (`callId`).
3. Calls to `confirm` tools wait for a spoken confirmation. Every other call goes out right away.
4. Each call resolves to `ok`, `error` or `timeout` (per-tool timeout, default `[TBD] ms`). A failed call never blocks the others.
5. The agent speaks one summary that names any failures: "Lights and playlist done. The glasses didn't answer."

### 3.5 Bridges
WebMCP tools live in pages. Something has to carry calls from the cloud agent to those pages:

- **Browser bridge (extension).** WebMCP lets agents that run in extensions call page tools. The scrov extension connects to the agent, discovers tools in open scrov pages, and relays calls and results.
  *Open question:* how the extension API to WebMCP takes shape in Chrome and Edge, and whether Firefox adopts WebMCP at all (standards-position discussion is open).
- **Desktop hub.** Generated from the `tauri-leptos-ssr` template: Tauri v2 with Leptos SSR, the Axum server embedded, and one native binary. It serves its own tool pages in its WebView and relays like the extension does.
  *Fallback:* if the system WebView has no WebMCP, the hub announces the same tool schemas straight over its agent WebSocket. The tool contract doesn't change.

### 3.6 Devices and tools

| Device | Reached through | Tools (initial set) |
|---|---|---|
| Home Assistant | ESPHome native API, through Home Assistant | `ha_light_set`, `ha_scene_run`, `ha_sensor_read` |
| Google Cast | Cast Web Sender SDK (`CastContext`, `RemotePlayerController`) | `cast_play`, `cast_pause`, `cast_volume_set` |
| VITURE XR glasses | VITURE XR Glasses SDK (Android, Linux, Windows, macOS: 3DoF/6DoF tracking, IMU, camera, device control) | `glasses_overlay_show`, `glasses_timer_pin`, `glasses_recenter` |
| Desktop | Tauri v2 + Leptos SSR hub | `desktop_notify`, `desktop_open_url`, `desktop_clipboard_write` |
| reSpeaker Clip | Open firmware + SDK | `clip_oled_show`, `clip_vibrate` |
| Discord | scrov/bot on Cloudflare Workers (HTTP interactions) | `discord_post`, `discord_status` |

Every row is planned.

## 4. Tool conventions

- **Names** use `[a-z0-9_]` and start with the device prefix: `cast_play`, not `cast.play` or `castPlay`. MCP clients don't all accept dots.
- **Descriptions** are one sentence that says what happens and to what: "Play a playlist on the Cast device in a room."
- **Schemas** are JSON Schema `object`s with `required` filled in. Rooms and areas are `snake_case` ids that match Home Assistant areas.
- **Results** return `{ content: [{ type: "text", text }] }`, where `text` is one short sentence the agent can speak as is.
- **Lifetime:** register only what the current screen can do, and abort the signal when it can't any more.

```js
const tools = new AbortController();
await document.modelContext.registerTool({
  name: "cast_play",
  description: "Play a playlist on the Cast device in a room",
  inputSchema: {
    type: "object",
    properties: {
      room: { type: "string", description: "Room name, e.g. living_room" },
      playlist: { type: "string" }
    },
    required: ["room", "playlist"]
  },
  async execute({ room, playlist }) {
    await playOnCast(room, playlist);
    return { content: [{ type: "text", text: `Playing ${playlist} in ${room}` }] };
  }
}, { signal: tools.signal });
```

## 5. One turn, step by step

1. The mic streams PCM to the agent. STT detects the end of the utterance.
2. `onTurn` sends the transcript and the tool list to the LLM, which returns three calls.
3. The dispatcher sends `ha_light_set` and `cast_play` to the browser bridge and `glasses_timer_pin` to the desktop hub, all at once.
4. Results come back as they finish. The agent waits until all have settled or timed out.
5. TTS speaks: "Lights at 30%, focus playlist on the TV, timer pinned." `clip_vibrate` buzzes once.

## 6. Security and privacy

- Secrets stay in Cloudflare (`wrangler secret`): the Discord bot token, the Discord public key, and the bridge keys. Browsers and the Clip never see the bot token.
- Each bridge authenticates to the agent with its own revocable key.
- The agent runs on the user's own Cloudflare account. The conversation text persists in that agent's SQLite storage. Raw audio isn't stored.
- Tools that unlock, buy, delete or send messages to other people are registered with `confirm: true`.
- Discord requests are checked against the application's Ed25519 public key before anything runs.

## 7. Discord bot and install script

- **Bot (scrov/bot):** answers over HTTP interactions from a Cloudflare Worker, with no gateway process. Planned commands: `/status`, `/metrics`, `/deploy` (maintainers only) and `/wiki`.
- **Install script:** https://gist.github.com/scrov/e7742404d04727602473a038acdd3ecd (`.i.sh`). Today it is a placeholder. Planned steps:
  1. Check for `git`, a JavaScript runtime (Node or Bun) and `wrangler`.
  2. Clone `scrov/bot`.
  3. Ask for `DISCORD_APPLICATION_ID`, `DISCORD_PUBLIC_KEY` and `DISCORD_TOKEN`, and store them with `wrangler secret put`.
  4. Run `wrangler deploy`, register the slash commands, and print the interactions URL to paste into the Discord developer portal.
- Read any script before piping it to `sh`.

## 8. Licenses

scrov's own code is open source: `scrov/bot` is released under The Unlicense. Third-party code keeps its own license, and scrov follows it.

| Component | License | What scrov does |
|---|---|---|
| Cloudflare Agents SDK (`agents`) | MIT | Dependency |
| ONNX Runtime | MIT | Dependency for on-device inference |
| ONNX | Apache-2.0 | Model format |
| Ultralytics YOLO11 | AGPL-3.0 | Any scrov part that ships YOLO11 code or weights is released under AGPL-3.0, or uses an Ultralytics Enterprise license |
| ESPHome | MIT (Python) · GPL-3.0 (C++ runtime) | Firmware built from the C++ runtime is shared under GPL-3.0 |
| Home Assistant | Apache-2.0 | Integrated through its API, not bundled |
| Tauri v2 | MIT / Apache-2.0 | Desktop hub framework |
| Leptos | MIT | Desktop hub UI |
| tauri-leptos-ssr | MIT | Template the hub is generated from; notice kept |
| reSpeaker Clip firmware (Seeed) | Apache-2.0 | Firmware base; NOTICE kept |
| Azure Speech SDK | Microsoft Speech SDK license (samples MIT) | Optional provider, installed from Microsoft, not bundled |
| Google Cast Web Sender SDK | Google APIs Terms of Service | Loaded from Google at runtime, not bundled |
| VITURE XR Glasses SDK | VITURE SDK terms | Downloaded from VITURE, not bundled |
| Inter, JetBrains Mono (site fonts) | SIL OFL 1.1 | Self-hosted with license files in `public/platform/fonts/` |

## 9. Open questions

- Extension and WebView access to WebMCP (§3.5) in each browser, and whether Firefox supports it.
- The end-to-end latency budget, from the end of speech to the first device acting: `[TBD]`.
- Which LLM plans the calls, and how many tools fit in one prompt before tools need to be grouped by room.
- How the Clip carries audio: Wi-Fi straight to the agent, or BLE to a phone or the desktop hub that relays it.
- How `confirm: true` sounds in practice without slowing down everyday commands.

## 10. References

- WebMCP explainer: https://github.com/webmachinelearning/webmcp
- WebMCP implementation status: https://github.com/webmachinelearning/webmcp/blob/main/implementation-status.md
- Cloudflare Agents voice: https://developers.cloudflare.com/agents/agent-class/communication-channels/voice/
- reSpeaker Clip: https://www.seeedstudio.com/respeaker-clip-wearable-ai-recorder.html
- VITURE developer: https://www.viture.com/developer
- Home Assistant ESPHome integration: https://www.home-assistant.io/integrations/esphome/
- Google Cast Web Sender: https://developers.google.com/cast/docs/web_sender/integrate
- Azure Speech SDK samples: https://github.com/Azure-Samples/cognitive-services-speech-sdk
- tauri-leptos-ssr: https://github.com/codeitlikemiley/tauri-leptos-ssr
