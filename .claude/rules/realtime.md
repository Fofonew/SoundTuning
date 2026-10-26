---
paths: ["crates/audio-io/**", "crates/engine/**"]
---

# Règles temps réel et budgets de performance

Après toute modification de ces crates : skill `audio-dsp-review`. En cas de régression de bench : `audio-performance-debug` + `ecc:benchmark-optimization-loop`.

## Chaîne de threads

```
callback entrée cpal ──rtrb──┐
callback sortie cpal ──rtrb──┤ (référence interne : échantillons émis + index de trame)
                             ▼
                     thread moteur (pas H, FFT N, accumulateurs, grille)
                             │            └── tâche délai (FFT 131072, ≤ 2/s)
                             ▼
              arc-swap / triple_buffer (instantané immuable)
                             ▼
                          UI egui (60 i/s, lit le dernier instantané, ne bloque jamais)
```

## Dans un callback audio (entrée ou sortie) : interdit

- **Allocation** : `Vec::push`/`extend`/`collect`, `Box::new`, `String`, `format!`, `to_string`, `Arc::new`, **libération du dernier `Arc`**, clonage d'un conteneur, closure qui capture par valeur un type alloué et le relâche.
- **Verrou** : `Mutex`, `RwLock`, `parking_lot`, `std::sync::mpsc` et canaux non bornés, `Condvar`, `OnceLock` en cours d'initialisation.
- **I/O et appels système** : fichiers, `println!`, macros `tracing` (`info!`, `warn!`…), `thread::sleep`, `yield_now`, horloge système autre que l'horodatage fourni par `cpal`.
- **Panique** : pas d'indexation `[]` pouvant déborder, pas de découpage de tranche non vérifié, pas d'`unwrap`. Utiliser `get`, `chunks_exact`, `zip`, `iter`.
- **Travail non borné** : boucles dont la durée dépend d'autre chose que la taille du tampon.

## Dans un callback audio : autorisé

- `rtrb::Producer::write_chunk_uninit` / `push` (sans allocation), atomiques (`Relaxed`/`Release`/`Acquire` justifiés par le nom), tampons et état pré-alloués, tableaux de taille fixe, arithmétique.
- Les erreurs (tampon plein, format inattendu) incrémentent un compteur atomique (`dropped_frames`, `overruns`) lu et journalisé par le thread moteur.
- Déconnexion : le callback d'erreur `cpal` lève un drapeau atomique ; la reprise est pilotée hors callback.

## Générateur

- Bruit blanc : PRNG déterministe à état local (xorshift/PCG maison sans allocation), graine fixe en test.
- Bruit rose : filtre de Paul Kellet ou Voss-McCartney, coefficients précalculés.
- Dénormalisés : pas d'`unsafe` donc pas de FTZ matériel ; vider en logiciel l'état des filtres à rétroaction (seuil nommé, ex. `DENORMAL_FLUSH_THRESHOLD`).
- Référence interne : chaque échantillon émis sur la sortie choisie est aussi poussé vers le moteur avec l'index de trame de sortie. Le moteur aligne x et y par index de trame et horodatages `cpal` ; la latence système calibrée est soustraite côté moteur, jamais dans le callback.

## Thread moteur

- Allocation autorisée à la (re)configuration uniquement (changement de N, H, fs, périphérique). En régime établi, chemin chaud sans allocation : plans FFT, tampons de travail et scratch pré-alloués.
- Tampons `rtrb` dimensionnés pour ≥ 1 s d'audio. Si le moteur prend du retard : jeter les plus anciens, lever un indicateur visible dans l'UI, ne jamais bloquer les callbacks.
- Instantanés publiés par `arc-swap` (ou `triple_buffer`) : structure immuable, l'UI ne copie que ce qu'elle dessine. Les anciens instantanés sont libérés sur le thread moteur ou UI, jamais dans un callback.
- Recherche de délai à basse cadence (≤ 2/s) sur un thread ou une tâche séparée pour ne jamais retarder les mises à jour à pas H.
- Mode fichier : même moteur, alimenté par un lecteur WAV à cadence réelle ou accélérée (tests), aucun flux `cpal` ouvert.

## Budgets (release, Apple Silicon)

| Mesure | Budget | Vérification |
|---|---|---|
| Mise à jour moteur (FFT 32k × 2 canaux, accumulateurs, γ², lissage sur grille ≈ 957 pts) | < 2 ms | bench criterion `engine_update` |
| Recherche de délai (FFT 131072) | < 15 ms | bench `delay_search` |
| Préparation d'une image de tracé depuis un instantané | < 1 ms | bench `plot_frame_prep` (dans `crates/app`) |
| Délai optimal entre deux traces | < 50 ms | bench `optimal_alignment` |
| Cadence des instantanés à 48 kHz, H = 2048 | ≥ 20/s (≈ 23,4) | test moteur en mode fichier |
| CPU en mesure continue, réglages par défaut | < 10 % d'un cœur | `docs/MANUAL_TESTS.md` |
| Callback audio | ≪ durée du tampon, aucun appel bloquant | revue `audio-dsp-review` |

Un bench au-dessus de son budget est une régression : diagnostiquer avant de continuer, ne jamais relever le budget.
