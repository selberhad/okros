# Toy 8 — Telnet + MCCP Fragmentation: LEARNINGS

## Step 0 — Learning Goals
- IAC state machine: Handle telnet commands (IAC, WILL/WONT/DO/DONT, GA, EOR) split across reads; track carry‑over correctly.
- Prompt detection: Confirm GA/EOR prompt semantics and propagation to InputLine; avoid false positives.
- MCCP pipeline: Integrate `mccpDecompress` behavior; feed partial chunks; verify output integrity and error paths.
- Response emission: Ensure required replies are generated and written in correct order while input continues buffering.
- Buffer limits: Choose safe sizes mirroring MAX_MUD_BUF; define behavior on overflow (error vs drop vs truncate).
- Test vectors: Build deterministic sequences covering split IACs, compressed/uncompressed interleaves, and errors.

## Findings (Tests & Parity)

- Telnet parser parity:
  - GA/EOR detection generates prompt events; app text excludes telnet command bytes.
  - Special-case per reference: `WILL EOR (25)` → reply `DO EOR`.
  - Session ignores other options (e.g., NAWS, DO ECHO); MCCP replies are emitted by the decompressor layer, not by the session parser.
  - SB/SE subnegotiation content is ignored/stripped; `IAC IAC` inside SB is treated as literal 255 and discarded with the SB block.
  - 1-byte fragmentation across all cases (IAC, options, SB, GA/EOR) handled without dropping/duplicating bytes.
- MCCP scaffold:
  - Pipeline order matches reference: decompress → telnet parse.
  - Handshake semantics reproduced: `WILL COMPRESS2` → `DO COMPRESS2`; `WILL COMPRESS` accepted unless v2 already seen (then `DONT COMPRESS`).
  - Start sequences stripped (v1: `IAC SB 85 WILL SE`; v2: `IAC SB 86 IAC SE`).
  - Added simulated error sentinel and end-of-stream sentinel to validate error and EOS paths without zlib.
  - Verified response ordering correctness under mixed negotiations.

## Open Questions

- Real zlib integration: Replace stub with actual inflate state; ensure buffer and error semantics match `mccpDecompress.c`.
- Buffer sizing and overflow: Align with `MAX_MUD_BUF` behavior and define Rust-side limits + error reporting.
- Additional options: Any telnet options the reference implicitly accepts (beyond EOR)? Confirm and codify exceptions.
- Prompt handling with ANSI: Reference inserts a reset before prompts; do we replicate that here or defer to higher layers?
