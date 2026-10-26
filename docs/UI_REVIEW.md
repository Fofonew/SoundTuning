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

### Phase 4 — tour 3 (2026-10-26, après la revue de code)
- Écran : le même (3 traces, panneau Traces) avec la nouvelle case « Sél. » (sélection séparée du renommage).
- Critiques : deuxième ligne surchargée, « × » de suppression chevauchant le champ de décalage et rogné par le bord du panneau ; puis « × » collé au nom au lieu d'être aligné à droite ; « Visible » rogné par le champ de décalage.
- Corrections : suppression déplacée sur la première ligne, alignée à droite ; décalage aligné à droite de la deuxième ligne ; retrait de la deuxième ligne réduit à 4 px.
- Captures : avant `traces-tour2-final.png`, après `docs/ui-review/phase-4/traces-tour3-apres.png`.

### Phase 5 — tour 1 (2026-10-26)
- Écran : alignement de deux traces (B arrivée 1,2 ms plus tard), plage Sub, bande 60–150 Hz, somme prédite.
- Critiques : contenu du panneau Alignement débordant à gauche (« race A » rogné) car la ligne « Δ + Inverser la polarité de B » dépassait la largeur en disposition horizontale ; consigne en 30 pt coupée sur deux lignes ; notification de dérive tronquée ; notifications masquant le bas du panneau.
- Corrections : sélecteurs A/B en lignes à libellé de largeur fixe et liste tronquée ; Δ et polarité sur deux lignes ; consigne en chiffres tabulaires 18 pt ; texte de dérive raccourci (« Délai changé : 2,02 ms (+1,20 ms) ») ; notifications ancrées à gauche du panneau latéral quand il est ouvert.
- Captures : avant `docs/ui-review/phase-5/alignment-tour1-avant.png`.

### Phase 5 — tour 2 (2026-10-26)
- Critiques : ligne Δ centrée verticalement dans tout l'espace restant (case de polarité et aide poussées hors de l'écran) ; aide « Δ > 0 … » coupée avec un « A » orphelin.
- Corrections : ligne Δ en `horizontal` ; aide sur deux lignes explicites. Vérifié : somme prédite (rose, `predicted`, trait épais) nettement distincte des traces Okabe–Ito et de la trace live ; bande choisie ombrée avec ses bornes en chiffres ; proposition (accent) et réglage à recopier (texte principal) hiérarchisés.
- Captures : après `docs/ui-review/phase-5/alignment-tour2-apres.png`.

### Phase 6 — tour 1 (2026-10-26)
- Écran : EQ sur la mesure en direct (figée), 3 filtres (cloche 1 kHz −5 dB, plateau grave 120 Hz −4 dB, coupe-bas LR 24 à 30 Hz), vue « Prédit ».
- Critiques : panneau EQ beaucoup trop large (contenu poussé hors de l'écran, seule une bande de 70 px visible) : ligne « Banque A/B + Bypass global », éditeur de filtre « type + Bypass + × » sur une ligne, libellé « Coupe-bas Linkwitz-Riley 24 dB/oct » non tronqué dans la liste du type, rangées d'export non repliables.
- Corrections : « Bypass global » sur sa propre ligne ; listes de type, de pente et de cible à largeur fixe et tronquées ; la liste du type n'affiche que la famille (la pente a sa propre liste) ; rangées en `horizontal_wrapped` ; suppression alignée à droite.
- Captures : avant `docs/ui-review/phase-6/eq-tour1-avant.png`.

### Phase 6 — tour 2 (2026-10-26)
- Critiques : carte de filtre sur quatre lignes (Q seul sur une ligne), « Trace inversée » renvoyée à la ligne dans le sélecteur de vue.
- Corrections : carte en trois lignes (type + × ; fréquence + gain ; Q + largeur en octaves + Bypass à droite) ; bouton de vue « Inversée » (le libellé complet reste celui de la courbe). Vérifié : résultat prédit (rose, épais) plat à −6 dB entre 60 Hz et 10 kHz, courbe d'EQ (accent, fine) distincte, poignées numérotées, filtre sélectionné en accent dans le tracé et dans le panneau.
- Captures : après `docs/ui-review/phase-6/eq-tour2-apres.png`.

### Phase 7 — tour 1 (2026-10-26, revue complète, deux thèmes)
- Écrans : premier lancement, mesure Nuit, mesure Plein jour, traces, alignement (Nuit + Plein jour), EQ (Nuit + Plein jour).
- Critiques : bouton « Suivi » éteint indiscernable d'un texte (aucun cadre, aucun état) ; glyphes − / + en police à chasse fixe, « − » lu comme un tiret court ; état vide de l'EQ sans titre (incohérent avec Traces et Alignement) ; aucun libellé d'accessibilité sur les champs numériques (bande, Δ, fréquence, gain, Q, décalage) ni sur les listes (traces A/B, cible, type, pente, canaux de l'assistant). Points vérifiés sans défaut : contraste Plein jour (texte #0A0C0F sur blanc, somme prédite magenta #A3007D), poignées d'EQ lisibles dans les deux thèmes, notifications hors du panneau.
- Corrections : bascule « ○ Suivi » / « ● Suivi » encadrée, accent quand active ; − / + en Inter demi-gras 20 pt ; titre « EQ virtuel » dans l'état vide ; libellés AccessKit sur tous les champs et listes (« Fréquence du filtre 2 », « Décalage — Trace 1 — 21:42 »…), test `every_control_has_an_accessible_label_in_every_tab`.
- Captures avant : `docs/ui-review/phase-7/*-tour1-avant.png`.

### Phase 7 — tour 2 (2026-10-26)
- Critiques : aucune régression ; barre supérieure équilibrée (délai 30 pt, réglage fin et suivi groupés à sa droite, actions principales à droite) ; écart proposition 1,21 ms / dérive 1,20 ms cohérent (mesure).
- Captures après : `docs/ui-review/phase-7/*-tour2-apres.png` (8 écrans).

### Vues Phase, IR, superposition, plein écran et définition des traces — tour 1 (2026-10-26, demande de l'utilisateur)
- Écrans : Phase au-dessus de Magnitude (Nuit), IR seule (Nuit), Phase au-dessus de l'IR en plein écran (Plein jour), EQ (contrôle de non-régression avec la nouvelle barre de vues).
- Critiques : sur la ligne d'une trace, la liste « Définition » (1/48) écrasait la case « Visible » (libellé disparu) ; dans l'IR, l'étiquette « Pic : 0,00 ms » chevauchait le pic (à gauche de la vue par défaut) ; l'axe des temps ne disait pas que 0 ms = délai appliqué (le pic à 0 ms pouvait passer pour une erreur alors que le délai affiché est 2,02 ms) ; boutons de la vue du dessus homonymes de ceux de la vue principale (« Phase » deux fois pour l'accessibilité).
- Corrections : ligne de trace sur trois lignes (nom ; Sél., Visible, décalage ; « Définition » + liste) ; étiquette du pic en haut à droite ; axe « ms · 0 = délai appliqué » ; boutons du dessus nommés « Phase (au-dessus) »… pour AccessKit.

### Vues — tour 2 (2026-10-26)
- Critiques : aucune régression sur EQ, alignement, traces, réglages audio (la barre de vues décale le tracé de 40 px, l'aire centrale reste > 70 %) ; phase repliée à ±180° sans trait vertical parasite ; IR normalisée au plus grand pic, enveloppe min/max par colonne ; plein écran : panneau latéral et barre inférieure masqués, délai toujours visible en haut. Défaut connu assumé : trace et mesure identiques se superposent exactement (la trace n'est visible qu'après un changement).
- Captures après : `docs/ui-review/vues/phase_stacked-tour2-apres.png`, `impulse-tour2-apres.png`, `impulse_stacked_fullscreen_daylight-tour2-apres.png`, `eq-tour2-apres.png`.

### Petites fenêtres (Windows 125–150 %) — tour 1 (2026-10-26, signalé par l'utilisateur)
- Reproduction : `egui_kittest` à 1280 × 680 avec un facteur 1,25 (Windows 1920 × 1080 à 150 %, barre des tâches déduite) et à 1024 × 640, réglages audio ouverts.
- Critiques : barre inférieure : « Réf. interne » chevauche « Commandes » ; à 1024 px, pastilles et boutons s'empilent les uns sur les autres ; barre supérieure : vu-mètres sur le nom du logiciel ; réglages audio : l'aide sous « Appliquer les réglages » est coupée par la barre inférieure.
- Corrections : densités de la barre inférieure choisies d'après les largeurs mesurées ; barre supérieure compacte ; pied des réglages audio fixé en bas, champs défilants.
- Captures avant : `docs/ui-review/fenetre-reduite/avant_1280_125.png`, `avant_1024.png`.

### Petites fenêtres — tour 2 (2026-10-26)
- Critiques : dans le pied des réglages audio, « Actualiser » passait à la ligne vers le bas et recouvrait l'aide (rangée à retour automatique dans une mise en page de bas en haut).
- Corrections : les deux boutons sont sur une ligne s'ils tiennent, sinon empilés explicitement. Vérifié à 1024 × 600 : traces, alignement, EQ, assistant, sans chevauchement ; à 1280 × 680 : raccourcis des boutons discrets masqués seulement.
- Captures après : `docs/ui-review/fenetre-reduite/apres_1280_125.png`, `apres_1024.png`, `apres_min_traces.png`, `apres_min_eq.png`.
