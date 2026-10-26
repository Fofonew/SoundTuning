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
