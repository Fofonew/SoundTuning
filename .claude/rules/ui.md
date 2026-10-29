---
paths: ["crates/app/**"]
---

# Système de design, UX et revue visuelle

Début de phase 3 : `ecc:frontend-design-direction`. À chaque revue visuelle : `ecc:make-interfaces-feel-better` (principes web transposés à egui : espacements, hiérarchie, états, rayons, ombres, contrastes, zones cliquables).

## Système de design (module dédié, ex. `crates/app/src/design/`)

- **Jetons de couleurs** sémantiques (`surface`, `surface_raised`, `text_primary`, `text_muted`, `accent`, `warning`, `danger`, `success`, `grid_major`, `grid_minor`, `coherence_band`…), définis pour chaque thème. Aucune couleur littérale hors du module.
- **Espacement** : échelle 4 / 8 / 12 / 16 / 24 / 32 uniquement.
- **Rayons**, **tailles de texte** (échelle nommée : `caption`, `body`, `label`, `title`, `display`), **épaisseurs de trait** (grille, trace mémorisée, trace live, réticule) : jetons nommés.
- **Polices embarquées** : Inter (interface), JetBrains Mono à chiffres tabulaires (toutes les valeurs numériques). Licence SIL OFL notée dans `docs/DECISIONS.md`.
- **Thèmes** : « Nuit » (défaut, sombre, éblouissement minimal, pas de blanc pur) et « Plein jour » (clair, contraste élevé, lisible au soleil). Bascule touche L. Aucun écran avec le style egui par défaut.
- **Traces** : palette Okabe–Ito ; trace live plus épaisse et visuellement distincte (épaisseur + luminosité), traces mémorisées plus fines.
- **Textes** : tous dans le module de textes (ex. `crates/app/src/text.rs`), en français, aucune chaîne d'interface en dur ailleurs. Nombres au format français (virgule décimale, « 12,43 ms »).

## Disposition (structure SysTune, D-051)

- à gauche, panneau **Measure / Tools** repliable (P), comme la zone de contrôle de SysTune : sections Input (Signal Channel, Delay Offset en gros chiffres avec −/+, Peak, Auto, Reference Channel OUT/1/2…, Calibrate), Start Analysis / Stop Analysis et Freeze, Output (Play Signal, niveau du Pink Noise, Play Channel), Parameters (FFT Size, Averages, Reset), Capture Measurement, puis onglets Overlays (liste des traces : couleur, V, nom, propriétés ; Average, Save, Remove) et Levels (Signal et Reference en Peak / RMS dBFS) ; l'onglet Tools contient le Color Scheme, Check for Updates, Commands et les réglages audio ;
- zone centrale ≥ 70 % de la surface : deux graphes superposés (un seul au choix, V), chacun avec sa barre de sélection (Transfer Function : IR, Mag, Phs ; External : Virtual EQ), sa légende et son panneau Display à droite (Show Coherence, Mask γ², Smoothing ; Unit pour l'IR ; Delay Analysis / Virtual EQ pour le module) ; bande de cohérence sous le graphe Mag ;
- entre les deux graphes, la barre souris : coordonnées du pointeur et valeur de la trace la plus proche, modes Peek / Drag / Zoom, un ou deux graphes, plein écran ; elle sert aussi de séparation redimensionnable ;
- en bas, barre d'état : état de la mesure, ligne « Setup: » (source, fréquence, référence, signal joué et sorties), Commands (Cmd/Ctrl+K) et version.
- Délai courant toujours lisible à 1 m : Delay Offset en chiffres ≥ 28 pt dans la section Input.
- Termes des boutons repris de SysTune (Start Analysis, Peak, Auto, Calibrate, Play Signal, Capture Measurement, Overlays, Average, Save, Remove, Display, Show Coherence, Smoothing, Peek, Drag, Zoom, Virtual EQ, Delay Analysis) ; aides et notifications en français.

## Tracé (painter egui, pas d'`egui_plot`)

- Axe fréquence log, graduations standard (20, 50, 100, 200, 500, 1k, 2k, 5k, 10k, 20k ; mineures intermédiaires).
- Réticule au survol : fréquence, dB, valeur de la trace la plus proche.
- Zoom molette/pincement et par sélection au glisser (comme SysTune), déplacement au clic droit ou Cmd/Ctrl + glisser, double-clic ou Full View = réinitialiser ; modes Peek / Drag / Zoom dans la barre souris ; plages 1–4 (plein spectre, sub 20–200 Hz, médium, aigu).
- Ne dessine que la grille d'affichage (≈ 957 points) ; points invalides ou sous le seuil de cohérence non tracés (ruptures de courbe).
- Préparation d'une image < 1 ms ; aucun calcul DSP dans la boucle de rendu ; aucune animation sur les courbes.

## Règles d'UX

- Chaque action principale : bouton visible sur l'écran principal **et** raccourci (Espace, D, A, S, F, 1–4, L, Cmd/Ctrl+K, Cmd/Ctrl+Z, Shift+Cmd/Ctrl+Z).
- Palette de commandes (Cmd/Ctrl+K) : toutes les actions avec leurs raccourcis.
- Aucune fenêtre modale pendant la mesure : notifications non bloquantes avec action (« Délai trouvé : 12,43 ms — Appliquer »).
- États explicites avec marche à suivre : pas de signal, écrêtage, référence absente, cohérence faible, autorisation micro manquante, carte son déconnectée, latence non calibrée.
- Animations ≤ 150 ms, uniquement pour les changements d'état.
- Zones cliquables ≥ 28 px ; mise à l'échelle Cmd/Ctrl + et − ; navigation clavier ; libellés AccessKit sur tous les contrôles.
- Assistant de premier lancement en 3 étapes, une seule fois.
- Annuler/rétablir pour traces, alignement, EQ.

## Tests `egui_kittest`

- Tests d'interaction pour chaque raccourci et chaque bouton principal ; vérifier qu'aucune modale ne s'ouvre pendant la mesure.
- Captures PNG en mode fichier (aucun flux audio), `#[cfg(target_os = "macos")]` : premier lancement, mesure Nuit, mesure Plein jour, alignement, EQ.

## Boucle de revue visuelle (phases 3 à 7, au moins deux tours par phase)

1. Générer les captures PNG des écrans clés avec `egui_kittest` en mode fichier.
2. Ouvrir chaque PNG et le critiquer : hiérarchie visuelle, lisibilité à 1 m, alignements, espacements, contrastes, densité, cohérence des états, fidélité au système de design.
3. Corriger, régénérer, recommencer.
4. Consigner dans `docs/UI_REVIEW.md` : date, phase, tour, critiques, corrections, chemins des captures avant et après (copies conservées dans `docs/ui-review/phase-N/`).
