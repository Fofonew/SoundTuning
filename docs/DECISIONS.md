# Journal des décisions

Format : identifiant, date, contexte, décision, conséquences. Les plus récentes en bas.

---

### D-001 — Nom définitif : SoundTuning (2026-10-26)
- Contexte : « CalageLive » était un nom de travail.
- Décision : produit « SoundTuning », binaire `soundtuning`, bundle `SoundTuning.app`, titre de fenêtre « SoundTuning ».

### D-002 — CI : GitHub Actions (2026-10-26)
- Décision : `.github/workflows/ci.yml`, runners `macos-latest` et `windows-latest`. Aucun remote créé localement ; le fichier sert quand l'utilisateur poussera le dépôt.

### D-003 — Fréquence d'échantillonnage par défaut : 48 kHz (2026-10-26)
- Décision : valeurs par défaut de la spec inchangées (N = 32768, H = 2048, FFT délai 131072).

### D-004 — Référence interne en mode par défaut (2026-10-26)
- Contexte : l'utilisateur mesure avec une UAX 2-2, micro sur l'entrée 1, bruit rose sur la sortie G ou D, « la référence se fait au sein du soft ». La spec initiale ne prévoyait la référence interne que comme mode de secours marqué « moins fiable ».
- Décision : deux modes de premier plan (`docs/SPEC.md` §2.1) : **référence interne** (défaut) et **référence rebouclée** (indispensable pour mesurer sur la musique). En référence interne :
  - le générateur publie les échantillons émis vers le moteur avec leur index de trame ;
  - le délai mesuré inclut la latence aller-retour de la carte ; un **calibrage de latence** en un clic (câble sortie → entrée) la mémorise par (périphérique, fs, taille de tampon) et elle est soustraite du délai affiché ;
  - sans calibrage, l'UI affiche « latence non calibrée » et marque la distance comme approximative ;
  - le marquage « moins fiable » ne s'applique que si entrée et sortie sont sur des périphériques (donc horloges) différents.
- Conséquences : un critère d'acceptation ajouté (référence interne, latence simulée 7,3 ms) ; un état explicite ajouté (« latence non calibrée ») ; phases 2 et 5 étendues (alignement, calibrage).
- À vérifier sur le matériel : nombre réel de canaux de l'UAX 2-2 et stabilité de la latence entre redémarrages du flux (`docs/MANUAL_TESTS.md`).

### D-005 — Hook GateGuard d'ECC (2026-10-26)
- Décision (utilisateur) : dans le bloc `env` de `.claude/settings.json` :
  - `GATEGUARD_BASH_ROUTINE_DISABLED=1` : plus de blocage sur les commandes Bash courantes ;
  - `GATEGUARD_EXEMPT_GLOBS` : pas de blocage à la première écriture dans `crates/**`, `docs/**`, `assets/**`, `packaging/**`, `.github/**` et les fichiers de configuration racine (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, `README.md`, `CLAUDE.md`, `.gitignore`).
- Restent gardés : commandes Bash destructives, fichiers hors de ces chemins (dont `.claude/**`).
- Prise d'effet au redémarrage de la session Claude Code.

### D-006 — Revue des skills audio importés (2026-10-26)
- Source : `github.com/kunitoki/sonic-skills`, commit `8b6dcfe`. Aucune instruction suspecte (pas d'`allowed-tools`, pas de script, pas de commande réseau). Les quatre skills sont conservés.
- Adaptations à nos règles :
  - pas d'`unsafe` → pas de FTZ/DAZ matériel ; dénormalisés traités en logiciel (seuil nommé) ;
  - « commenter les constantes » → constantes nommées à la place (règle « pas de commentaires ») ;
  - « `Arc::clone` sans risque dans le callback » : incomplet, la libération du dernier `Arc` désalloue → interdit dans les callbacks ;
  - `assert_no_alloc` suggéré : hors stack, à justifier ici s'il est adopté.

### D-007 — Noms des agents ECC (2026-10-26)
- Les agents cités dans la spec (`rust-reviewer`, `rust-build-resolver`) sont fournis par le plugin ECC sous les noms `ecc:rust-reviewer` et `ecc:rust-build-resolver`.

### D-008 — Noms de crates et du binaire (2026-10-26)
- Crates du workspace : `dsp-core`, `audio-io`, `engine`, `app` (non publiées, dossiers `crates/<nom>`). Le crate `app` produit le binaire `soundtuning`.

### D-009 — Dépendance hors stack : `clap` (phase 0)
- Contexte : sous-commandes `devices` et `record`, option `--input-file`, aide en français.
- Décision : `clap` (derive) plutôt qu'un analyseur maison : validation des canaux (≥ 1), aide générée, messages d'erreur cohérents, testable (`Cli::command().debug_assert()`).

### D-010 — `serde` dans `audio-io` (phase 0)
- Les réglages de flux (`StreamSettings`, `ReferenceMode`, `NoiseKind`) seront sérialisés dans la session (phase 4) ; ils dérivent `Serialize`/`Deserialize` dès maintenant. `serde` fait partie de la stack.

### D-011 — Contrat entre `audio-io` et `engine` (phase 0)
- `audio-io` expose un trait `FrameSource` (`read_frames` non bloquant, trames `StereoFrame { reference, measurement }` déjà alignées).
- Implémentations : `LoopbackFrameSource` (référence rebouclée), `InternalReferenceAligner` (référence interne), `FilePlayback` (mode fichier temps réel), `RecordingCursor` (lecture instantanée pour les tests).
- Référence interne : le callback de sortie publie chaque échantillon émis dans un `rtrb` ; l'aligneur apparie l'échantillon émis k et l'échantillon capturé i avec k = i + round((t_capture₀ − t_lecture₀)·fs), à partir des horodatages `cpal` du premier callback de chaque flux. Le délai résiduel mesuré = délai acoustique + latence des convertisseurs, compensée par le calibrage (phase 5).
- « Horloges séparées » (référence interne moins fiable) : calculé d'après les identifiants des périphériques réellement ouverts, pas d'après les réglages (avec les réglages par défaut sur Mac, entrée = micro intégré, sortie = haut-parleurs).

### D-012 — Tampons `rtrb` pré-touchés (phase 0)
- Revue `audio-dsp-review` : les tampons de 2 s sont remplis puis vidés à l'ouverture pour éviter les défauts de page au premier callback.

### D-013 — Dénormalisés sans `unsafe` (phase 0)
- Pas de FTZ matériel (interdit sans `unsafe`). Le filtre de bruit rose remet à zéro ses états sous `DENORMAL_FLUSH_THRESHOLD` à chaque bloc.

### D-014 — Suites de la revue `ecc:rust-reviewer` de la phase 0
Corrigé :
- **Index de trame absolu** dans `InputFrame` et `GeneratorSample` (compteur par callback, incrémenté même si l'anneau est plein). L'aligneur apparie par index, plus par comptage : une perte de trames ne décale plus jamais référence et mesure ; elle est signalée par `ReadReport::discontinuity`.
- **Contrat `FrameSource`** : `read_frames` renvoie `ReadReport { frames_read, discontinuity, finished }` (fin de fichier, rebouclage, pertes).
- **Canaux du générateur validés** à l'ouverture (`NoGeneratorChannel`, index ≥ 64 refusé) : plus de référence non nulle alors que rien n'est joué.
- **Configuration de flux** : plus petit nombre de canaux suffisant (au lieu du maximum).
- **Mode fichier** : contre-pression (attente quand l'anneau est plein) au lieu de jeter des trames ; rebouclage signalé comme discontinuité.
- **Erreur d'ouverture du générateur en référence rebouclée** remontée (`LiveSession::generator_error`).
- Horodatage `cpal` non convertible ignoré au lieu d'une valeur sentinelle ; différence d'horodatages en `i128`.
- `too_many_arguments` supprimé (état partagé regroupé).
Non corrigé, justifié :
- **Reprise après déconnexion / changement de réglages** : on reconstruit un `LiveSession` complet (nouvelle horloge, nouveaux anneaux). Plus simple et sans état résiduel ; la reprise « en un clic » de la phase 5 utilisera ce chemin.
- **Base de temps `StreamInstant` entre deux périphériques distincts** : non vérifiable sans matériel. Le mode est déjà marqué « moins fiable » dans ce cas ; test ajouté dans `docs/MANUAL_TESTS.md`.
- **RMS du dernier bloc** pour les vu-mètres : suffisant pour l'affichage (la crête est maintenue jusqu'à lecture) ; la balistique est faite côté UI.
