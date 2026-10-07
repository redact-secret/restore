# Claude Code

This is `redact-secret/restore`, the Rust controlled reconstruction engine for
the Redact Secret ecosystem. It owns scanning, planning, bulk authority interaction,
and all-or-nothing reconstruction; vault owns authorization and lifecycle state.

Shared repository instructions are maintained in one place:

@AGENTS.md

Repository skills are canonical in `.agents/skills/` and discovered through
relative directory symlinks in `.claude/skills/`. Edit the canonical files.
Preserve separately installed tool-managed skills such as `graft`.

Read the current issue and epic #1 for implementation work. In particular, #8
must establish and test consume/concurrency/cancellation semantics before any
public async API. Prefer sequential work on this low-memory device; when asked
to orchestrate subagents, use one worker at a time.
