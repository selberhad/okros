# LEARNINGS — Internal MUD for Testing & Offline Play

## Goals (What We Need to Learn)

### Primary Questions
1. **Interface Pattern**: How does an internal MUD feed the Session pipeline?
   - Fake socket with loopback?
   - Memory pipe/channel?
   - Direct feed to Session::feed()?
   - What's simplest for both testing and production?

2. **Telnet Protocol**: Should internal MUD emit real telnet sequences?
   - IAC commands (WILL/DO/DONT/WONT)?
   - GA/EOR for prompts?
   - MCCP compression negotiation?
   - Or bypass telnet layer entirely?

3. **ANSI Output**: What level of ANSI support?
   - Just SGR color codes (ESC[31m)?
   - Full attribute support (bold, underline)?
   - Can we test edge cases (fragmentation, reset)?

4. **State Management**: How to structure game state?
   - Room graph (nodes + edges)?
   - Item/inventory system?
   - Player state (location, inventory, flags)?
   - Serializable for save/load?

5. **Command Parser**: What's the minimal parser?
   - Simple verb-noun (go north, take sword)?
   - Aliases (n → go north)?
   - Error handling (unknown commands)?

6. **Testing Integration**: How to drive e2e tests?
   - Programmatic command injection?
   - Deterministic outcomes (no RNG)?
   - Observable state for assertions?
   - Can we run headless mode with internal MUD?

### Secondary Questions
7. **Feature Scope**: What's MVP vs nice-to-have?
   - MVP: 3-5 rooms, basic navigation, simple items
   - Nice: combat, NPCs, quests, persistence

8. **Production Integration**: How to wire into okros?
   - CLI flag (--offline, --demo)?
   - Feature gate (#[cfg(feature = "offline-mud")])?
   - Separate binary or integrated?

9. **Script Integration**: Can we implement MUD in Perl/Python?
   - Would dogfood plugin system
   - More flexible game content
   - But adds complexity to testing

## Decisions (To Be Made)

- [ ] Interface mechanism: fake socket vs memory pipe vs direct feed
- [ ] Telnet layer: full protocol vs bypass
- [ ] Game complexity: Zork-like adventure vs minimal test harness
- [ ] Implementation language: Rust vs Perl/Python plugin
- [ ] Integration approach: feature flag vs CLI mode vs separate tool

## Hypotheses (To Test)

1. **Fake Socket Hypothesis**: Using a socketpair() for loopback will let us reuse all existing Session pipeline code without modification
2. **Telnet Bypass Hypothesis**: We can skip telnet negotiation and just emit ANSI text, feeding Session after telnet parsing
3. **Minimal State Hypothesis**: A simple HashMap<RoomId, Room> + player state is sufficient for testing
4. **Script Plugin Hypothesis**: Implementing the MUD as a Python/Perl script will be more flexible than Rust

## Experiments to Run

1. Build minimal room graph (3 rooms, bidirectional navigation)
2. Test fake socket integration with Session pipeline
3. Emit ANSI color codes and verify scrollback rendering
4. Implement basic command parser (go, look, take, inventory, quit)
5. Drive via headless mode control server (JSON commands)
6. Measure: Can we write e2e tests with zero external dependencies?

## Success Criteria

- [ ] Can navigate between rooms using Session pipeline
- [ ] ANSI colors render correctly in scrollback
- [ ] Can run deterministic e2e test (command sequence → expected output)
- [ ] Works in headless mode (control server sends commands, reads output)
- [ ] Zero external dependencies (no real MUD server needed)
- [ ] Pattern extracted for production integration

## Anti-Goals

- Not building a full MUD engine (that's a different project)
- Not replacing real MUD testing (still need that for network/protocol validation)
- Not a user-facing game (just testing infrastructure with offline bonus)

---

## Findings (To Be Filled During Implementation)

_This section will document what actually worked, what failed, and why._
