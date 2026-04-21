### Task Management
We use balls `bl` to do task tracking. Never commit directly on main. Invoke it for all task execution. Never ever make commits directly on main; all changes occur in a worktree and are merged in. Merges are always no-ff, to ensure that the merge is clean and representative.

When creating a task, always create the following gates:
- tests; ensure test coverage is at 100% and all tests pass. If there's something broken, you have to fix it before merge, no exceptions.
- docs: make sure the docs have been updated to represent the current state.
- alignment: check that the implementation is coherent against the various docs, such as docs/TAXONOMY.md.

### Terminology discipline
Terms of art used in code, docs, prompts, or commit messages must have an explicit definition in `docs/TAXONOMY.md` or in the document introducing them (e.g. `docs/ARCHITECTURE.md` §2.1). Any undefined term of art requires user approval before use. When in doubt, check the taxonomy first, then ask — do not coin silently. Banned terms are listed in `docs/ARCHITECTURE.md` §2.1 (currently: bare "call", "turn", "session", "compression" in the context-management sense).
