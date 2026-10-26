# SoundTuning — spécification

Outil desktop de mesure acoustique pour le calage de systèmes de sonorisation en live.
Rust, macOS (développement) et Windows. Nom de travail initial : « CalageLive », nom définitif : **SoundTuning**.

Ce document est la référence. Il est relu avant chaque phase. Toute adaptation est tracée dans `docs/DECISIONS.md`.

---

## 1. Priorités

1. **Un workflow inspiré de SysTune** : mesure continue en boucle ouverte par double FFT (référence + micro), avec la musique ou le bruit rose comme signal de test, pendant le concert. On mesure, on cale le délai, on lit, on capture, on aligne, on égalise, on compare, sans jamais relancer de mesure ni ouvrir de fenêtre bloquante. Le workflow attendu est celui de la section 4, pas un souvenir de SysTune.
2. **Rapide** : courbes fluides et réactives, démarrage immédiat, chaque action courante en un appui de touche ou un clic.
3. **Moderne et agréable, avec une très bonne UX** : interface soignée, lisible de loin en régie comme en plein soleil, qui ne ressemble pas à un outil de debug.

**Ordre d'arbitrage** : justesse des mesures > réactivité > qualité de l'interface > nombre de fonctionnalités.

---

## 2. Réponses de l'utilisateur (étape 2)

| Sujet | Réponse | Conséquence |
|---|---|---|
| Carte son | UAX 2-2 (carte USB, 2 entrées / 2 sorties d'après son nom ; le nombre réel de canaux est lu par `cpal`) | Rien n'est codé en dur : canaux listés dynamiquement. |
| Canaux | Micro sur l'entrée 1. Bruit rose sur la sortie gauche **ou** droite. **La référence est prise dans le logiciel** (signal du générateur). | Le mode « référence interne » devient un mode de premier plan, configuration par défaut (voir §2.1). |
| Fréquence par défaut | 48 kHz | Valeurs par défaut de la spec inchangées (N = 32k, H = 2048). |
| CI | GitHub Actions | `.github/workflows/ci.yml`, runners `macos-latest` et `windows-latest`. |
| Nom | SoundTuning | Binaire `soundtuning`, bundle `SoundTuning.app`, titre de fenêtre « SoundTuning ». |

### 2.1 Modes de référence

- **Référence interne (défaut)** : x = échantillons émis par le générateur sur la sortie choisie (G ou D), y = entrée micro choisie (défaut : entrée 1). Le générateur publie ses échantillons vers le moteur (via `rtrb`) avec leur index de trame ; le moteur aligne x et y par index de trame et horodatage `cpal`.
  - Même périphérique en entrée et en sortie (cas UAX 2-2) : même horloge, mesure fiable. Le délai mesuré inclut la latence aller-retour de la carte (tampons + convertisseurs).
  - **Calibrage de latence** : en un clic, avec un câble de la sortie générateur vers une entrée libre (ou vers l'entrée micro, micro débranché), le délai mesuré est mémorisé comme latence système pour le triplet (périphérique, fréquence, taille de tampon). Le délai acoustique affiché = délai mesuré − latence système. Sans calibrage, l'UI l'indique clairement (« latence non calibrée ») et la distance affichée est marquée approximative.
  - Entrée et sortie sur des périphériques différents (ex. micro intégré du Mac) : horloges distinctes, dérive possible → mode marqué « moins fiable » dans l'UI, comme le prévoyait la spec initiale.
- **Référence rebouclée** : x = une entrée de la carte (sortie console rebouclée), y = l'entrée micro. Indispensable pour mesurer sur la musique pendant le concert. Toutes les règles de la spec s'appliquent sans changement.

L'assistant de premier lancement propose la référence interne par défaut et permet de choisir la référence rebouclée.

---

## 3. Règles non négociables

- Rust stable, édition 2024, workspace Cargo. Dépendances ajoutées avec `cargo add` (versions actuelles, jamais de numéro inventé). Toute dépendance hors de la stack (§9) est justifiée dans `docs/DECISIONS.md`.
- Code, identifiants et logs en anglais. Pas de commentaires d'explication dans le code : des noms explicites à la place. Interface en français, tous les textes centralisés dans un module (traduction future).
- Aucun `unwrap`/`expect` hors tests. `thiserror` dans les bibliothèques, `anyhow` dans l'application, `tracing` pour les logs. Pas de `unsafe`.
- Temps réel : aucune allocation, aucun verrou, aucune I/O dans le callback audio.

---

## 4. Workflow de calage (référence fonctionnelle)

Tout se passe sur un seul écran principal ; la mesure ne s'interrompt jamais pendant ces étapes.

1. **Ouvrir** : l'application reprend la dernière configuration (périphérique, canaux, mode de référence, réglages, session). Espace lance la mesure continue.
2. **Mesurer en continu** : magnitude, cohérence et délai se mettent à jour en temps réel. L'opérateur déplace le micro et lit, sans relancer de mesure.
3. **Caler le délai d'abord** : au lancement de la mesure, le délai est cherché automatiquement puis appliqué. Si le délai mesuré change nettement (micro déplacé), une notification non bloquante propose de le recaler, en un appui (D puis A).
4. **Lire et capturer** : S fige l'état courant en trace nommée automatiquement (« Trace 3 — 21:42 »), renommable ensuite d'un double-clic.
5. **Aligner** : l'opérateur choisit deux traces (tête et sub, façade et rappel). L'outil propose automatiquement délai et polarité optimaux, pour un niveau sommé maximal ou pour la réponse la plus plate dans une bande choisie à la souris sur le tracé. La sommation prédite s'affiche immédiatement, avec la valeur à recopier dans le processeur.
6. **Égaliser** : EQ virtuel sur la trace live ou sur une trace mémorisée, résultat prédit en direct, puis export de la liste des filtres.
7. **Comparer** : superposer avant/après, moyenner plusieurs traces pour un calage multi-positions.

---

## 5. Rapidité

- Cadence de mise à jour indépendante de la taille de FFT : au moins 20 rafraîchissements des courbes par seconde à 48 kHz par défaut.
- Rendu à 60 images/s ou à la fréquence de l'écran, découplé du moteur. Les courbes ne saccadent jamais au zoom, au survol ou en édition d'EQ.
- Démarrage jusqu'à la première image utilisable < 1 s en release sur Apple Silicon (contrôle manuel).
- < 10 % d'un cœur en mesure continue avec les réglages par défaut (contrôle manuel).
- Chaque action courante (démarrer, trouver ou appliquer le délai, capturer une trace, ajouter un filtre, changer de plage) en un appui de touche ou un clic.
- Sauvegarde automatique de la session au plus 10 s après une modification, reprise après plantage.

---

## 6. Interface et UX

- **Système de design** dans un module dédié de `crates/app` : jetons de couleurs, échelle d'espacement 4/8/12/16/24/32, rayons, tailles de texte, épaisseurs de trait. Aucun écran ne garde l'apparence egui par défaut.
- **Polices embarquées** : Inter (interface), JetBrains Mono à chiffres tabulaires (valeurs numériques). Licence SIL OFL, récupérées depuis leurs dépôts officiels, licence notée dans `docs/DECISIONS.md`.
- **Deux thèmes**, touche L : « Nuit » (sombre, éblouissement minimal, défaut) et « Plein jour » (clair, contraste élevé, extérieur au soleil).
- **Couleurs des traces** : palette Okabe–Ito (daltoniens). Trace live plus épaisse et nettement distinguée des traces mémorisées.
- **Disposition** :
  - barre supérieure fine : source et fréquence d'échantillonnage, mode de référence, état de la mesure, vu-mètres compacts référence et mesure, délai courant en gros chiffres ;
  - zone centrale ≥ 70 % de la surface : tracé magnitude et, dessous, bande de cohérence ;
  - panneau latéral repliable à onglets : Traces, Alignement, EQ ;
  - barre inférieure : réglages (FFT, moyennage, lissage, seuil de cohérence) en pastilles modifiables d'un clic ou à la molette.
- **Tracé** :
  - painter egui, sans `egui_plot` ;
  - axe fréquence logarithmique, graduations standard ;
  - réticule au survol : fréquence, dB, valeur de la trace la plus proche ;
  - zoom molette ou pincement, déplacement au glisser, double-clic pour réinitialiser ;
  - plages touches 1 à 4 : plein spectre, sub 20–200 Hz, médium, aigu.
- **Palette de commandes** (Cmd/Ctrl+K) listant toutes les actions et leurs raccourcis.
- **Annuler / rétablir** (Cmd/Ctrl+Z, Shift+Cmd/Ctrl+Z) pour traces, alignement et EQ.
- **Aucune fenêtre modale pendant la mesure** : notifications non bloquantes avec action quand c'est utile (« Délai trouvé : 12,43 ms — Appliquer »).
- **Assistant de premier lancement en 3 étapes** : périphérique ; mode de référence et canaux (micro, référence ou sortie générateur) avec vu-mètres en direct ; test et démarrage. Ne réapparaît plus ensuite.
- **États explicites**, chacun avec la marche à suivre : pas de signal, écrêtage, référence absente, cohérence faible, autorisation micro manquante, carte son déconnectée, latence non calibrée (référence interne).
- **Animations** ≤ 150 ms, réservées aux changements d'état, jamais sur les courbes.
- **Taille et lisibilité** : zones cliquables ≥ 28 px, chiffres du délai ≥ 28 pt, mise à l'échelle Cmd/Ctrl + et −, navigation clavier, libellés AccessKit sur tous les contrôles.

---

## 7. Architecture

- `crates/dsp-core` : maths pures, aucune I/O, testées à fond.
- `crates/audio-io` : périphériques, mapping des canaux, flux, générateur (avec publication des échantillons émis pour la référence interne), détection de connexion et déconnexion.
- `crates/engine` : threads, alignement référence/mesure, fenêtre glissante, moyennage, délai, instantanés.
- `crates/app` : interface egui, système de design, état de l'application, annuler/rétablir. Binaire `soundtuning`.
- **Mode « entrée fichier »** : un WAV 2 canaux (référence, mesure) remplace la carte son, pour tester sans matériel, en CI et pour les captures d'écran.

---

## 8. Définitions DSP (à respecter exactement)

- x = référence (référence rebouclée : sortie console sur une entrée ; référence interne : signal émis par le générateur), y = micro de mesure. En référence rebouclée, x et y viennent du même périphérique et du même flux d'entrée (même horloge).
- Fenêtre glissante de N points (8k / 16k / 32k / 64k, défaut 32k à 48 kHz), fenêtre de Hann, avancée par pas de H échantillons indépendamment de N (défaut H = 2048, ≈ 23 mises à jour/s à 48 kHz).
- Gxx = ⟨X*·X⟩, Gyy = ⟨Y*·Y⟩, Gxy = ⟨X*·Y⟩, moyennés en complexe.
- Moyennage exponentiel de constante de temps τ (0,5 / 1 / 2 / 4 s) ou cumul infini, avec reset. α = 1 − exp(−H / (τ·fs)), pour que τ soit une durée réelle quels que soient H et N.
- H1 = Gxy / Gxx. Magnitude dB = 20·log10|H1|.
- Cohérence γ² = |Gxy|² / (Gxx·Gyy). Masquage des points où γ² < seuil (défaut 0,5, réglable).
- Grille d'affichage logarithmique au 1/96 d'octave de 20 Hz à 20 kHz (≈ 957 points), calculée une fois. Lissage, cohérence et EQ sont évalués sur cette grille ; le tracé ne dessine que cette grille.
- Lissage par fraction d'octave (aucun, 1/48, 1/24, 1/12, 1/6, 1/3) sur |H|² (domaine puissance), puis conversion en dB.
- Délai : pic de |IFFT(Gxy)| sur une FFT longue dédiée (défaut 131072 points à 48 kHz, ≈ 2,7 s), calculée à basse cadence (≤ 2 fois/s), interpolation parabolique pour la précision sub-échantillon. Positif = micro en retard sur la référence. Confiance = rapport pic/bruit en dB.
- Référence interne : délai acoustique = délai mesuré − latence système calibrée (§2.1).
- Compensation : ligne à retard d'un nombre entier d'échantillons sur la référence ; réglage fin par pas de 0,01 ms en rotation de phase e^{+jωτ} sur H.
- Distance = délai × c, c = 331,3·√(1 + T/273,15) m/s, T en °C réglable (défaut 20 °C).
- Sommation de deux traces : |H_A + p·H_B·e^{−jωΔ}|, Δ signé (positif = retarder B, négatif = retarder A), polarité p = ±1.
- Délai optimal : recherche sur Δ ∈ [−50 ; +50] ms, grossière puis fine au pas de 0,01 ms, pour p = +1 et p = −1, dans une bande [f1 ; f2] (défaut 60–150 Hz). Deux critères : niveau sommé moyen maximal, ou écart-type minimal du dB sommé.
- Moyenne de traces : moyenne en puissance de |H|², affichée en dB.
- Filtres d'EQ : biquads du « Audio EQ Cookbook » (R. Bristow-Johnson), évalués en H(e^{jω}) sur la grille d'affichage, à la fréquence d'échantillonnage de la mesure. Afficher Q et largeur de bande en octaves.

---

## 9. Stack

- Audio : `cpal` (CoreAudio sur macOS ; WASAPI, et ASIO derrière une feature `asio`, sur Windows).
- FFT : `realfft` / `rustfft`, `num-complex`.
- Échanges temps réel : `rtrb` entre callbacks et moteur ; `arc-swap` ou `triple_buffer` pour publier les instantanés vers l'UI.
- UI : `eframe` / `egui`, tracé au painter. En dépendance de développement : `egui_kittest` (features `snapshot` et `wgpu`) pour les tests d'interaction et les captures PNG.
- Fichiers : `hound` (WAV), `serde` + `serde_json` (sessions).
- Qualité : tests unitaires, `proptest` si utile, `criterion` pour les benchs.
- Erreurs et logs : `thiserror`, `anyhow`, `tracing`.

---

## 10. Fonctionnalités V1

1. **Audio**
   - Choix du périphérique, de la fréquence d'échantillonnage, de la taille de tampon, du mode de référence, du canal référence (rebouclée) ou de la sortie générateur (interne), du canal mesure ; préréglages de configuration rappelables.
   - Vu-mètres référence et mesure (dBFS, crête, témoin d'écrêtage).
   - Générateur bruit rose / bruit blanc sur une sortie choisie (G, D, ou les deux), niveau réglable.
   - Référence interne (§2.1) avec calibrage de latence en un clic ; marquée moins fiable si entrée et sortie sont sur des périphériques différents.
   - Entrée entièrement nulle : alerte qui évoque les autorisations micro de macOS. Carte son déconnectée : alerte non bloquante et reprise en un clic.
2. **Vue magnitude** (vue principale) : voir §6. Démarrer / arrêter / figer ; réglages lissage, taille FFT, moyennage, seuil de cohérence.
3. **Traces mémorisées**
   - Capture en un appui avec nom automatique ; couleur, visibilité, offset dB, renommage.
   - Chaque trace conserve H brut, γ², le délai mesuré et la compensation appliquée (et la latence système en référence interne), pour reconstruire son temps d'arrivée absolu.
   - Moyenne de traces sélectionnées.
   - Sessions JSON avec sauvegarde automatique et reprise, export CSV.
4. **Délais et alignement**
   - Recherche automatique au lancement, « Trouver le délai » (D), « Appliquer » (A), suivi activable (mise à jour seulement si la confiance dépasse un seuil), ajustement fin ±.
   - Affichage en gros chiffres : ms, mètres, échantillons, confiance.
   - Alignement de deux traces : délai et polarité optimaux proposés selon le critère choisi, bande choisie à la souris, sommation prédite affichée, Δ et polarité ajustables à la main, source à retarder clairement indiquée.
5. **EQ virtuel**
   - Banque de 16 filtres maximum : cloche, plateau grave, plateau aigu, coupe-bas et coupe-haut (Butterworth 12/24 dB/oct, Linkwitz-Riley 24/48 dB/oct), notch.
   - Bypass par filtre et global, banques A/B.
   - Édition à la souris sur le tracé (glisser = fréquence/gain, molette = Q, double-clic = ajouter) et saisie numérique.
   - Affichages : courbe d'EQ seule, trace + EQ (produit complexe = résultat prédit), trace inversée.
   - Purement virtuel : aucun traitement appliqué au signal audio.
   - Export de la liste des filtres (type, fréquence, gain, Q) en texte et JSON.
6. **Raccourcis** : Espace = démarrer/arrêter, D = trouver le délai, A = appliquer, S = capturer, F = figer, 1 à 4 = plages, L = thème, Cmd/Ctrl+K = palette, Cmd/Ctrl+Z = annuler, Shift+Cmd/Ctrl+Z = rétablir.

**Hors périmètre V1** : vue phase, réponse impulsionnelle, mode sweep, RT60, plus de 2 entrées, fonctions réseau.

---

## 11. Critères d'acceptation chiffrés (tests automatisés)

### Mesure
- **Délai** : bruit rose, y = x retardé de 3,2 ms + bruit décorrélé à 20 dB de RSB → délai trouvé à ±0,02 ms.
- **H1** : y = x filtré par une cloche 1 kHz / +6 dB / Q 2 → erreur < 0,2 dB de 50 Hz à 18 kHz là où γ² > 0,95.
- **Cohérence** : bruit décorrélé ajouté sur y à 0 dB de RSB → γ² = 0,5 ± 0,05.
- **Moyennage** : à H = 2048 et H = 4096 avec le même τ, constante de temps mesurée sur un échelon identique à 5 % près.
- **Biquad** : gain au centre d'une cloche = gain demandé ± 0,01 dB.
- **Lissage** : une réponse plate reste plate à ± 0,01 dB.
- **EQ** : trace contenant une cloche +6 dB + EQ virtuel cloche −6 dB aux mêmes paramètres → plat à ± 0,1 dB.
- **Sommation** : deux traces identiques, Δ = 0 → +6,02 dB ; Δ = demi-période à f → creux de plus de 30 dB à f.
- **Délai optimal** : deux traces identiques dont B est retardée de 1,5 ms → l'outil propose de retarder A de 1,5 ms ± 0,02 ms, polarité normale, pour les deux critères, dans la bande 60–150 Hz.
- **Moyenne de traces** : la moyenne de deux traces identiques est identique à chacune à ± 0,01 dB.
- **Robustesse** : bandes silencieuses (Gxx = 0) → aucun NaN ni Inf en sortie, points marqués invalides.
- **Référence interne** (ajout §2.1) : générateur simulé, latence système simulée L = 7,3 ms et délai acoustique d = 3,2 ms, après calibrage de L → délai acoustique affiché = d ± 0,02 ms.

### Rapidité (benchs criterion, Apple Silicon)
- Mise à jour moteur (FFT 32k sur deux canaux, accumulateurs, cohérence, lissage sur la grille) < 2 ms.
- Recherche de délai (FFT 131072) < 15 ms.
- Préparation d'une image de tracé à partir d'un instantané < 1 ms.
- Recherche du délai optimal entre deux traces < 50 ms.

### UX (`egui_kittest`)
- Espace démarre et arrête, D lance la recherche de délai, A applique, S capture une trace, Cmd/Ctrl+Z annule la capture, Cmd/Ctrl+K ouvre la palette, L change de thème.
- Chaque action principale existe à la fois comme bouton visible sur l'écran principal et comme raccourci.
- Aucune fenêtre modale ne s'ouvre pendant que la mesure tourne.
- Captures PNG en mode fichier : premier lancement, mesure en cours (Nuit), mesure en cours (Plein jour), alignement, EQ. Tests de captures sur macOS uniquement.

---

## 12. Phases

0. Workspace ; CI GitHub Actions qui compile macOS et Windows ; listing des périphériques et canaux ; flux 2 entrées + 1 sortie ; vu-mètres dans une fenêtre egui minimale ; sous-commande CLI d'enregistrement de 10 s en WAV ; mode entrée fichier.
1. `dsp-core` : fenêtres, FFT, accumulateurs, H1, γ², grille d'affichage, recherche de délai, lissage, biquads, sommation, délai optimal, moyenne de traces, avec tous les critères « Mesure ».
2. `engine` : threads, fenêtre glissante à pas H, alignement référence interne / rebouclée, ligne à retard de la référence, recherche de délai à basse cadence, latence système calibrée, instantanés vers l'UI, mode fichier, benchs « Rapidité ».
3. Fondations de l'interface : direction visuelle, système de design (jetons, polices, deux thèmes), disposition, tracé magnitude custom (axe log, réticule, zoom, plages), bande de cohérence, vu-mètres, états explicites, assistant de premier lancement, palette de commandes, mise en place d'`egui_kittest` et première revue visuelle.
4. Traces et sessions : capture, liste, couleurs, renommage, moyenne de traces, annuler/rétablir, sauvegarde automatique et reprise, export CSV.
5. Délais et alignement : recherche automatique au lancement, suivi, application, ajustement fin, calibrage de latence, notifications, panneau d'alignement avec délai optimal et sommation prédite.
6. EQ virtuel : banque de filtres, édition souris et clavier, résultat prédit, trace inversée, banques A/B, export.
7. Finition UX et performance : revue complète des écrans dans les deux thèmes, cohérence du système de design, états vides et messages, accessibilité, profilage du rendu et du moteur, passage des critères UX et « Rapidité », mise à jour de `docs/MANUAL_TESTS.md`.
8. Préparation Windows (sans machine Windows) : CI macOS + Windows complète, feature `asio` documentée (SDK Steinberg fourni par l'utilisateur), `docs/WINDOWS.md` avec la procédure de test sur un vrai PC, packaging (.app macOS avec `NSMicrophoneUsageDescription`, .exe Windows) sans signature, outil de packaging choisi et noté dans `docs/DECISIONS.md`.

À la fin de la phase 8 : rapport final (ce qui est fait, décisions, captures finales de chaque écran, contenu de `docs/MANUAL_TESTS.md`, écarts avec la spec).

---

## 13. Autonomie (pendant /goal)

- Enchaîner les phases 0 à 7, puis préparer la phase 8, sans validation entre les phases.
- Avant chaque phase : relire ce document et la section « État » de `CLAUDE.md`.
- Fin de chaque phase : agent `ecc:rust-reviewer` (corriger ou justifier dans `docs/DECISIONS.md`), puis skill `verify` avec sortie affichée, mise à jour de « État » dans `CLAUDE.md`, commit local « phase N : <résumé> », puis `git log --oneline -5`.
- Revue visuelle (phases 3 à 7) : captures PNG `egui_kittest` en mode fichier → ouvrir et critiquer chaque PNG (hiérarchie, lisibilité à 1 m, alignements, espacements, contrastes, densité, cohérence des états, fidélité au système de design) → corriger, régénérer, au moins deux tours par phase → consigner dans `docs/UI_REVIEW.md` (critiques, corrections, chemins avant/après).
- Ne rien pousser, ne créer aucun remote, ne rien modifier hors de ce dossier.
- Tests automatisés : signaux synthétiques et mode fichier WAV uniquement. Jamais de flux audio réel, jamais d'interface dans une fenêtre bloquante. Rendu hors écran `egui_kittest` autorisé. Ce qui demande du matériel ou un œil humain va dans `docs/MANUAL_TESTS.md`.
- S'arrêter pour questionner uniquement si : critère d'acceptation impossible (le dire, ne jamais l'assouplir), décision irréversible, information impossible à déduire. Sinon, option la plus raisonnable, notée dans `docs/DECISIONS.md`.
- Skills et agents :
  - `ecc:rust-patterns` en écrivant du code ; `ecc:rust-testing` pour tous les tests ;
  - `audio-dsp-review` après toute modification de `crates/audio-io` ou `crates/engine` ;
  - `audio-numerics-review` après toute modification de `crates/dsp-core` (divisions par Gxx ≈ 0, log10(0), NaN/Inf, dénormalisés) ;
  - `dsp-algorithm-guide` pour biquads, FFT et lissage ;
  - `audio-performance-debug` et `ecc:benchmark-optimization-loop` en phase 2, en phase 7 et à chaque régression de bench ;
  - `ecc:frontend-design-direction` au début de la phase 3 ; `ecc:make-interfaces-feel-better` à chaque revue visuelle (principes web transposés à egui) ;
  - agent `ecc:rust-reviewer` en fin de phase ; agent `ecc:rust-build-resolver` après deux échecs de compilation consécutifs sur le même problème.
- Diagnostics rust-analyzer après chaque modification.
