# Toy 9 — Nonblocking Connect Semantics: LEARNINGS

## Step 0 — Learning Goals
- EINPROGRESS flow: Set non‑blocking, call `connect`, handle EINPROGRESS; confirm readiness on write.
- Completion check: Validate reference’s `getpeername` approach vs `getsockopt(SO_ERROR)` and align with C++ behavior.
- Address setup: Capture local/remote addresses and ports post‑connect; parity with reference getters.
- Error paths: Timeouts, DNS failure, refused, network unreachable; produce stable error text mapping.
- Event loop: Ensure fd is placed in read/write sets correctly while connecting and when outbound buffer has data.
- Cleanup: Confirm final flush and close behavior on teardown.

