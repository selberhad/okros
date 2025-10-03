# Toy 8 — Telnet + MCCP Fragmentation: LEARNINGS

## Step 0 — Learning Goals
- IAC state machine: Handle telnet commands (IAC, WILL/WONT/DO/DONT, GA, EOR) split across reads; track carry‑over correctly.
- Prompt detection: Confirm GA/EOR prompt semantics and propagation to InputLine; avoid false positives.
- MCCP pipeline: Integrate `mccpDecompress` behavior; feed partial chunks; verify output integrity and error paths.
- Response emission: Ensure required replies are generated and written in correct order while input continues buffering.
- Buffer limits: Choose safe sizes mirroring MAX_MUD_BUF; define behavior on overflow (error vs drop vs truncate).
- Test vectors: Build deterministic sequences covering split IACs, compressed/uncompressed interleaves, and errors.

