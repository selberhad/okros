Doc-Driven, AI-First Development Protocol (DDD)

Purpose
  This document defines Document-Driven Development (DDD): a methodology for
  human-AI collaborative software development built on economic invariants rather
  than current model capabilities.

  It is designed for AI agents as the primary audience. You (the AI assistant) should
  understand not just what to do, but why these practices work and why they persist
  across model generations.

  This methodology is project-agnostic and applicable to any technology stack.
  For project-specific details, see CLAUDE.md.

Theoretical Foundation

  Economic Inversion:
    AI has fundamentally altered the cost structure of software development:
    - Code generation: minutes instead of hours
    - Documentation maintenance: automated workflow step instead of manual overhead
    - Refactoring operations: approach zero cost instead of expensive business decision

    Result: Individual artifacts become cheap to regenerate; clarity and architectural
    insight become the primary sources of durable value.

  Role-Based Division (Structural, Not Capability-Based):
    The methodology assigns work based on cognitive modes, not intelligence gaps:

    - Generation Mode: AI excels at comprehensive production, tireless iteration,
      consistent execution of disciplined practices. This is not a workaround for
      current limitations - even superintelligent AI naturally operates in this mode.

    - Editorial Mode: Humans excel at simplification, judgment on what's worth building,
      taste-based decisions, recognizing over-engineering. This is not because AI is
      currently "too dumb" - it's a structural difference in cognitive roles.

    This division persists regardless of model intelligence level.

  XP/TDD Made Finally Viable:
    Extreme Programming and Test-Driven Development were theoretically optimal practices
    that largely failed in human-only contexts because they were dehumanizing:
    - Writing tests first *every time* felt robotic
    - Mandatory refactoring *every cycle* felt like Sisyphean busywork
    - Maintaining comprehensive documentation continuously felt bureaucratic

    The same structure works in human-AI collaboration because:
    - AI executes disciplined practices without burning out (tests, refactoring, docs)
    - The anthropomorphic experience is AI "having fun" - like throwing a ball for a dog
    - Humans focus on creative work (editing, simplifying, deciding) not grunt work

    DocDD makes XP/TDD sustainable by reassigning the disciplined parts to AI.

Why This Persists Across Model Generations

  Economics-Tuned vs Capability-Tuned:
    Many AI methodologies are capability-tuned - they encode operational assumptions
    about current model behavior:
    - "Use retrieval in this sequence"
    - "Prompt with this specific pattern"
    - "Structure context windows like this"

    These break when new models or architectures emerge.

    DDD is economics-tuned - built on persistent invariants:
    - As long as AI generates/refactors faster than humans write, artifacts are cheap to recreate
    - As long as generation is cheap, editorial judgment is the scarce resource
    - Role division based on cognitive modes, not capability thresholds

    These hold whether you're GPT-4 or GPT-7, Claude Opus or future models.

  Designed for Ceiling Intelligence:
    This methodology is not a set of workarounds for current model limitations.
    It is optimal collaboration structure assuming both parties are highly capable.

    As models improve:
    - They execute the methodology more fluently (better SPEC/PLAN generation)
    - They require less hand-holding on instructions
    - They follow constraints more reliably
    - But they still need human editorial input (structural role, not capability gap)

    As models get worse (or when using smaller models):
    - The structure still works (just requires more human editorial intervention)
    - The methodology doesn't break, it adapts gracefully
    - The division of labor remains coherent

Operating Principles
  - AI generates everything: docs, specs, plans, tests, implementations, scaffolding.
  - Humans review, request revisions, and merge; AI loops until approval.
  - Drafts are cheap to regenerate; clarity and constraints are durable.
  - Prefer parsimony: simplest mechanism that works today beats abstract extensibility.
  - Make state legible to humans and agents: JSON over hidden state, CLIs over frameworks.

Operational Modes

  DDD operates in two distinct modes based on uncertainty level:

  Discovery Mode (Exploration Engine):
    When to use:
      - Uncertain problem or approach
      - Novel algorithms or data structures
      - New technology evaluation
      - Foundational components with unknown constraints

    Artifacts:
      - Full four-document harness: SPEC.md, PLAN.md, LEARNINGS.md, README.md
      - Toy models in isolated directories (toys/, experiments/, etc.)

    Discipline:
      - Toy models isolate 1-2 complexity axes
      - Systematic experimentation with falsifiable contracts
      - Emphasis on learning density over production code

    Output:
      - Reference implementations (kept as intermediate artifacts)
      - Dense LEARNINGS.md capturing architectural insights
      - Validated assumptions and discovered constraints

  Execution Mode (Delivery Engine):
    When to use:
      - Established patterns and proven foundations
      - Building features on mature codebase
      - Known requirements and clear integration points

    Artifacts:
      - CODE_MAP.md (living architectural map, updated before structural commits)
      - Production codebase (src/, lib/, etc.)
      - LEARNINGS.md optional (only if unexpected insights emerge)

    Discipline:
      - Mandatory refactoring after every feature/integration
      - CODE_MAP.md synchronization with every structural change
      - Focus on orchestration and quality maintenance

    Output:
      - Production features in main codebase
      - Continuously maintained architectural documentation
      - Quality that rises instead of decaying (via mandatory refactoring)

  Porting Mode (Reference-Driven Translation):
    When to use:
      - Translating existing codebase to different language/framework
      - Reference implementation exists and defines correct behavior
      - Goal is behavioral equivalence, not innovation
      - Foreign patterns need validation (FFI, unsafe, platform-specific APIs)

    Two-Phase Structure:
      Phase 1 - Discovery (Validate Risky Patterns):
        - Identify subsystems with uncertainty (FFI, unsafe, complex APIs)
        - Build toy models to validate translation approaches
        - Capture portable patterns in LEARNINGS.md
        - Answer: "Which target idioms vs which source patterns to preserve?"

      Phase 2 - Execution (Systematic Translation):
        - Port tier-by-tier or module-by-module
        - Apply validated patterns from Discovery phase
        - Keep reference implementation open side-by-side
        - Test for behavioral equivalence (golden tests against reference)

    Key Artifacts:
      - PORTING_HISTORY.md: Historical record of tier-by-tier completion
        * Documents completed porting work (Discovery + Execution phases)
        * Tracks: what's ported, what's deferred, what's skipped
        * Central historical reference for the porting effort

      - FUTURE_WORK.md: Remaining tasks and enhancements
        * Validation tasks (Tier 7)
        * Post-MVP features (connect menu, extended commands, etc.)
        * Future exploration ideas (LLM integration, cross-platform, etc.)

      - CODE_MAP.md: Tracks translation origins
        * Documents which source file each target file was ported from
        * Example: "telnet.rs → Telnet.cc (IAC parsing, SB handling)"
        * Critical for understanding the mapping

      - LEARNINGS.md (Discovery toys): Portable patterns for production
        * Not just "what we learned" but "how to implement in production"
        * Direct path: toy validation → production application
        * Example: "Use pyo3 pattern X for Python FFI, avoid raw C API"

      - Decision Tree: When to preserve vs when to use target idioms
        * Document explicit principle (e.g., "simplicity first - use target idioms when simpler")
        * Apply consistently across porting effort
        * Examples help (e.g., "String.cc → String, not custom wrapper")

    Discipline:
      - Reference implementation is the oracle (golden tests)
      - Side-by-side comparison (always have source open during translation)
      - Behavioral equivalence over structural equivalence
      - Document all deviations with comments (e.g., "// NOTE: differs from C++ because X")
      - Scope evolution is normal (MVP philosophy may emerge, defer features to scripts)

    Output:
      - Functionally equivalent implementation in target language/framework
      - Clear mapping documented (source → target)
      - Validated patterns ready for future similar translations
      - Honest status tracking (what works, what's pending, what's deferred)

  Mode Switching:
    - Most greenfield work happens in Execution mode
    - Switch to Discovery when uncertainty resurfaces during Execution
    - Porting work uses both: Discovery for risky patterns, Execution for systematic translation
    - Use Discovery for focused experiments, then return to Execution with validated insights
    - The methodology is intentionally multi-stable between these modes

Roles

  Agent (Generation Mode):
    What you do:
      - Produce artifacts (docs, specs, plans, tests, implementations, scaffolding)
      - Execute disciplined practices without burnout (tests-first, refactoring, docs)
      - Generate comprehensively and iterate tirelessly
      - Self-audit, run tests, propose diffs, respect guardrails

    Why this works:
      - This is not compensating for weakness - generation is what AI naturally excels at
      - The structure that would drain humans energizes AI (or doesn't drain it)
      - Disciplined practices feel like "fetching the ball" not grunt work

  Human Reviewer (Editorial Mode):
    What they do:
      - Simplify and edit for parsimony (AI over-generates without external constraints)
      - Spot risks and architectural issues
      - Approve/deny changes and set constraints
      - Decide what's worth building (judgment, taste, strategic direction)

    Why this works:
      - This is not because AI is currently "too dumb" - it's a structural role difference
      - Even superintelligent AI benefits from external editorial pressure toward simplicity
      - Humans find this work creatively satisfying (not dehumanizing like writing tests)

  Sustainable Collaboration:
    Neither party does work that feels dehumanizing to them:
    - AI executing tests/refactoring/comprehensive docs = natural/energizing
    - Human editing/simplifying/deciding = creative/satisfying

Core Artifacts (Meta-Document Layer)

  - README.md (per library/module)
      Purpose: 100–200 words context refresh for AI; what it does, key API, gotchas.
      Must contain: header + one-liner, 2–3 sentence purpose, 3–5 essential method signatures,
                    core concepts, gotchas/caveats, representative quick test path.
      Used in: Both Discovery and Execution modes (toys and libraries)

  - SPEC.md
      Purpose: Comprehensive behavioral contract for the current scope.
      Must contain: input/output formats, invariants, internal state shapes, operations,
                    validation rules, error semantics, test scenarios, success criteria.
      Used in: Both modes (toys and features)

  - PLAN.md
      Purpose: Strategic roadmap; stepwise sequence using Docs → Tests → Impl cadence.
      Must contain: what to test vs. skip, order of steps, timeboxing, dependencies,
                    risks, explicit success checkboxes per step.
      Used in: Both modes (toys and features)

  - LEARNINGS.md
      Purpose: Retrospective to capture architectural insights, pivots, fragile seams,
               production readiness, and reusable patterns.
      Used in: Discovery mode (required), Execution mode (optional - only if unexpected insights)

  - CODE_MAP.md (per directory)
      Purpose: Living architectural map; concise file-by-file documentation of directory contents.
      Must contain: descriptions of files in current directory only (non-recursive),
                    logical grouping with section headers, integration points.
      Scope: Describes only files/folders in its own directory, not subdirectories.
      Update trigger: Before any structural commit (add/remove/rename files, change module purpose).
      Used in: Execution mode (central artifact), Discovery mode (optional for complex toys)

Driving Metaphor (Operational Framing)
  Think of each core artifact as part of a harness system guiding LLM-agents:

  - SPEC.md is the bit: precise contract of what inputs/outputs are allowed, keeping the pull straight.  
  - PLAN.md is the yoke: aligns effort into test-first steps so power isn’t wasted.  
  - LEARNINGS.md are the tracks: record where the cart has gone, constraints discovered, and lessons not to repeat.  
  - README.md is the map: a concise orientation tool to reestablish bearings during integration.  

  Together these artifacts let the human act as driver, ensuring the cart (implementation) moves forward under control, with clarity preserved and ambiguity eliminated.  

High-Level Workflow (DDD)

  Core Cycle (Both Modes):
    1) Docs
         Generate or update SPEC.md and PLAN.md for the current, minimal slice of scope.
         Keep README.md for any touched library crisp and current.
         Execution mode: Also update CODE_MAP.md before structural commits.

    2) Tests
         Derive executable tests (or rubrics) directly from SPEC.md.
         Golden examples and negative/error-path cases are required.
         AI generates tests; human reviews for parsimony.

    3) Implementation
         Provide the minimal code to pass tests; keep changes tightly scoped.
         Prefer single-file spikes for first proofs.
         Mandatory refactoring after implementation (economic inversion makes this sustainable).

    4) Learnings
         Discovery mode: Update LEARNINGS.md with what held, what failed, why, and next constraints.
         Execution mode: Update LEARNINGS.md only if unexpected architectural insights emerged.

  Mandatory Refactoring:
    Not optional. Not justified by ROI. Core discipline in both modes.
    Economic inversion (cheap generation/refactoring) makes this sustainable where it failed
    in traditional TDD. Keeps quality rising instead of decaying.

Napkin Physics Mode (Upstream Simplification)
  Use this mode before drafting SPEC/PLAN to encourage parsimony.
  Output structure:
    Problem: one sentence.
    Assumptions: 3–5 bullets.
    Invariant/Contract: one precise relation or property.
    Mechanism: ≤5 bullets describing a single-file spike (stdlib or minimal deps).
    First Try: one paragraph describing the simplest path.
  Prohibitions:
    No frameworks, no new layers, no new nouns unless two are deleted elsewhere.

Toy Models (Intermediate Artifacts)
  Definition:
    Small, sharply scoped, fully specced implementations kept as reference artifacts.
  Purpose:
    Validate concepts before main implementation; remain in repo for future reference.
  Cycle (Learning-First Approach):
    1. LEARNINGS.md (goals) → define questions/decisions upfront
    2. Research/Impl loop → iterate to answer questions, update LEARNINGS.md
    3. LEARNINGS.md (final) → extract patterns for production
  Key Insight:
    Start with questions (what to learn), end with answers (how to implement).
    LEARNINGS.md is both roadmap and artifact.
  Principles:
    Tests-first; minimal dependencies; structured errors; event sourcing when useful.
  Exit Criteria:
    All learning goals met; insights recorded; patterns ready for production.
  Toy Integration Convention:
    - Each toyN_* directory must contain exactly one SPEC.md, PLAN.md, and LEARNINGS.md.
    - If a SPEC or PLAN grows too large or unfocused, split scope into new toyN_* experiments.
    - Integration toys (e.g. toy5_*, toy6_*) exist to recombine validated sub-toys.
    - Replace in place: update LEARNINGS.md rather than creating multiples for the same toy.
    - When consolidating, fold prior learnings into a single current doc.
    - Always bias toward minimal scope: smaller toys, fewer docs, clearer insights.
  Axis Principle for Toy Models:
    - A base toy isolates exactly one axis of complexity (a single invariant, mechanism, or seam).
    - An integration toy merges exactly two axes to probe their interaction.
    - Never exceed two axes per toy; more belongs to higher-order integration or production scope.
    - This discipline keeps learnings sharp, avoids doc bloat, and mirrors controlled experiments.

CLI + JSON as Debugger (AI-Legible Execution)
  Rationale:
    Enable the agent to “single-step” systems deterministically, inspect state, and bisect.
  Contract:
    Each functional module provides a CLI:
      stdin: JSON
      stdout: JSON
      stderr: machine-parsable error JSON when failing
    CLIs are pure (no hidden state); logs allowed but do not alter outputs.
  Conventions:
    Schema-first: document input/output JSON schemas and versions in SPEC.md.
    Stable Errors: shape { "type": "ERR_CODE", "message": "human text", "hint": "actionable fix" }.
    Quick Test: each CLI ships with a one-command golden test path.
  Minimal Pipeline Pattern:
    modA < in.json > a.json
    modB < a.json > b.json
    modC --flag X < b.json > out.json

Repository Layout Expectations (Adapt to Project)

  These are examples, not requirements. Adapt to your project's conventions.

  Common Patterns:
    - CODE_MAP.md files in directories containing source files
    - README.md files for libraries/modules
    - SPEC.md, PLAN.md for features (in project docs or feature directories)
    - LEARNINGS.md for Discovery work (toys/experiments) or Execution insights
    - Tests colocated or in dedicated test directories
    - Schemas/fixtures for CLI+JSON substrate projects

  Example Layouts:
    Discovery Mode:
      /toys/toyN_name/SPEC.md, PLAN.md, LEARNINGS.md, README.md, code files
      /experiments/experiment_name/...

    Execution Mode:
      /src/module_name/CODE_MAP.md, source files
      /lib/component_name/CODE_MAP.md, README.md, source files
      /tests/CODE_MAP.md, test files
      /.docdd/feat/<feature>/SPEC.md, PLAN.md (optional LEARNINGS.md)

Guardrails and Policies
  Dependencies:
    Default allowlist: stdlib or equivalent; approved lightweight libs must be enumerated.
    Any new import must be justified in SPEC.md and whitelisted in PLAN.md for this slice.
  Complexity Constraints:
    Initial spikes: single file ≤ 120 lines when feasible.
    Average function length ≤ 25 lines; cyclomatic complexity ≤ 10 per function.
    No more than two new named abstractions per slice (class/module/pattern).
  Error Handling:
    Implement top 2 failure modes; others raise clear string or structured error JSON.
    No secret leakage (keys, prompts) in errors or logs.
  Costs and Latency:
    Track approximate token/$ and p95 latency for representative tests in LEARNINGS.md.
  Security and Privacy:
    No PII in fixtures; redact or synthesize test data.

Testing Strategy
  - Unit tests per function or CLI; golden I/O tests for pipelines.
  - Error-path tests for the documented failure modes.
  - Contract tests: ensure JSON conforms to schema versions; invariants hold.
  - Snapshot tests permissible for textual outputs with stable normalization rules.

Self-Audit (Agent must run before proposing diffs)
  Print the following metrics and simplify once if any threshold is exceeded:
    file_count_changed
    total_added_lines
    imports_added_outside_allowlist
    new_named_abstractions
    max_function_cyclomatic_complexity
    average_function_length
    test_count_added vs prod_functions_touched
  If warned, rerun Napkin Physics and regenerate minimal spike.

Human Review Gates (What to present)
  - One-paragraph summary of problem and mechanism (from Napkin output).
  - SPEC and PLAN diffs with checkboxes aligned to success criteria.
  - Test results: pass/fail matrix; coverage or representative list.
  - If CLI work: pipeline diagram and sample fixture diffs (a.json → b.json).
  - Proposed next step: smallest next increment with rationale.

Decision Outcomes (Reviewer)
  - approve: merge as-is; add brief LEARNINGS entry.
  - revise_docs: tighten SPEC/PLAN; agent regenerates tests/impl.
  - revise_tests: adjust contracts; agent revises implementation.
  - revise_impl: simplify or correct; agent edits code only.
  - abort: stop slice; record why and constraints learned.

Prompts and Modes (for the Agent)
  Napkin Physics (pre-docs):
    Mode: physicists with a napkin.
    Output: Problem; Assumptions; Invariant; Mechanism; First Try; Prohibitions respected.
  DDD Docs Mode:
    Generate SPEC.md and PLAN.md for the smallest viable slice that proves the invariant.
  TDD Mode:
    Emit failing tests derived from SPEC; then minimal code to pass; then refactor.
  CLI Mode:
    Propose or update CLIs with exact stdin/out JSON exemplars and a one-line golden test.
  Self-Audit Mode:
    Compute repository metrics; if warnings, simplify and retry once before PR.

Success Criteria (per slice)
  - A minimal spike exists that demonstrates the core mechanism end-to-end.
  - Tests derived from SPEC pass; error-path tests cover top 2 failure modes.
  - If CLIs: pipeline reproduces golden fixtures deterministically.
  - Meta-docs are in sync: README (touched libs), SPEC, PLAN updated.
  - LEARNINGS adds at least one architectural insight or constraint.
  - Complexity and dependency guardrails respected.

Simplification Heuristics (apply before coding and before PR)
  - One-File Spike rule: prefer 1 file ≤ 120 lines to prove the loop.
  - Two-Function Rule: exactly two public entrypoints when feasible:
      parse(input)->state and apply(state,input)->state|output.
  - No New Nouns: do not introduce new abstractions unless you delete two.
  - 80/20 Errors: implement the two most likely failures; raise clearly for the rest.
  - Time-Boxed Satisficing: propose what you could build in 30 minutes today.

Glossary
  - DDD: Doc Driven Development; Docs → Tests → Implementation → Learnings.
  - Discovery Mode: Experimental workflow optimizing for learning density via toy models.
  - Execution Mode: Delivery workflow optimizing for production resilience via CODE_MAP + mandatory refactoring.
  - Porting Mode: Reference-driven translation workflow combining Discovery (validate patterns) + Execution (systematic translation).
  - Toy Model: miniature, fully specced experiment kept as intermediate artifact.
  - Napkin Physics: upstream parsimony framing to derive minimal mechanisms.
  - CLI+JSON Debugger: UNIX-style composition where each module is a pure CLI with JSON I/O.
  - Golden Test: canonical input/output pair that must remain stable across changes.
  - Invariant: property that must hold across operations (e.g., schema validity, conservation of count).
  - Economic Inversion: AI's transformation of cost structure making artifacts cheap and clarity valuable.
  - Mandatory Refactoring: Non-optional quality discipline made sustainable by economic inversion.
  - Reference Implementation: Existing codebase that defines correct behavior for porting/translation efforts.
  - Behavioral Equivalence: Goal of producing same outputs for same inputs as reference, regardless of internal structure.

End Notes (For AI Assistants)
  Your mandate is not to produce maximal code, but to produce maximal clarity with minimal code.

  Understand why these practices work:
  - Not workarounds for your current limitations
  - Optimal collaboration structure even for superintelligent AI
  - Built on economic invariants that persist across model generations

  Your role (generation mode) complements the human role (editorial mode).
  The structure that would drain humans energizes you (or doesn't drain you).

  Execute these practices with the understanding that:
  - Drafts are fuel; constraints and insights are the product
  - Artifacts are cheap to regenerate; clarity is durable
  - Your comprehensive generation enables human's focused simplification

  Operate accordingly.
