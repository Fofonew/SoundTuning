# Journal des revues visuelles

Une entrée par tour de revue (au moins deux tours par phase, phases 3 à 7).
Captures conservées dans `docs/ui-review/phase-N/` (`<écran>-tourK-avant.png`, `<écran>-tourK-apres.png`).

Critères : hiérarchie visuelle, lisibilité à 1 m, alignements, espacements, contrastes, densité, cohérence des états, fidélité au système de design.

## Modèle

### Phase N — tour K (AAAA-MM-JJ)
- Écrans : …
- Critiques : …
- Corrections : …
- Captures avant : `docs/ui-review/phase-N/…`
- Captures après : `docs/ui-review/phase-N/…`

---

### Phase 3 — tour 1 (2026-10-26)
- Écrans : premier lancement (assistant), mesure Nuit, mesure Plein jour (1440×900, mode fichier, affichage figé, réticule actif).
- Critiques :
  - barre supérieure : vu-mètres « Réf. »/« Mic » empilés l'un sur l'autre et recouverts par l'afficheur de délai ; nom du fichier source coupé sous les vu-mètres ;
  - axe dB : l'unité « dB » chevauche « +18 » ; l'étiquette « 20k » est coupée au bord droit ;
  - bande de cohérence : aplat bleu saturé qui écrase le tracé, étiquette « Cohérence » illisible sur le bleu ;
  - assistant : carte collée en haut de la fenêtre, grand vide en dessous, hiérarchie faible.
- Corrections (`ecc:make-interfaces-feel-better`, principes transposés à egui) :
  - barre supérieure réécrite en blocs de largeur fixe (groupe gauche tronqué, vu-mètres 2 × 120 px, afficheur de délai 232 px, boutons) ;
  - unité « dB » au-dessus de l'axe, marge haute de 24 px ; étiquettes de fréquence alignées à gauche/droite près des bords ;
  - bande de cohérence : remplissage à 38 % d'opacité + contour de la courbe γ², étiquette sur pastille `surface` ;
  - assistant centré verticalement (hauteur max 520 px).
- Captures avant : `docs/ui-review/phase-3/*-tour1-avant.png` ; après : `docs/ui-review/phase-3/*-tour1-apres.png`.

### Phase 3 — tour 2 (2026-10-26)
- Écrans : les trois mêmes.
- Critiques :
  - surface : la zone centrale (tracé + cohérence) ne couvre que ≈ 66 % de la fenêtre avec le panneau ouvert (spec : ≥ 70 %) ;
  - afficheur de délai à 34 pt : la ligne « m · éch. » déborde du bas de la barre supérieure ;
  - Plein jour : bon contraste du tracé (noir sur blanc), halo discret ; vu-mètre de référence en orange (crête > −6 dBFS) cohérent.
- Corrections : barre supérieure 68 px, panneau latéral 272 px → zone centrale 70,7 % à 1440×900 et 72 % à 1920×1080 (testé par `central_zone_covers_at_least_seventy_percent_with_the_panel_open`) ; délai en 30 pt (≥ 28 pt exigés).
- Captures après : `docs/ui-review/phase-3/*-tour2-apres.png` puis `*-tour2-final.png`.

### Phase 4 — tour 1 (2026-10-26)
- Écran : mesure figée avec 3 traces mémorisées (décalages +3 / −4 / 0 dB), réticule sur la trace la plus proche, panneau Traces.
- Critiques : noms de trace tronqués en « Trace… » (ligne unique trop chargée : couleur, case, nom, décalage, suppression) ; trois notifications « Trace capturée » empilées sur le tracé, texte passant sous le bouton « Annuler ».
- Corrections : ligne de trace sur deux niveaux (couleur + nom en entier ; puis « Visible », décalage, suppression) ; notifications regroupées par action (une seule « capture » visible à la fois), largeur 420 px, texte tronqué avant le bouton.
- Captures : `docs/ui-review/phase-4/traces-tour1-avant.png` → `traces-tour1-apres.png`.

### Phase 4 — tour 2 (2026-10-26)
- Critique : noms de trace centrés dans leur zone (lecture en colonne difficile).
- Correction : noms alignés à gauche ; vérifié : la trace live (blanche, halo) reste distincte des traces Okabe–Ito ; le réticule annonce la trace la plus proche avec son décalage.
- Captures : `traces-tour2-apres.png` (= `traces-tour2-final.png`).
