---
paths: ["crates/dsp-core/**"]
---

# Définitions DSP et critères d'acceptation

`dsp-core` : maths pures, aucune I/O, aucun thread, aucun état global. Après toute modification : skill `audio-numerics-review`. Pour biquads, FFT, lissage : `dsp-algorithm-guide`.

## Définitions (à respecter exactement)

- x = référence, y = micro. Positif = micro en retard sur la référence.
- Fenêtre glissante de N points (8192 / 16384 / 32768 / 65536, défaut 32768 à 48 kHz), Hann, pas H indépendant de N (défaut 2048).
- Gxx = ⟨X*·X⟩, Gyy = ⟨Y*·Y⟩, Gxy = ⟨X*·Y⟩, moyennés en complexe.
- Moyennage exponentiel : α = 1 − exp(−H / (τ·fs)), τ ∈ {0,5 ; 1 ; 2 ; 4} s, ou cumul infini ; reset.
- H1 = Gxy / Gxx ; magnitude dB = 20·log10|H1|.
- γ² = |Gxy|² / (Gxx·Gyy) ; points avec γ² < seuil (défaut 0,5) masqués.
- Grille d'affichage : log au 1/96 d'octave de 20 Hz à 20 kHz (≈ 957 points), calculée une fois. Lissage, cohérence et EQ évalués sur cette grille.
- Lissage par fraction d'octave (aucun, 1/48, 1/24, 1/12, 1/6, 1/3) sur |H|² (puissance), puis dB.
- Délai : pic de |IFFT(Gxy)| sur FFT dédiée de 131072 points, interpolation parabolique sub-échantillon. Confiance = pic/bruit en dB.
- Compensation : retard entier en échantillons sur la référence + réglage fin (pas 0,01 ms) par rotation de phase e^{+jωτ} sur H.
- Distance = délai × c, c = 331,3·√(1 + T/273,15) m/s, T défaut 20 °C.
- Sommation : |H_A + p·H_B·e^{−jωΔ}|, Δ > 0 = retarder B, Δ < 0 = retarder A, p = ±1.
- Délai optimal : Δ ∈ [−50 ; +50] ms, recherche grossière puis fine au pas de 0,01 ms, p = ±1, bande [f1 ; f2] (défaut 60–150 Hz). Critères : niveau sommé moyen maximal, ou écart-type minimal du dB sommé.
- Moyenne de traces : moyenne en puissance de |H|², affichée en dB.
- EQ : biquads « Audio EQ Cookbook » (Bristow-Johnson) évalués en H(e^{jω}) sur la grille, à la fs de la mesure. Types : cloche, plateaux grave/aigu, coupe-bas/haut Butterworth 12/24 dB/oct et Linkwitz-Riley 24/48 dB/oct (cascades), notch. Q et largeur de bande en octaves affichables.
- Référence interne : délai acoustique = délai mesuré − latence système calibrée.

## Numérique

- `f64` pour coefficients, accumulateurs, spectres moyennés et calculs sur la grille.
- Toute division par Gxx, Gyy ou un module : garde par seuil nommé (`MIN_AUTO_SPECTRUM_POWER`). En dessous : point **invalide** (type explicite, ex. `Option` ou masque booléen), jamais NaN ni Inf en sortie.
- `log10` : argument plancher nommé (`MIN_POWER_FOR_DB`) ; les points invalides ne sont jamais convertis.
- Cohérence bornée à [0 ; 1] après calcul.
- Interpolation parabolique : dénominateur gardé ; repli sur le pic entier si dégénéré.
- Fonctions pures avec buffers de sortie fournis par l'appelant (`&mut [T]`) pour les chemins chauds : pas d'allocation par appel.
- Résultats déterministes : graines PRNG fixes dans les tests.

## Critères d'acceptation (tests automatisés, signaux synthétiques)

| Critère | Condition | Tolérance |
|---|---|---|
| Délai | bruit rose, y = x retardé de 3,2 ms (retard fractionnaire exact, 153,6 éch. à 48 kHz) + bruit décorrélé à 20 dB de RSB | ±0,02 ms |
| H1 | y = x filtré par cloche 1 kHz / +6 dB / Q 2 | erreur < 0,2 dB de 50 Hz à 18 kHz là où γ² > 0,95 |
| Cohérence | bruit décorrélé sur y à 0 dB de RSB | γ² = 0,5 ± 0,05 |
| Moyennage | échelon, H = 2048 vs H = 4096, même τ | constantes de temps identiques à 5 % |
| Biquad | gain au centre d'une cloche | ± 0,01 dB |
| Lissage | réponse plate | reste plate à ± 0,01 dB |
| EQ | trace avec cloche +6 dB + EQ cloche −6 dB mêmes paramètres | plat à ± 0,1 dB |
| Sommation | traces identiques, Δ = 0 | +6,02 dB |
| Sommation | Δ = demi-période à f | creux > 30 dB à f |
| Délai optimal | traces identiques, B retardée de 1,5 ms, bande 60–150 Hz | retarder A de 1,5 ms ± 0,02 ms, p = +1, pour les deux critères |
| Moyenne de traces | deux traces identiques | identique à chacune à ± 0,01 dB |
| Robustesse | bandes silencieuses (Gxx = 0) | aucun NaN/Inf, points marqués invalides |
| Référence interne | latence simulée 7,3 ms + délai acoustique 3,2 ms, après calibrage | délai acoustique = 3,2 ms ± 0,02 ms |

Un critère impossible à atteindre : s'arrêter et le signaler. Ne jamais élargir une tolérance.
