#!/usr/bin/env bash
#
# git-init.sh - Initialize MCL Rust Port repository with logical commit structure
#
# This script systematically commits the Discovery Phase work in logical groups
# using conventional commit format.

set -euo pipefail

# Ensure we run from the script directory (repo root expected)
cd "$(dirname "$0")"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== MCL Rust Port - Git Initialization (resume-safe) ===${NC}\n"

# Helper: commit only if there are staged changes
commit_if_any() {
  local msg="$1"
  if ! git diff --cached --quiet; then
    git commit -m "$msg"
  else
    echo "(no staged changes) — skip commit"
  fi
}

# Initialize git repository if needed
if [ ! -d .git ]; then
    echo -e "${GREEN}1. Initializing git repository${NC}"
    git init
    echo ""
else
    echo -e "${GREEN}1. Git repository already exists — resuming steps${NC}"
    echo ""
fi

# Clean up embedded git repositories inside toys/* (vendor content, not submodules)
echo -e "${GREEN}1a. Checking for embedded git repos under toys/${NC}"
if command -v find >/dev/null 2>&1; then
    EMBEDDED_REPOS=$(find toys -type d -name .git 2>/dev/null || true)
    if [ -n "${EMBEDDED_REPOS}" ]; then
        echo "Found embedded repos; removing their .git to vendor content:"
        echo "${EMBEDDED_REPOS}"
        # Remove each embedded .git directory
        while IFS= read -r gitdir; do
            rm -rf "$gitdir"
        done <<< "${EMBEDDED_REPOS}"
    else
        echo "No embedded repos found under toys/"
    fi
else
    echo "Skip embedded repo check (find not available)"
fi
echo ""

# Set up main branch (ok if already on main)
git branch -M main || true
echo ""

## Removed: steps already completed (docs, toys 1–3)

# Commit: Toy 4 - Python pyo3
echo -e "${GREEN}2. Committing Toy 4 (Python/pyo3)${NC}"
git add toys/toy4_python/

commit_if_any "feat(toy4): validate Python embedding via pyo3

Decision: Use pyo3 (simpler and safer than C API)

Key findings:
- pyo3 abstracts Python C API beautifully
- Automatic reference counting (no manual INCREF/DECREF)
- Result<> for error handling (no PyErr_Print)
- GIL management via Python::with_gil()
- All C++ patterns replicate cleanly

Tests (8/8 passing):
- Initialization
- Eval Python code
- Set/get variables (strings, integers)
- Call functions
- Load Python files
- Python computation (list comprehension)
- String manipulation
"
echo ""

# Commit: Toy 5 - Perl raw FFI
echo -e "${GREEN}3. Committing Toy 5 (Perl raw FFI)${NC}"
git add toys/toy5_perl/

commit_if_any "feat(toy5): validate Perl embedding via raw FFI

Decision: Use raw FFI with PERL_SYS_INIT3 for modern Perl

CRITICAL discovery: MCL targets Perl 5.10 (2007), modern Perl 5.34+
requires PERL_SYS_INIT3() for threaded builds (didn't exist in 5.10).

Key findings:
- PERL_SYS_INIT3() required before perl_alloc() (modern Perl)
- Function name mangling: Perl_ prefix (e.g., Perl_eval_pv)
- Threading context (pTHX_) becomes explicit first parameter
- Working init: sys_init3 → alloc → construct → parse → run
- perl_eval_pv, variable get/set all working

Tests:
- Eval \"2 + 2\" → Result: 4 (WORKS)
- Set/get Perl variables (WORKS)
- Full lifecycle (WORKS)

This unlocks 13k+ LOC perlmudbot integration.
"
echo ""

# Commit: .gitignore
echo -e "${GREEN}4. Committing .gitignore${NC}"
git add .gitignore

commit_if_any "chore: add .gitignore

Excludes:
- mcl-cpp-reference/ (separate repo, future submodule)
- perlmudbot/ (separate repo, future submodule)
- Rust build artifacts (target/, Cargo.lock)
- IDE/OS files
"
echo ""

# Summary
echo -e "${BLUE}=== Git initialization complete! ===${NC}\n"
git log --oneline --decorate
echo ""
echo -e "${GREEN}Repository initialized/resumed with Discovery Phase artifacts${NC}"
echo -e "${GREEN}Ready to begin Execution Phase (tier-by-tier porting)${NC}"
echo ""
echo "Next steps:"
echo "  1. Create root Cargo.toml"
echo "  2. Begin Tier 1 (Foundation) porting"
echo "  3. Follow IMPLEMENTATION_PLAN.md"
