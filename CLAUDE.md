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

- **Phase en cours : 4 (traces et sessions)** — fonctionnelle, pas encore committée comme « phase 4 ».
- Fait dans la phase 4 : `session.rs` (Trace, Session, UndoHistory, moyenne, CSV, SessionStore), `ui/traces_panel.rs` (lignes à deux niveaux : couleur, nom, renommage double-clic, visible, décalage, suppression ; moyenne, export CSV), câblage `app.rs` (S capture, Cmd+Z / Maj+Cmd+Z, autosauvegarde `session.json` 2 s, reprise au lancement, export `~/Documents/SoundTuning`, traces superposées au tracé avec décalage), notifications regroupées par action ; tests : 17 interactions + 4 captures (dont `traces`) ; revue visuelle 2 tours consignée ; `cargo test -p app` : 85 réussis ; dernier `verify` complet (phase 3) au vert, aucun échec en cours.
- **Reste pour clore la phase 4** — suites de la revue `ecc:rust-reviewer` (verdict BLOQUANT, non corrigées) :
  1. HIGH : sérialisation + écriture de `session.json` sur le thread UI → thread d'écriture dédié (canal borné, coalescence, JSON compact, `sync_all` avant `rename`) ; synchrone seulement dans `on_exit`.
  2. HIGH : `session.json` illisible → le renommer en `session.corrupt-<horodatage>.json`, notifier, garder un `.bak` de la dernière sauvegarde valide.
  3. HIGH : échec d'autosauvegarde jamais retenté → ne remettre `session_dirty_since` à `None` qu'en cas de succès, notifier une fois.
  4. HIGH : `Session::to_csv` en O(n²) (`displayed_db()` par point) → calculer une fois par trace.
  5. MEDIUM : double-clic de renommage qui bascule la sélection (créer une case de sélection dédiée) ; Échap doit annuler le renommage ; F2 pour renommer ; libellés AccessKit incluant le nom de la trace.
  6. MEDIUM : notification « Annuler » liée à son édition (jeton = profondeur d'historique) ; dédoublonnage par groupe plutôt que par action.
  7. MEDIUM : historique mémoire (traces en `Arc<Trace>`), sélection/visibilité/couleur hors de l'historique ; fusion des glissés de décalage terminée au `drag_stopped`.
  8. MEDIUM : capture refusée à 64 traces / moyenne impossible → erreurs typées et textes dédiés ; métadonnées divergentes dans la moyenne.
  9. CSV : virgule décimale (locale française) ; nom de fichier unique même dans la même seconde ; tests manquants (suppression, renommage, moyenne, décalage, session corrompue, échec d'écriture, redo vidé après édition).
  Puis : `verify`, mise à jour État, commit « phase 4 : … », `git log --oneline -5`.
- Déjà amorcé pour les phases suivantes (compilé, non câblé) : `ui/alignment_panel.rs` (phase 5), `ui/eq_panel.rs` (phase 6), textes associés dans `text.rs`, `docs/WINDOWS.md` et feature `asio` (phase 8).
- Phases terminées : setup ; 0 ; 1 (`dsp-core`) ; 2 (`engine`) ; 3 (interface).
- **Prochaine action** : corriger les points HIGH 1 à 4 ci-dessus avec un test chacun.
- Suite : 5 (délai : suivi, notification de dérive, ajustement fin, calibrage ; panneau Alignement + bande à la souris + somme prédite ; capture `alignment`), 6 (EQ : câblage panneau, édition sur le tracé, résultat prédit, trace inversée, exports ; capture `eq`), 7 (finition, profilage, critères UX/Rapidité, `MANUAL_TESTS.md`), 8 (CI complète, packaging `cargo-packager`, rapport final).
