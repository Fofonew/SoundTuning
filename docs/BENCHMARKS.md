# Journal des benchmarks

Commande : `cargo bench --workspace --bench '*' -- --noplot` (release, Apple Silicon de développement).
Budgets : `.claude/rules/realtime.md`. Un bench au-dessus de son budget est une régression.

| Date | Phase | Bench | Médiane | Budget | Variante | Notes |
|---|---|---|---|---|---|---|
| 2026-10-26 | 2 | `engine_update` (FFT 32k × 2, accumulateurs, H1, γ², lissage 1/48 et projection complexe sur 957 points) | 0,61 ms | < 2 ms | référence | aucune optimisation nécessaire |
| 2026-10-26 | 2 | `delay_search` (FFT 131072 × 3, pic, interpolation, confiance) | 1,78 ms | < 15 ms | référence | |
| 2026-10-26 | 2 | `optimal_alignment` (±50 ms, p = ±1, bande 60–150 Hz, grossier puis fin) | 3,08 ms | < 50 ms | référence | |
