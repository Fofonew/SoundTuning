---
name: verify
description: >
  Project verification gate for SoundTuning. Runs, in order and stopping at the first failure:
  cargo fmt check, clippy with -D warnings, the whole test suite, a Windows (x86_64-pc-windows-msvc)
  cargo check without the asio feature, then the criterion benches once they exist, and compares
  bench results with the performance budgets. Use at the end of every phase, before committing,
  or whenever asked to "verify", "run the checks" or "is the build green".
---

# verify

Run each step from the repository root, **sequentially**. After each step, show its full output
(or the relevant failing part if it is very long). **Stop at the first failing step**: report which
step failed, show its output, and do not run the following steps.

## Steps

1. Formatting
   ```bash
   cargo fmt --all -- --check
   ```
2. Lints
   ```bash
   cargo clippy --workspace --all-targets -- -D warnings
   ```
3. Tests
   ```bash
   cargo test --workspace
   ```
4. Windows cross-check (default features only, never `--features asio` or `--all-features`)
   ```bash
   cargo check --workspace --target x86_64-pc-windows-msvc
   ```
5. Benches — only if at least one criterion bench exists:
   ```bash
   ls crates/*/benches/*.rs 2>/dev/null
   ```
   If the list is non-empty:
   ```bash
   cargo bench --workspace --benches -- --noplot
   ```
   Then compare each reported median with the budgets of `.claude/rules/realtime.md`
   (`engine_update` < 2 ms, `delay_search` < 15 ms, `plot_frame_prep` < 1 ms,
   `optimal_alignment` < 50 ms). A bench over budget is a **failure** of this step.
   If no bench exists yet, print "benches: none yet (skipped)".

## Report

End with a short summary table:

| Step | Result |
|---|---|
| fmt | ✅ / ❌ / not run |
| clippy | … |
| test | … (passed / failed / ignored counts) |
| windows check | … |
| benches | … (medians vs budgets) / skipped |

Never edit code inside this skill, never loosen a lint, a test tolerance or a budget to make a step pass.
