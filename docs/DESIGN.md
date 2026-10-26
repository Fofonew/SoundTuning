# Direction visuelle — SoundTuning

Fixée au début de la phase 3 (`ecc:frontend-design-direction`). Référence des revues visuelles (`docs/UI_REVIEW.md`).

## Contrat

- **Rôle** : lire en continu une fonction de transfert (magnitude + cohérence) et un délai pendant un calage, puis capturer, aligner, égaliser, comparer.
- **Utilisateur** : ingénieur du son en régie façade, souvent debout, regard intermittent à ~1 m, mains occupées (console, processeur). En salle : régie sombre. En plein air : soleil direct.
- **Ce qu'il lit d'abord** : (1) la courbe live et sa cohérence, (2) le délai courant, (3) l'état de la mesure (live / figé / problème).
- **Action principale** : Espace (démarrer / arrêter), puis D, A, S.
- **Viewports cibles** : 1440×900 (portable 14–16"), 1920×1080 (écran de régie). Minimum utilisable : 1280×720.

## Ton

**Instrument de mesure, pas outil de debug.** Interface calme et dense, au chrome effacé, où les données ressortent : l'esprit d'un analyseur matériel ou d'un afficheur broadcast. Pas de dégradés, pas de cartes dans des cartes, pas de texte qui décrit l'interface.

**Signature** :
- l'**afficheur de délai** en chiffres tabulaires de grande taille (≥ 28 pt), avec sa confiance sous forme de jauge segmentée, comme un afficheur à LED d'appareil de mesure ;
- la **trace live**, plus épaisse et d'une teinte neutre très claire (Nuit) ou très sombre (Plein jour), entourée d'un halo léger. Les traces mémorisées utilisent la palette Okabe–Ito : la trace live ne se confond jamais avec elles ;
- la **bande de cohérence**, une bande sous le tracé dont l'opacité suit γ² et qui devient hachurée sous le seuil.

## Jetons

### Couleurs

| Jeton | Nuit | Plein jour | Usage |
|---|---|---|---|
| `background` | `#0B0E12` | `#FFFFFF` | fond de fenêtre |
| `surface` | `#12161C` | `#F3F5F8` | barres, panneau |
| `surface_raised` | `#1A2028` | `#E7EBF0` | pastilles, boutons |
| `surface_hover` | `#232B35` | `#DCE1E8` | survol |
| `border` | `#2A333E` | `#9AA3AE` | séparateurs, contours |
| `text_primary` | `#E4E8EE` | `#0A0C0F` | texte principal (jamais de blanc pur la nuit) |
| `text_muted` | `#8E98A5` | `#3E4752` | légendes, unités |
| `accent` | `#56B4E9` | `#0060A0` | focus, sélection, action principale |
| `warning` | `#E69F00` | `#9A5F00` | cohérence faible, latence non calibrée |
| `danger` | `#D55E00` | `#B23A00` | écrêtage, carte déconnectée |
| `success` | `#009E73` | `#00704F` | mesure en cours, calage OK |
| `plot_background` | `#090B0E` | `#FFFFFF` | zone de tracé |
| `grid_major` | `#2B323C` | `#B3BAC4` | décades, 0 dB |
| `grid_minor` | `#1A1F26` | `#E1E5EA` | graduations intermédiaires |
| `trace_live` | `#F2F4F7` | `#0A0C0F` | trace live |
| `coherence_band` | `#56B4E9` | `#0060A0` | bande de cohérence (opacité = γ²) |

Traces mémorisées (Okabe–Ito, dans l'ordre) : `#E69F00` orange, `#56B4E9` bleu ciel, `#009E73` vert bleuté, `#F0E442` jaune, `#0072B2` bleu, `#D55E00` vermillon, `#CC79A7` pourpre. En Plein jour, le jaune passe à `#B8A800` pour le contraste sur blanc.

### Espacement, rayons, typographie, traits

- Espacement : `xs 4`, `sm 8`, `md 12`, `lg 16`, `xl 24`, `xxl 32` (px logiques).
- Rayons : `small 4` (pastilles), `medium 6` (boutons, champs), `large 10` (notifications, palette).
- Texte (Inter) : `caption 12`, `label 13`, `body 14`, `title 16`, `heading 20`. Valeurs (JetBrains Mono, chiffres à chasse fixe) : `value 14`, `value_large 18`, `display 34` (délai).
- Traits : grille mineure 0,5 ; grille majeure 1 ; trace mémorisée 1,5 ; trace live 2,5 (halo 6, alpha 18 %) ; réticule 1 ; bord de focus 2.
- Zones cliquables ≥ 28 px de haut ; boutons principaux 32 px.

## Disposition (1440×900)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● Mesure  UAX 2-2 · 48 kHz · Réf. interne   Réf ▮▮▮▮▯ −18   Mic ▮▮▮▯▯ −24   12,43 ms  4,27 m · 597 éch.  ▮▮▮▮▯  [D] [A] │  56 px
├───────────────────────────────────────────────────────────┬──────────────────┤
│                                                           │ Traces │ Align. │ EQ │
│   tracé magnitude (dB / Hz log)                           │                  │
│                                                           │  panneau 320 px  │
│                                                           │  repliable       │
├───────────────────────────────────────────────────────────┤                  │
│   bande de cohérence (64 px)                              │                  │
├───────────────────────────────────────────────────────────┴──────────────────┤
│ FFT 32k · Moyenne 1 s · Lissage 1/48 · Seuil γ² 0,50 · Bruit rose −20 dBFS   [Espace Démarrer] [S Capturer] [F Figer] ⌘K │ 44 px
└──────────────────────────────────────────────────────────────────────────────┘
```

Zone centrale (tracé + cohérence) : ≥ 70 % de la surface avec le panneau ouvert à 1440×900.

## États

| État | Déclencheur | Retour visible | Suite proposée |
|---|---|---|---|
| Chargement / préchauffage | démarrage, fenêtre N pas encore pleine | pastille « Préchauffage » + barre fine de remplissage dans la barre supérieure, tracé grisé | aucune action, < 1 s |
| Vide | aucune mesure lancée | tracé vide avec message centré « Espace pour démarrer la mesure » + bouton | Démarrer |
| Erreur | carte déconnectée, flux invalidé, échec d'ouverture | bandeau `danger` non bloquant sous la barre supérieure | « Reprendre » |
| Partiel | cohérence < seuil, référence absente, latence non calibrée | points masqués (ruptures de courbe), bande hachurée ; bandeau `warning` avec la marche à suivre | « Calibrer », « Vérifier la référence » |
| Succès | délai trouvé, trace capturée, filtre ajouté | notification en bas à droite (≤ 150 ms d'apparition), avec action | « Appliquer », « Annuler » |
| Permission | micro refusé par macOS, entrée entièrement nulle | bandeau `danger` avec le chemin Réglages › Confidentialité › Micro | « Réessayer » |

## Interaction

- Tout se pilote au clavier ; chaque bouton principal affiche son raccourci (touche dessinée en `caption` monospace).
- Survol : `surface_hover` ; focus clavier : contour `accent` de 2 px ; actif : `accent` à 20 % d'opacité.
- Animations ≤ 150 ms, seulement pour l'apparition des notifications et des bandeaux, et le repli du panneau. Jamais sur les courbes.
- Mise à l'échelle Cmd/Ctrl + et − (facteur de zoom egui, de 0,75 à 2).
