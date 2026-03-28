# Code generation from OBS WebSocket spec

## Protocol spec source

- Machine-readable: https://raw.githubusercontent.com/obsproject/obs-websocket/master/docs/generated/protocol.json
- Human-readable: https://github.com/obsproject/obs-websocket/blob/master/docs/generated/protocol.md

## protocol.json structure

The canonical structured spec for code generation. Top-level keys:

- `enums` — 7 enum types (EventSubscription, RequestStatus, WebSocketOpCode, etc.)
- `requests` — all request types with `requestType`, `category`, `requestFields[]`, `responseFields[]` (each field has `valueName`, `valueType`, `valueDescription`)
- `events` — all event types with `eventType`, `category`, `dataFields[]`

## Approach

The current handler (`obs-mock/src/handler.rs`) is handwritten. To keep in sync with spec updates:

1. Vendor or fetch `protocol.json` from the obs-websocket repo
2. Write a `build.rs` that parses `protocol.json` and generates handler match arms with default mock responses based on field types
3. Output generated code to `OUT_DIR` and include via `include!` macro in `handler.rs`

This avoids manually updating handlers when new request types are added to the spec.
