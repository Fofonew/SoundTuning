# Rapport final — SoundTuning (phases 0 à 8)

Date : 2026-10-26. Dépôt local uniquement (aucun remote, rien poussé).

## Ce qui est fait

| Phase | Contenu | Commit |
|---|---|---|
| 0 | Workspace Rust 2024 (4 crates), CI GitHub Actions, listing des périphériques et canaux, flux 2 entrées + 1 sortie (cpal), générateur rose/blanc, référence interne, mode fichier WAV, CLI `devices` / `record`, fenêtre vu-mètres | `8eb28f0` |
| 1 | `dsp-core` : Hann, FFT, accumulateurs Gxx/Gyy/Gxy, H1, γ², grille 1/96 d'octave (957 points), lissage en puissance, délai (FFT 131072 + parabole), biquads Cookbook, sommation, délai optimal (2 critères), moyenne de traces ; les 12 critères « Mesure » de la spec en tests | `da5a93b` |
| 2 | `engine` : thread de mesure à pas H, alignement x/y, compensation entière + rotation de phase, délai sur thread dédié, calibrage de latence, instantanés `triple_buffer`, discontinuités, benchs | `835a096` |
| 3 | Interface : système de design (jetons, Inter + JetBrains Mono, thèmes Nuit / Plein jour, Okabe–Ito), tracé custom (axe log, réticule, zoom, plages 1–4), bande de cohérence, vu-mètres, 7 états explicites, assistant en 3 étapes, palette Cmd/Ctrl+K, tests `egui_kittest` | `6e4133e` |
| 4 | Traces : capture S, sélection, renommage (double-clic, F2), couleur, visibilité, décalage, moyenne, annuler/rétablir, autosauvegarde sur thread dédié (`.bak`, session illisible mise de côté), export CSV | `0dc4d87` |
| 5 | Délai : recherche et application automatiques, suivi (T), dérive notifiée, réglage fin ±0,01 ms ; panneau Alignement (proposition adoptée, somme prédite, bande Maj + glisser) | `adec054` |
| 6 | EQ virtuel : 16 filtres × banques A/B, bypass, poignées sur le tracé (glisser, molette = Q, double-clic = ajouter), vues Prédit / EQ seul / Inversée, cible live ou trace, exports texte et JSON | `b0a5a96` |
| 7 | Finition : revue des 8 écrans dans les deux thèmes, accessibilité de tous les contrôles testée, bench d'image complète, profil CPU | `e6c4afa` |
| 8 | CI complète macOS + Windows (lint, tests, benchs compilés, contrôle croisé Windows, packaging en artefacts), packaging `cargo-packager` (.app vérifié en local avec `NSMicrophoneUsageDescription`, NSIS en CI), `docs/WINDOWS.md` | `ee5af1c` + commit de clôture |

### Critères chiffrés

- **Mesure** (tests `crates/dsp-core/tests/measurement_acceptance.rs`, `crates/engine/tests/engine_acceptance.rs`, `crates/app/src/eq_state.rs`) : délai ±0,02 ms, H1 < 0,2 dB, cohérence 0,5 ± 0,05, moyennage à 5 %, biquad ±0,01 dB, lissage ±0,01 dB, EQ plat ±0,1 dB, sommation +6,02 dB et creux > 30 dB, délai optimal 1,5 ms ± 0,02 (deux critères), moyenne ±0,01 dB, robustesse sans NaN/Inf, référence interne ±0,02 ms. Tous verts.
- **Rapidité** (criterion, Apple Silicon) : `engine_update` 0,62 ms (< 2), `delay_search` 1,76 ms (< 15), `plot_frame_prep` 0,053 ms (< 1), `optimal_alignment` 2,95 ms (< 50). ≥ 20 instantanés/s testé. Image d'interface complète (hors GPU) 0,15–0,26 ms.
- **UX** (`egui_kittest`) : Espace, D, A, S, Cmd/Ctrl+Z, Cmd/Ctrl+K, L testés ; chaque action principale est un bouton visible et un raccourci ; aucune modale pendant la mesure ; captures premier lancement, mesure Nuit, mesure Plein jour, alignement, EQ (plus traces, alignement et EQ en Plein jour).

## Décisions

35 décisions journalisées dans `docs/DECISIONS.md` (D-001 à D-035). Les principales :
- D-004 : référence interne par défaut, latence système calibrée par carte et fréquence d'échantillonnage.
- D-018 / D-019 : détails du délai et garde-fou du critère « plus plat » (une annulation totale ne peut pas gagner).
- D-023 : architecture du moteur (cœur synchrone testable, thread de mesure, thread de délai, `triple_buffer`).
- D-028 / D-031 : traces stockées sur la grille d'affichage ; autosauvegarde sur thread dédié, `.bak`, session illisible mise de côté.
- D-032 : seuil de dérive 0,5 ms, proposition d'alignement adoptée automatiquement, bande ≥ 1/3 d'octave.
- D-033 : vues d'EQ, poignées, exports.
- D-034 : profil de performance (≈ 3,5 % d'un cœur hors GPU).
- D-035 : packaging `cargo-packager`, `.app` zippé au lieu d'un `.dmg`, NSIS construit par la CI.
- Dépendances hors stack justifiées : `clap` (D-009), `chrono` (D-028), polices Inter et JetBrains Mono sous licence SIL OFL.

## Captures finales de chaque écran

Dans `docs/ui-review/phase-7/*-tour2-apres.png` (régénérées par `cargo test -p app --test ui_screenshots` dans `crates/app/tests/snapshots/`, dossier ignoré par git car réécrit à chaque passage des tests) :
- `first_launch-tour2-apres.png` : assistant, étape 1 ;
- `measurement_night-tour2-apres.png` et `measurement_daylight-tour2-apres.png` : mesure en cours ;
- `traces-tour2-apres.png` : trois traces décalées, réticule ;
- `alignment-tour2-apres.png` et `alignment_daylight-tour2-apres.png` : alignement, somme prédite, bande 60–150 Hz ;
- `eq-tour2-apres.png` et `eq_daylight-tour2-apres.png` : EQ à 3 filtres, résultat prédit.

Historique des revues visuelles (phases 3 à 7, au moins deux tours chacune) : `docs/UI_REVIEW.md`.

## Tests manuels

Contenu de `docs/MANUAL_TESTS.md` (reproduit ci-dessous dans la réponse finale) : matériel et audio (UAX 2-2, autorisation micro, générateur, calibrage, distance, rebouclage, débranchement, CLI), traces et sessions, délais et alignement, EQ virtuel, performance (démarrage < 1 s, CPU < 10 %), visuel (lisibilité à 1 m, plein soleil, daltonisme, VoiceOver). Procédure Windows : `docs/WINDOWS.md`.

## Écarts avec la spec

1. **« H brut » des traces** (§10.3) : la réponse complexe est stockée sur la grille d'affichage (957 points, lissage du moment), pas par case FFT. Toutes les fonctions (lissage, cohérence, EQ, alignement, moyenne) travaillent sur cette grille ; session ≈ 50 Ko par trace au lieu de ≈ 600 Ko (D-028).
2. **Packaging macOS** : `.app` zippé au lieu d'un `.dmg` (la création du `.dmg` pilote le Finder et échoue sans session graphique) (D-035).
3. **Installeur Windows et ASIO** non exécutés ici (pas de PC Windows, SDK Steinberg non redistribuable) : construction de l'installeur par la CI, procédures dans `docs/WINDOWS.md`.
4. **Mesures matérielles et perceptives** (démarrage < 1 s, CPU < 10 % en conditions réelles, lisibilité au soleil, carte réelle) : non automatisables, listées dans `docs/MANUAL_TESTS.md` ; l'estimation à partir des benchs (≈ 3,5 % d'un cœur hors GPU) est dans D-034.

Aucun critère d'acceptation n'a été assoupli.
