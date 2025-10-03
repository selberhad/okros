# Toy 8 — Telnet + MCCP Fragmentation (SPEC)

## Objective
Validate a byte-stream pipeline that handles Telnet IAC negotiation and MCCP decompression when data arrives in arbitrarily fragmented chunks, matching `Session.cc` + `mccpDecompress.c` behavior.

## Scope
- Telnet parser: IAC, WILL/WONT/DO/DONT, SB/SE (subnegotiation), GA/EOR prompts; ignore/strip telnet commands from app text.
- MCCP integration: Feed inbound bytes through decompressor; support partial input and incremental output; track error state.
- State/carry-over: Maintain input buffer + position across reads; detect incomplete IAC/option sequences and resume on next chunk.
- Responses: Generate required replies (e.g., WILL/WONT/DO/DONT) and queue them for write.

## Inputs
- Sequence of inbound byte chunks (test vectors) mixing: plain text, IAC sequences split across boundaries, MCCP-compressed data.
- Configuration flags: enable/disable MCCP, telnet options supported.

## Outputs
- Decompressed, telnet-clean text chunks delivered to the client (e.g., OutputWindow).
- Outbound response bytes (negotiation replies) in correct order.
- Prompt events when GA/EOR encountered (for input line updates).

## Behavior
- Parse loop consumes as much as possible; on partial IAC/subneg, store remainder and return.
- MCCP: Once enabled, pass bytes to `mccpDecompress_*` and read available output until drain; on error, close session.
- Prompt handling: When GA/EOR encountered, call prompt hook with current line buffer.
- Bounds: Respect `MAX_MUD_BUF`-like capacities; on overflow, set error to buffer overflow (aligning with reference semantics).

## Success Criteria
- Deterministic transformation of test vectors: same clean text and same response bytes as C++ reference.
- No dropped or duplicated bytes across chunk boundaries.
- Robust against pathological fragmentation (1 byte per chunk) without busy looping.

## Test Plan
- Unit vectors:
  - Split IAC sequences over N boundaries (N∈{1..4}).
  - Mixed MCCP/plain: enable -> compressed payload -> disable.
  - GA/EOR prompt detection with and without trailing text.
  - Error paths: malformed IAC, decompressor error → session close.
- Golden tests: Compare outputs to captured C++ pipeline for identical inputs.

## Non-Goals
- Network sockets themselves (use in-memory buffers for determinism).
- Full option matrix; implement only options observed in MCL flows.

