# Autonomie (construction pilotée par /goal)

Règles reprises intégralement de la section « Autonomie » du prompt initial, avec les noms exacts des skills et agents installés.

## Enchaînement

- Enchaîner les phases 0 à 7, puis préparer la phase 8, **sans demander de validation entre les phases**.
- Avant chaque phase : relire `docs/SPEC.md` et la section « État » de `CLAUDE.md`.

## Fin de chaque phase (dans cet ordre)

1. Agent `ecc:rust-reviewer` sur le code de la phase : corriger ce qu'il remonte, ou noter dans `docs/DECISIONS.md` pourquoi on ne le fait pas.
2. Skill `verify` (fmt, clippy `-D warnings`, tests, `cargo check --target x86_64-pc-windows-msvc` sans `asio`, benchs criterion `cargo bench --workspace --bench '*' -- --noplot` comparés aux budgets) ; **montrer sa sortie**.
3. Mettre à jour la section « État » de `CLAUDE.md`.
4. Commit local « phase N : <résumé> » (sans ligne d'attribution).
5. Afficher `git log --oneline -5`.

## Revue visuelle (toute phase qui touche l'interface : phases 3 à 7)

1. Générer les captures PNG des écrans clés avec `egui_kittest` en mode fichier (`crates/app/tests/ui_screenshots.rs`, macOS, rendu wgpu hors écran ; sorties dans `crates/app/tests/snapshots/`).
2. Ouvrir chaque PNG et le critiquer : hiérarchie visuelle, lisibilité à 1 m, alignements, espacements, contrastes, densité, cohérence des états, fidélité au système de design (`docs/DESIGN.md`).
3. Corriger, régénérer et recommencer : **au moins deux tours par phase**.
4. Consigner dans `docs/UI_REVIEW.md` les critiques, les corrections et les chemins des captures avant et après (copies dans `docs/ui-review/phase-N/`).

## Limites

- Ne rien pousser, ne créer aucun remote, ne rien modifier hors de ce dossier.
- Tests automatisés : uniquement signaux synthétiques et mode fichier WAV. Ne jamais ouvrir de flux audio réel, ne jamais lancer l'interface dans une fenêtre bloquante. Le rendu hors écran d'`egui_kittest` est autorisé. Tout ce qui demande du matériel ou un œil humain va dans `docs/MANUAL_TESTS.md`.
- **Ne jamais assouplir un critère d'acceptation.** S'il est impossible à atteindre, s'arrêter et le dire.

## Quand interroger l'utilisateur

Uniquement dans trois cas :
1. un critère d'acceptation impossible à atteindre (le dire, ne jamais l'assouplir) ;
2. une décision irréversible ;
3. une information impossible à déduire.

Sinon : choisir l'option la plus raisonnable, la noter dans `docs/DECISIONS.md` et continuer.

## Skills et agents (noms exacts)

| Quand | Skill / agent |
|---|---|
| En écrivant du code | `ecc:rust-patterns` |
| Pour tous les tests | `ecc:rust-testing` |
| Après toute modification de `crates/audio-io` ou `crates/engine` | `audio-dsp-review` |
| Après toute modification de `crates/dsp-core` (divisions par Gxx ≈ 0, log10(0), NaN/Inf, dénormalisés) | `audio-numerics-review` |
| Biquads, FFT, lissage | `dsp-algorithm-guide` |
| Phase 2, phase 7 et à chaque régression de bench | `audio-performance-debug` + `ecc:benchmark-optimization-loop` |
| Début de la phase 3 (direction visuelle) | `ecc:frontend-design-direction` |
| À chaque revue visuelle (principes web transposés à egui : espacements, hiérarchie, états, rayons, ombres, contrastes, zones cliquables) | `ecc:make-interfaces-feel-better` |
| Fin de chaque phase, avant `verify` | agent `ecc:rust-reviewer` |
| Compilation en échec deux fois de suite sur le même problème | agent `ecc:rust-build-resolver` |

- S'appuyer sur les diagnostics de rust-analyzer après chaque modification.

## Fin de construction

À la fin de la phase 8, afficher un rapport final : ce qui est fait, les décisions prises, les captures finales de chaque écran, le contenu de `docs/MANUAL_TESTS.md` et les éventuels écarts avec la spec.

## Compaction

Lors d'une compaction, conserve la phase en cours, les tâches restantes, les derniers échecs de tests ou de verify et les décisions pas encore journalisées.
