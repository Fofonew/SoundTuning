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
- Tests de captures PNG : `#[cfg(target_os = "macos")]`, sorties dans `crates/app/tests/snapshots/` (ignoré par git ; copies de référence dans `docs/ui-review/`).
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
- Interface restructurée comme SysTune (2026-10-29, D-051 / D-052) : panneau Measure / Tools à gauche, deux graphes avec Display, barre souris Peek / Drag / Zoom, barre d'état « Setup: », module Virtual EQ (Delay Analysis, Virtual EQ), Peak qui applique le délai (Cmd/Ctrl+Z pour revenir). Captures : `docs/ui-review/interface-systune/`.
- Retours du 2026-11-23 (D-055 à D-058) : délai total sans calibrage comme SysTune, même carte reconnue sous Windows et bandeau « horloges distinctes » masquable, curseur de niveau −60…0 dB (−6 par défaut), jauges d'entrée avec CLIP tenu 2 s, pilote affiché pour chaque périphérique Windows. **Sur un PC Windows : lire d'abord `docs/WINDOWS_HANDOFF.md`** (ASIO à diagnostiquer, ASIO Control Panel, mode exclusif ; l'icône de l'exe est faite, D-059).
- Première session sur PC Windows (2026-11-23, D-059 à D-061) : sous-système graphique en release (plus de console au lancement, `win32console` pour la sortie des commandes dans un terminal), icône et métadonnées du `.exe` (`crates/app/build.rs`, `winresource`) ; affichage plafonné à 60 fps avec option 30 fps (Tools → Frame Rate), rafraîchissement à l'arrivée des instantanés, attente adaptative du moteur (CPU 20,3 % → ≈ 8,6 % en mode fichier sur écran 165 Hz) ; vu-mètre dans les boutons de canal comme SysTune ; runtime Visual C++ intégré à l'exe (`.cargo/config.toml`, D-062). Carte de l'utilisateur : Swissonic UA-2X2 (pilote générique Windows, sans ASIO), flux WASAPI réels vérifiés. Installeur NSIS produit localement. `cargo-packager` aussi dans `target\tools\bin` sous Windows.
- Dernier `verify` complet (2026-11-23, sur Windows, après fusion des sessions Mac D-055 à D-058 et Windows D-059 à D-062) : au vert, 347 tests (les tests de captures, réservés à macOS, ne tournent pas sous Windows ; la session Mac comptait 351 tests avant la fusion). Benchs sous les budgets mais mesurés pendant qu'un autre processus occupait un cœur : `engine_update` 1,82 ms, `delay_search` 2,44 ms, `plot_frame_prep` 0,20 ms, `optimal_alignment` 6,11 ms ; `ui_frame_cpu_frozen` 0,71–1,09 ms. Sur machine calme avant le runtime statique : 0,79 / 1,06 / 0,086 / 2,66 ms. Le runtime statique coûte ≈ +15–20 % sur `engine_update` en A/B (D-062). Captures de référence (`docs/ui-review/`) à régénérer à la prochaine session macOS pour y montrer les boutons à vu-mètre. Aucun échec en cours.
- Décisions D-001 à D-062 dans `docs/DECISIONS.md` (D-038 : sortie « Automatique » et négociation de fréquence ; D-039 : ASIO, installeur Windows compilé avec ASIO par la CI ; D-040 : vues Phase / IR, superposition, plein écran ; D-041 : définition des traces modifiable après capture, exacte ; D-042 : recherche de mise à jour au lancement, hors connexion sans effet ; D-043 : releases par tag `vX.Y.Z` (`scripts/release.sh`, `docs/RELEASE.md`) ; D-044 : petites fenêtres et DirectX 12 sous Windows ; D-045 : alignement façon SysTune Delay Analysis ; D-046 : termes techniques en anglais ; D-047 : icônes Phosphor ; D-048 : zoom par sélection au glisser, déplacement au clic droit ; D-049 : séparation des vues redimensionnable ; D-050 : somme de l'alignement en 3ᵉ courbe à côté de Reference et Delay ; D-051 : interface structurée comme SysTune ; D-052 : Peak trouve et applique le délai ; D-053 : `.app` signé ad hoc ; D-054 : logo et icônes, `scripts/icons.sh` ; D-055 : délai total sans calibrage ; D-056 : même carte sous Windows, bandeau masquable ; D-057 : curseur de niveau ; D-058 : jauges d'entrée ; D-059 : sous-système graphique Windows et icône du `.exe` ; D-060 : cadence d'affichage plafonnée et rafraîchissement à l'arrivée des instantanés ; D-061 : vu-mètre dans les boutons de canal ; D-062 : runtime Visual C++ intégré, aucun prérequis Windows) ; revues visuelles des phases 3 à 7 dans `docs/UI_REVIEW.md`.
- **Release en cours (2026-11-23)** : tag `v0.2.4` poussé, mais son workflow Release a échoué (test instable, corrigé dans `57149aa`) et rien n'a été publié ; le tag est laissé tel quel, la prochaine release sera `v0.2.5`. **Bloquant** : sur `windows-latest`, `cargo check -p app --features asio` échoue en CI depuis `e484377`. Il passe en local avec libclang 18.1.1 ; libclang figé en CI (D-063) n'a pas suffi. Le journal de la CI exige une authentification : le lire (onglet Actions) avant toute nouvelle correction, puis `scripts/release.sh 0.2.5`.
- Attention : la sortie de `cargo test` filtrée par rtk n'affiche que la dernière suite ; pour un total fiable : `rtk proxy cargo test --workspace`.
- `cargo-packager` 0.11.8 installé localement dans `target/tools/bin` ; `.app` : `cargo build --release -p app && cd crates/app && ../../target/tools/bin/cargo-packager --release --formats app` (plus de `before-packaging-command`, D-039).
- Dépôt `origin` = github.com/Fofonew/SoundTuning, **public** depuis le 2026-10-26 (historique réécrit avec l'adresse Gmail de l'utilisateur ; pousser uniquement avec son accord). Releases : `docs/RELEASE.md`.
- Reste à faire par l'utilisateur : tests matériels et visuels de `docs/MANUAL_TESTS.md`, cases restantes de la procédure Windows de `docs/WINDOWS.md` (audio réel WASAPI et ASIO, installeur, console dans un vrai terminal), première release (`scripts/release.sh`). ASIO non compilé sur ce PC : LLVM (`winget install LLVM.LLVM`) absent, installation système laissée à l'utilisateur.
