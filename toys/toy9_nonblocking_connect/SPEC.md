# Toy 9 — Nonblocking Connect Semantics (SPEC)

## Objective
Validate the nonblocking `connect()` flow and readiness-driven completion/error reporting to match `Socket.cc` behavior (EINPROGRESS, write readiness, endpoint introspection).

## Scope
- Socket setup: AF_INET TCP, set O_NONBLOCK (or FNDELAY), optional SO_REUSEADDR for server case.
- connect(): Expect immediate success or EINPROGRESS; track `waitingForConnection`.
- Readiness: On writability, determine success/failure and finalize remote/local endpoints.
- Introspection: Populate `remote`/`local` and expose getters for ports/IPs.
- Error mapping: DNS failure, refused, unreachable, timeout; stable error strings.

## Inputs
- Host: `127.0.0.1` (loopback) or mock server; invalid host for DNS error; closed port for refusal.
- Async flag: on (nonblocking) vs off (optional check; primary focus is async).

## Outputs
- State transitions: `connecting` → `connected` or `error`.
- Events: `connectionEstablished()` callback on success; `errorEncountered(errno)` on error.
- Endpoints: `getLocalPort()`, `getRemotePort()`, `getLocalIP()`, `getRemoteIP()` populated.

## Behavior
- After EINPROGRESS, place fd in write set; on readiness, use `getsockopt(fd, SOL_SOCKET, SO_ERROR)` to determine result, then call `getpeername`/`getsockname`.
- Preserve reference semantics (which rely on `getpeername`/`getsockname`) while using SO_ERROR as the decisive indicator.
- Buffering: Allow queued outbound data to flush as soon as writable; trigger `outputSent()` when buffer drains.

## Success Criteria
- Deterministic transitions and correct endpoints across cases: success, refused, unreachable, bad DNS, timeout.
- Outbound buffer drains post-connect; no spurious read readiness before connected.

## Test Plan
- Loopback server: Start a TCP listener; connect nonblocking; assert established and endpoint info.
- Refusal: Connect to a closed port; assert error string and no endpoints set.
- DNS failure: Use invalid host; assert error mapping.
- Timeout: Connect to unroutable IP; ensure timeout behavior is handled (configurable).

## Non-Goals
- Cross-platform portability beyond Linux/macOS.
- TLS or higher-level protocols.

