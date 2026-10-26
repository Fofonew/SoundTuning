# SoundTuning

Outil desktop de mesure acoustique (double FFT, boucle ouverte) pour le calage de systèmes de sonorisation en live. Rust, macOS + Windows.

**Avant chaque phase : relire `docs/SPEC.md` en entier, puis la section « État » ci-dessous.**

## Priorités et arbitrage

1. Workflow de calage inspiré de SysTune (`docs/SPEC.md` §4) : mesure continue, jamais interrompue, jamais de modale.
2. Rapidité : ≥ 20 mises à jour/s des courbes, rendu 60 i/s découplé du moteur, une touche ou un clic par action.
3. Interface moderne, lisible à 1 m en régie et en plein soleil.

Ordre d'arbitrage : **justesse des mesures > réactivité > qualité de l'interface > nombre de fonctionnalités.**

## Règles non négociables

- Rust stable, édition 2024, workspace Cargo. Dépendances via `cargo add` uniquement, jamais de version inventée. Hors stack (`docs/SPEC.md` §9) → justification dans `docs/DECISIONS.md`.
- Code, identifiants, logs en anglais. Pas de commentaires d'explication : des noms explicites. Constantes numériques nommées.
- Interface en français ; tous les textes dans le module de textes de `crates/app` (aucune chaîne UI en dur ailleurs).
- Aucun `unwrap`/`expect` hors tests. `thiserror` (bibliothèques), `anyhow` (application), `tracing` (logs). Pas de `unsafe`.
- Callbacks audio : aucune allocation, aucun verrou, aucune I/O (détails : `.claude/rules/realtime.md`).
- Tests : signaux synthétiques et mode fichier WAV uniquement. Jamais de flux audio réel, jamais de fenêtre bloquante. Rendu hors écran `egui_kittest` autorisé.
- Ne rien pousser, aucun remote, rien modifier hors de ce dossier.
- Ne jamais assouplir un critère d'acceptation : s'il est impossible, s'arrêter et le dire.

## Architecture

| Crate | Rôle |
|---|---|
| `crates/dsp-core` | Maths pures, aucune I/O. Règles : `.claude/rules/dsp.md` |
| `crates/audio-io` | Périphériques, canaux, flux `cpal`, générateur, (dé)connexion. Règles : `.claude/rules/realtime.md` |
| `crates/engine` | Threads, alignement x/y, fenêtre glissante, moyennage, délai, instantanés. Règles : `.claude/rules/realtime.md` |
| `crates/app` | egui, système de design, état, annuler/rétablir. Binaire `soundtuning`. Règles : `.claude/rules/ui.md` |

Modes de référence : **interne** (défaut, signal du générateur, latence système calibrée) et **rebouclée** (entrée console). Voir `docs/SPEC.md` §2.1.
Mode « entrée fichier » : WAV 2 canaux (référence, mesure) à la place de la carte son.

## Commandes

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p dsp-core                                      # une crate
cargo check --workspace --target x86_64-pc-windows-msvc     # sans la feature asio
cargo bench --workspace --bench '*' -- --noplot               # benchs criterion
cargo run --release -p app -- --input-file <fichier.wav>    # mode fichier
cargo run -p app -- record --seconds 10 --output <out.wav>  # enregistrement CLI
```

Skill `verify` : enchaîne fmt, clippy, tests, check Windows et benchs ; s'arrête au premier échec.

## Conventions

- Noms de crates : `dsp-core`, `audio-io`, `engine`, `app` (non publiées).
- Tests unitaires dans chaque module (`#[cfg(test)]`), tests d'acceptation dans `crates/*/tests/`, benchs dans `crates/*/benches/`.
- Tests de captures PNG : `#[cfg(target_os = "macos")]`, sorties dans `crates/app/tests/snapshots/`.
- Unités explicites dans les noms (`delay_ms`, `frequency_hz`, `gain_db`, `latency_samples`).
- `f64` pour les coefficients et les accumulateurs, `f32` pour les échantillons des flux.
- Commits : « phase N : <résumé> », en local, sans ligne d'attribution.

## Process d'une phase

1. Relire `docs/SPEC.md` et « État ».
2. Coder avec `ecc:rust-patterns` ; tests avec `ecc:rust-testing` ; `dsp-algorithm-guide` pour biquads, FFT, lissage.
3. Après modification : `audio-dsp-review` (`audio-io`, `engine`), `audio-numerics-review` (`dsp-core`). Diagnostics rust-analyzer.
4. Phases 2 et 7, ou régression de bench : `audio-performance-debug` + `ecc:benchmark-optimization-loop`.
5. Phases 3 à 7 : revue visuelle (`.claude/rules/ui.md`), au moins deux tours, journal dans `docs/UI_REVIEW.md`. Début de phase 3 : `ecc:frontend-design-direction`.
6. Deux échecs de compilation consécutifs sur le même problème → agent `ecc:rust-build-resolver`.
7. Fin de phase : agent `ecc:rust-reviewer` (corriger ou justifier dans `docs/DECISIONS.md`) → skill `verify` (montrer la sortie) → mettre à jour « État » → commit « phase N : <résumé> » → `git log --oneline -5`.
8. Choix non trivial → `docs/DECISIONS.md`. Test matériel ou visuel → `docs/MANUAL_TESTS.md`.

Questions à l'utilisateur uniquement si : critère impossible, décision irréversible, information non déductible.

## Documents

- `docs/SPEC.md` : spécification complète (référence).
- `docs/DECISIONS.md` : journal des choix.
- `docs/MANUAL_TESTS.md` : checklist matériel et visuelle pour l'utilisateur.
- `docs/UI_REVIEW.md` : journal des revues visuelles.

## Autonomie

Règles complètes (enchaînement des phases, fin de phase `ecc:rust-reviewer` → `verify` → État → commit « phase N » → `git log --oneline -5`, revue visuelle kittest + `docs/UI_REVIEW.md`, skills et agents, cas où interroger l'utilisateur, interdiction d'assouplir un critère) :

@docs/AUTONOMY.md

**Lors d'une compaction, conserve la phase en cours, les tâches restantes, les derniers échecs de tests ou de verify et les décisions pas encore journalisées.**

## État

- **Construction terminée : phases 0 à 8 toutes terminées** (setup ; 0 ; 1 `dsp-core` ; 2 `engine` ; 3 interface ; 4 traces et sessions ; 5 délais et alignement ; 6 EQ virtuel ; 7 finition UX et performance ; 8 préparation Windows, CI, packaging). Rapport final : `docs/RAPPORT_FINAL.md`.
- Dernier `verify` complet (fin de phase 8, état final) : au vert, 254 tests ; benchs `engine_update` 0,62 ms, `delay_search` 1,78 ms, `plot_frame_prep` 0,053 ms, `optimal_alignment` 2,95 ms ; `ui_frame_cpu_frozen` 0,15–0,26 ms. Aucun échec en cours.
- Décisions D-001 à D-035 dans `docs/DECISIONS.md` ; revues visuelles des phases 3 à 7 dans `docs/UI_REVIEW.md`.
- Attention : la sortie de `cargo test` filtrée par rtk n'affiche que la dernière suite ; pour un total fiable : `rtk proxy cargo test --workspace`.
- `cargo-packager` 0.11.8 installé localement dans `target/tools/bin` ; `.app` : `cd crates/app && ../../target/tools/bin/cargo-packager --release --formats app`.
- Reste à faire par l'utilisateur : tests matériels et visuels de `docs/MANUAL_TESTS.md`, procédure Windows de `docs/WINDOWS.md`, premier push pour lancer la CI (aucun remote configuré).
