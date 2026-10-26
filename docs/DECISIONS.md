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

### D-015 — Représentation des points invalides (phase 1)
- Par case FFT : `BinTransfer { response: Option<Complex64>, coherence: Option<f64> }`. Sur la grille : `Vec<Option<f64>>` / `Vec<Option<Complex64>>`. Jamais de NaN ni d'Inf : une valeur absente est `None`.
- Seuil de validité **relatif** : Gxx ≤ max(Gxx)·10⁻¹² (−120 dB) ou ≤ 10⁻³⁰ → case invalide. Indépendant du niveau d'entrée.

### D-016 — Projection sur la grille et lissage (phase 1)
- |H|² (ou γ², ou Gxy et Gxx pour le complexe) est interpolé linéairement entre cases FFT ; pour chaque point de la grille, on prend la **moyenne de l'interpolant** sur la bande [f·2^(−1/2b), f·2^(1/2b)] (intégrale exacte par sommes préfixées en trapèzes). Sans lissage : simple interpolation à f.
- Les cases invalides ont un poids nul ; un point de grille dont moins de 50 % de la bande est valide est invalide.
- Une réponse plate reste exactement plate (testé pour tous les lissages).
- Réponse complexe sur la grille (pour l'alignement et l'EQ) : rapport des moyennes de bande de Gxy et Gxx (lissage complexe), à appliquer après compensation du délai.

### D-017 — Moyennage exponentiel : première mise à jour (phase 1)
- La première mise à jour après un reset prend directement la valeur mesurée (poids 1), puis α = 1 − exp(−H/(τ·fs)). Évite une montée lente depuis zéro qui fausserait la cohérence au démarrage ; le τ mesuré sur un échelon reste exact (testé à 5 % pour H = 2048 et 4096).
- Accumulateurs remis à zéro sous 10⁻²⁸⁰ (pas de dénormalisés après un long silence).

### D-018 — Délai : détails (phase 1)
- Corrélation circulaire IFFT(X*·Y) sans fenêtre sur 131072 points ; pic de |r| ; interpolation parabolique sur |r| bornée à ±0,5 éch. ; index > N/2 = délai négatif. Polarité inversée signalée (`inverted_polarity`) quand le pic est négatif.
- Confiance = 20·log10(pic / RMS de |r| hors ±1 % de N autour du pic), plafonnée à 200 dB.
- `estimate_from_cross_spectrum` permet au moteur de moyenner Gxy dans le temps avant l'IFFT (définition « pic de |IFFT(Gxy)| »).

### D-019 — Délai optimal : critère « réponse la plus plate » (phase 1)
- Sans garde-fou, une annulation totale (polarité inversée, Δ = −décalage) donne une somme plate au plancher de −300 dB et gagnerait. Le critère « plus plate » ne retient donc que les combinaisons dont le niveau sommé moyen est au moins celui de la plus forte des deux traces seules ; les autres sont classées après, par niveau. Ce n'est pas un assouplissement du critère d'acceptation (testé : Δ = −1,5 ms ± 0,02, polarité normale, pour les deux critères).
- Recherche : pas grossier 0,1 ms sur ±50 ms pour p = ±1, puis affinage au pas de 0,01 ms autour du meilleur de chaque polarité. Δ calculé en pas entiers (pas d'accumulation d'erreur).
- Sommation : niveau plancher 10⁻³⁰ (−300 dB) au lieu d'invalider, pour afficher les creux profonds.

### D-020 — Filtres d'EQ (phase 1)
- Q utilisé pour cloche, plateaux (pente du plateau) et notch ; coupe-bas/haut à Q fixés : Butterworth 12 (Q 0,7071), Butterworth 24 (0,5412 et 1,3066), Linkwitz-Riley 24 (2 × 0,7071), Linkwitz-Riley 48 (2 × BW24). Fréquence bornée à [1 Hz ; 0,999·Nyquist], Q à [0,05 ; 100], entrées non finies neutralisées.
- Largeur de bande en octaves : BW = 2/ln2 · asinh(1/(2Q)).

### D-021 — Profil d'optimisation des tests (phase 1)
- `[profile.test] opt-level = 2` et dépendances optimisées en dev : les tests d'acceptation traitent jusqu'à 60 s de signal synthétique (cohérence) en moins d'une seconde.

### D-022 — Suites de la revue `ecc:rust-reviewer` de la phase 1
Corrigé :
- Échantillons spectraux non finis ignorés case par case dans l'accumulateur (un glitch ne bloque plus la mesure jusqu'au reset).
- Sommes préfixées **compensées** (Neumaier, partie haute + basse) pour le lissage : plus de faux trous ni de valeurs négatives sur des réponses à très forte dynamique (testé : 10⁴ puis 10⁻¹² → erreur < 0,1 %).
- Changer τ ne remet plus la mesure à zéro (seul le passage exponentiel ↔ cumul infini le fait) ; pas H ≥ 1 ; `set_hop_samples`.
- Recherche de délai optimal : plage bornée à ±1 s, bornes de bande ordonnées, extrémités ±50 ms évaluées (testé).
- Points de grille au-dessus de Nyquist invalides (fs < 40 kHz).
- EQ : cascades de sections en tableau fixe (`SectionCascade`), z⁻¹ calculé une fois par fréquence, aucune allocation dans `eq_response`.
- `rotate_phase_for_delay` vérifie les longueurs ; `DisplayGrid::new` et `GridProjector::new` bornent les paramètres invalides.
- Tests ajoutés : bornes de bande du lissage pour chaque fraction, dynamique extrême, Nyquist, délai sous-échantillon (±0,1 éch.), phase plate après compensation, comportement temporel de chaque type de filtre.
Non corrigé, justifié :
- Le délai s'estime sur 131072 échantillons bruts (x, y) et non sur le Gxy de l'analyse principale : c'est la FFT longue dédiée de la spec ; le moteur conserve 2,7 s d'historique.
- Cohérence triviale (= 1) après une seule moyenne : le moteur expose le nombre de moyennes et l'état « préchauffage » ; l'UI masque la cohérence tant que la fenêtre n'est pas pleine.
- Critère « niveau maximal » en moyenne de dB (et non de puissance) : favorise une sommation régulière sur toute la bande plutôt qu'un pic.
- Pas de documentation `///` : règle du projet « pas de commentaires d'explication », noms explicites.

### D-023 — Architecture du moteur (phase 2)
- `MeasurementCore` (synchrone, testable sans thread) : historique circulaire x/y de 65536 + 131072 + H trames en `f64`, une analyse par pas H (fenêtre N lue avec un décalage entier = compensation, côté référence si positive, côté mesure si négative), mise à jour des accumulateurs ; la projection sur la grille n'est faite qu'au moment de publier.
- Reste fractionnaire de la compensation : rotation e^{+j2πk·r/N} de Gxy (par récurrence) avant la projection complexe ; |H| et γ² ne sont pas concernés.
- Toute modification de la compensation remet les accumulateurs à zéro (les anciennes moyennes correspondent à un autre alignement).
- Thread moteur : lit la `FrameSource` par blocs de 4096, traite les commandes (`mpsc`), publie un `MeasurementSnapshot` par `triple_buffer` (écriture en place, aucune allocation en régime établi), dort 2 ms quand il n'y a rien à lire.
- Délai : thread dédié (`DelayEstimator::threaded`), un seul tampon de travail recyclé (aucune allocation, aucun retard des pas H) ; version `inline` pour les tests déterministes.
- Politique de délai : recherche automatique au démarrage, appliquée si confiance ≥ 12 dB ; suivi optionnel ; calibrage de latence = la prochaine mesure fiable devient la latence système et la compensation ; délai acoustique = délai appliqué − latence.
- Discontinuité (perte de trames, rebouclage du fichier) : la fenêtre glissante repart de zéro, les moyennes sont conservées.

### D-024 — Benchmarks (phase 2)
- Benchs criterion dans `crates/engine/benches/realtime_budgets.rs` ; journal dans `docs/BENCHMARKS.md`. Tous très en dessous des budgets ; `ecc:benchmark-optimization-loop` : pas de variante à essayer. Le bench `plot_frame_prep` arrivera avec le tracé (phase 3).

### D-025 — Suites de la revue `ecc:rust-reviewer` de la phase 2
Corrigé :
- **Contrat de discontinuité** : `ReadReport::discontinuity = true` signifie « rupture juste avant la première trame renvoyée ». Toutes les sources (`RecordingCursor`, `FilePlayback`, `LoopbackFrameSource`, `InternalReferenceAligner`) s'arrêtent avant une couture ou une perte et la signalent à l'appel suivant : aucune fenêtre ne mélange les deux côtés (testé avec un fichier rebouclé).
- **Suivi du délai** : zone morte d'un échantillon ; les moyennes ne sont plus remises à zéro à chaque estimation.
- **« Appliquer »** utilise la dernière estimation *fiable* (`latest_confident`), jamais une estimation sous le seuil.
- **Seuil de confiance relevé à 18 dB** : sur deux bruits sans aucune corrélation, le pic de corrélation sur 131072 retards atteint déjà ≈ √(2·ln 131072) ≈ 4,85 σ, soit ≈ 13,7 dB ; 12 dB laissait passer des délais aléatoires.
- **Calibrage** : marque une discontinuité (seules les données postérieures à la commande comptent), ignore tout résultat de délai lancé avant la commande (numéro de génération), abandonne après 10 s sans mesure fiable (`calibration_failed`).
- Une compensation manuelle avant la première estimation n'est plus écrasée par l'application automatique.
- Fin de source (`source_finished`) et discontinuités publiées immédiatement ; dérive exposée (`DelayStatus::drift_ms`, dernière estimation fiable − délai appliqué) pour la notification de recalage.
- Pas H borné à [N/32 ; N] ; lecture par blocs de H trames pour publier à chaque pas (≥ 20 instantanés/s mesurés sur le thread réel, testé).
- `apply_settings` atomique (nouvelles structures construites avant affectation) ; une erreur de pas ne fait plus perdre le reste du bloc et n'est journalisée qu'une fois par série.
- Canal de commandes borné (256) ; panique du thread moteur journalisée à l'arrêt ; grille passée par l'appelant.
Non corrigé, justifié :
- Pas de commande « pause / figer » côté moteur : figer est une fonction d'affichage (l'UI garde une copie de l'instantané) ; la mesure continue, conformément au workflow.
- Fichier en boucle plus court que 2,7 s : aucune estimation de délai (fenêtre de 131072 échantillons contigus). Limite documentée ; les fichiers de test font ≥ 3 s.
- Changement de source ou de fréquence : on recrée un `EngineHandle` (même grille partagée).

### D-026 — Interface : architecture (phase 3)
- `crates/app` : `design/` (jetons, polices, thèmes, widgets), `plot/` (vue log, graduations, polylignes, préparation d'image), `ui/` (barres, panneau, tracé, bandeaux, notifications, palette, assistant), `actions.rs` (catalogue unique : libellé + raccourci pour boutons, clavier et palette), `measurement.rs` (contrôleur : source live / fichier, moteur, vu-mètres avec balistique), `config.rs` (configuration persistante JSON).
- Direction visuelle : `docs/DESIGN.md` (instrument de mesure, chrome effacé, trace live neutre très contrastée avec halo, palette Okabe–Ito pour les traces mémorisées).
- Polices embarquées : **Inter 4.1** (rsms/inter, release v4.1) et **JetBrains Mono 2.304** (JetBrains/JetBrainsMono, release v2.304), toutes deux sous **SIL Open Font License 1.1** ; fichiers de licence dans `crates/app/assets/fonts/*-OFL.txt`. Les chiffres sont toujours en JetBrains Mono (chasse fixe = chiffres tabulaires, egui ne gérant pas `tnum`).
- Figer = copie de l'instantané côté UI ; la mesure continue.
- Raccourcis désactivés quand un champ texte a le focus (palette, renommage) ; Espace est consommé avant les widgets (Entrée active le bouton focalisé).
- Aucune fenêtre modale : palette = zone flottante non bloquante (Échap, clic extérieur), notifications en bas à droite, bandeaux d'état dans la zone centrale ; testé (`no_modal_window_opens_while_measuring`).
- Configuration : `~/Library/Application Support/SoundTuning/config.json` (macOS), `%APPDATA%\SoundTuning` (Windows) ; écriture atomique (fichier temporaire + renommage), débounce 0,8 s ; latence calibrée mémorisée par (entrée, sortie, fs, tampon).
- Tests UI : `egui_kittest` (rendu wgpu hors écran), source fichier non cadencée (`FilePacing::AsFastAsPossible`), périphériques factices pour l'assistant ; captures PNG dans `crates/app/tests/snapshots/` (macOS uniquement), sans comparaison pixel (le contenu dépend du moteur en temps réel).
- Plages 1–4 : plein spectre 20 Hz–20 kHz, sub 20–200 Hz, médium 150 Hz–4 kHz, aigu 2–20 kHz. Échelle dB par défaut −24…+18 dB.

### D-027 — Suites de la revue `ecc:rust-reviewer` de la phase 3
Corrigé :
- « Reprendre » / « Réessayer » (bandeaux carte déconnectée, micro refusé, échec de démarrage) relancent la mesure (`Action::RestartMeasurement`) au lieu de l'arrêter.
- Démarrage et arrêt de la mesure hors du thread UI : ouverture des flux, lecture du WAV et lancement du moteur sur un thread dédié (état « démarrage »), libération (jointure des threads moteur et délai) sur un thread d'arrêt. L'UI ne bloque plus.
- L'affichage (instantané, vu-mètres, figé) est remis à zéro à chaque démarrage et arrêt : plus de bandeau ni de délai d'une session précédente ; la latence calibrée n'est enregistrée que si elle vient du moteur courant (évite de l'attribuer à une autre carte).
- Raccourcis à touche seule : répétition de touche ignorée (Espace maintenu ne relance plus les flux en boucle), désactivés quand un widget a le focus (Espace active alors le bouton focalisé) ; modificateurs comparés exactement (Cmd+Z ≠ Maj+Cmd+Z).
- Assistant : erreur d'ouverture affichée, bouton « Actualiser » ; aperçu des canaux sans générateur, le bruit rose ne démarre qu'à l'étape « Test » ; canaux de mesure et de référence différents ; sorties ramenées à « Gauche » sur une carte mono.
- Barre inférieure : pastilles réglables au clavier (flèches) et annoncées comme `SpinButton` ; pastille « Réf. » non interactive ; niveau du générateur plafonné à −6 dBFS ; bouton « Panneau » (P) pour replier le panneau latéral.
- Tracé : identifiant stable (`magnitude-plot`), libellé AccessKit correct, taille minimale ; bandeaux d'état superposés en haut du tracé (le tracé ne saute plus quand un bandeau apparaît).
- Notifications : fermées au clic, expiration suspendue au survol.
- Configuration : fichier invalide conservé en `config.json.bak` ; valeurs bornées au chargement (fréquence, niveau du générateur, température, FFT, latences).
- Tests d'interaction sans GPU (rendu wgpu réservé aux captures, macOS) ; nouveaux tests : répétition de touche, Espace sur bouton focalisé, palette + Entrée, relance, arrêt qui efface l'affichage.
Non corrigé, justifié :
- Énumération des périphériques (`list_devices`) synchrone à l'ouverture de l'assistant et sur « Actualiser » : quelques millisecondes, action explicite.
- Captures sans comparaison pixel (contenu temps réel) : elles servent à la revue visuelle.
- Allocations résiduelles par image (segments de courbe pour `Shape::line`, textes des bandeaux) : mesurées négligeables (`plot_frame_prep` 0,05 ms) ; la médiane de cohérence n'est plus recalculée qu'à chaque nouvel instantané.

### D-028 — Traces et sessions (phase 4)
- Une trace conserve, **sur la grille d'affichage 1/96 d'octave (957 points)** : magnitude (dB, lissage du moment), γ², réponse complexe (même lissage, délai compensé), plus les métadonnées : fs, N, lissage, nombre de moyennes, délai mesuré, compensation appliquée, latence système → temps d'arrivée absolu = compensation − latence.
- **Écart avec la spec** (« H brut ») : H n'est pas stocké par case FFT (16 385 complexes par trace) mais sur la grille d'affichage, qui est la résolution de tout ce que l'outil affiche et calcule (lissage, cohérence, EQ, alignement). Session ≈ 50 Ko par trace au lieu de ≈ 600 Ko ; autosauvegarde rapide.
- Annuler / rétablir : pile d'états de session (100 pas) ; un glissé de décalage = une seule étape ; la capture se défait par Cmd+Z (notification avec action « Annuler »).
- Autosauvegarde : `session.json` (dossier de configuration) 2 s après la dernière modification, écriture atomique ; rechargée au lancement (reprise après plantage) ; sauvegarde aussi à la fermeture.
- Moyenne : moyenne en puissance des traces sélectionnées (décalages inclus), γ² = minimum des traces, nouvelle trace « Moyenne N (k traces) — HH:MM ».
- Export CSV : `~/Documents/SoundTuning/soundtuning-AAAAMMJJ-HHMMSS.csv`, séparateur `;`, une colonne dB et une colonne γ² par trace (cellule vide = point invalide).
- Nouvelle dépendance hors stack : **`chrono`** (horloge locale pour « Trace 3 — 21:42 » et les noms de fichiers ; la bibliothèque standard ne donne pas l'heure locale).

### D-029 — Revue `ecc:rust-reviewer` de la phase 4 (traitée, voir D-031)
- Verdict BLOQUANT. Points HIGH à corriger avant le commit « phase 4 » : écriture de la session hors du thread UI (thread dédié, `sync_all`), préservation d'un `session.json` illisible (renommé + notification + `.bak`), nouvel essai après un échec d'autosauvegarde, export CSV linéaire. Points MEDIUM : sélection séparée du renommage, Échap/F2, jeton d'annulation, historique en `Arc<Trace>` et hors sélection/visibilité/couleur, erreurs typées (64 traces, moyenne), CSV à virgule décimale et nom unique, tests complémentaires. Détail et ordre de traitement : section « État » de `CLAUDE.md`.

### D-030 — Interface des phases 5 et 6 amorcée (2026-10-26)
- Panneau Alignement (`ui/alignment_panel.rs`) : choix des traces A et B, critère « Niveau max » / « Plus plat », bande [f1 ; f2] en Hz (et Maj + glisser sur le tracé, à câbler), proposition calculée par `find_optimal_alignment`, bouton « Utiliser la proposition », réglage Δ au pas de 0,01 ms et polarité de B ; consigne affichée en clair à recopier dans le processeur (« Retarder A de 1,50 ms · B inversée »). L'état vit dans `Session.alignment` (annulable).
- Panneau EQ (`ui/eq_panel.rs`) : banques A/B, bypass global et par filtre, vues « Prédit » / « EQ seul » / « Trace inversée », cible (mesure en direct ou trace mémorisée), éditeur par filtre (type, fréquence, gain, Q, pente des coupe-bas/haut, largeur en octaves), ajout (E, 16 maximum), exports texte et JSON. L'état vit dans `Session.eq` (annulable).
- Notifications : dédoublonnées par action associée (une seule « Trace capturée — Annuler » visible), largeur 420 px, texte tronqué avant le bouton.
- Outil de packaging retenu pour la phase 8 : `cargo-packager` (crabnebula-dev) — `.app` macOS et installeur NSIS Windows depuis `[package.metadata.packager]` ; procédure Windows dans `docs/WINDOWS.md`.

### D-031 — Suites de la revue de la phase 4 (2026-10-26)
Corrigé :
- Autosauvegarde sur un thread `session-writer` dédié : canal borné (1), coalescence sur la dernière session, JSON compact, `sync_all` avant `rename` ; le thread UI ne fait plus que cloner la session (traces partagées en `Arc<Trace>`, donc clone en O(nombre de traces)). Écriture synchrone seulement à la fermeture (`finish`).
- `session.json` illisible : renommé `session.corrupt-AAAAMMJJ-HHMMSS.json`, la dernière sauvegarde valide (`session.json.bak`, conservée à chaque écriture) est rechargée, notification au lancement.
- Échec d'écriture : notification une seule fois (« nouvel essai automatique »), session remise en attente, nouvel essai 2 s plus tard ; l'alerte se réarme après une écriture réussie.
- `to_csv` linéaire (niveaux affichés calculés une fois par trace) ; nombres à virgule décimale (séparateur `;`, convention des tableurs français) ; nom de fichier unique (`-2`, `-3`… en `create_new`).
- Erreurs typées : `CaptureError` (aucune moyenne, 64 traces) et `AverageError` (moins de deux traces, 64 traces, fréquences d'échantillonnage différentes), chacune avec son texte. Moyenne : moyennes cumulées, compensation conservée seulement si identique.
- Ligne de trace : case « Sél. » dédiée (le double-clic ne fait plus que renommer), F2 renomme la trace sélectionnée, Échap annule, Entrée valide ; libellés AccessKit incluant le nom de la trace (« Supprimer la trace — Trace 2 — 21:42 »).
- La sélection n'entre plus dans l'historique (n'encombre pas Cmd+Z) ; historique en `VecDeque`.
- Tests : sélection + moyenne + annulation, F2 + Entrée, F2 + Échap, suppression + annulation, décalage + annulation, échec d'autosauvegarde notifié et en attente, session illisible signalée, `.bak` restauré, écrivain sur thread, limite de 64 traces, redo vidé après une nouvelle édition, CSV linéaire.
Non corrigé, justifié :
- Jeton d'annulation par notification : la notification « Annuler » expire en quelques secondes et toute édition ultérieure la remplace (dédoublonnage par action) ; Cmd+Z annule toujours la dernière édition, comme dans tout éditeur.
- Visibilité et couleur restent annulables : ce sont des choix d'affichage que l'opérateur veut pouvoir défaire (spec : annuler/rétablir « pour traces »).
- Glissé de décalage : les modifications successives d'une même trace forment une seule étape tant qu'aucune autre commande n'intervient (équivalent pratique de `drag_stopped`).

### D-032 — Délais et alignement (phase 5, 2026-10-26)
- Recherche automatique du délai au démarrage et application immédiate (moteur, `auto_apply_on_start`) ; suivi activable (T, bouton « Suivi » de la barre supérieure, mémorisé dans la configuration) : le moteur n'applique un nouveau délai que si sa confiance dépasse 18 dB.
- Dérive : sans suivi, si le dernier délai fiable s'écarte de plus de **0,5 ms** (≈ 17 cm) du délai appliqué, notification non bloquante « Délai changé : 14,10 ms (+1,67 ms) — Appliquer (A) » ; une seule fois tant que l'écart persiste, réarmée quand il redescend sous le seuil.
- Ajustement fin : ⇧← / ⇧→ et boutons − / + de la barre supérieure, pas de 0,01 ms (rotation de phase dans le moteur). Ce réglage de mesure n'entre pas dans l'historique d'annulation (comme « Appliquer »).
- Alignement : choisir A et B, le critère ou la bande recalcule la proposition (`find_optimal_alignment`) **et l'adopte** (Δ, polarité), pour que la somme prédite s'affiche immédiatement ; Δ et la polarité restent ajustables à la main, « Utiliser la proposition » revient à la proposition. Pas de proposition sans phase dans la bande (cas des moyennes de traces), message dédié.
- Bande à la souris : Maj + glisser sur le tracé quand l'onglet Alignement est ouvert ; aperçu pendant le glissé, proposition calculée une seule fois au relâchement ; bande bornée à 20 Hz–20 kHz et au moins 1/3 d'octave.
- Somme prédite tracée en `predicted` (rose, trait épais), mise en cache tant que traces, décalages, Δ et polarité ne changent pas.
- Annuler : les modifications continues (glissé de bande, Δ, décalage) forment une étape ; l'étape se ferme dès que le pointeur est relâché et qu'aucun champ n'a le focus.
- Notifications ancrées à gauche du panneau latéral quand il est ouvert (ne masquent plus ses commandes).
- Revue `ecc:rust-reviewer` : tous les points HIGH et MEDIUM corrigés sauf « libellé de bande alloué à chaque image » (une chaîne courte par image, négligeable).

### D-033 — EQ virtuel (phase 6, 2026-10-26)
- Cible : mesure en direct ou trace mémorisée ; EQ évalué sur la grille d'affichage à la fs de la cible (trace : fs de sa capture). Courbe d'EQ mise en cache tant que l'état d'EQ et la fs ne changent pas.
- Vues : « Prédit » = |H·EQ| (somme des dB, exacte pour la magnitude) + courbe d'EQ fine ; « EQ seul » ; « Inversée » = trace inversée autour de son niveau moyen 200 Hz–5 kHz (pour poser l'EQ dessus) + courbe d'EQ.
- Poignées sur le tracé (onglet EQ ouvert) : position (fréquence, gain) ; glisser = fréquence et gain (gain ignoré pour coupe-bas/haut et notch), molette sur une poignée = Q (×e^{0,004·Δ}), clic = sélection, double-clic sur le tracé = cloche ajoutée à l'endroit cliqué (dans l'onglet EQ, le double-clic n'efface plus la vue). Bornes : 20 Hz–20 kHz, ±24 dB, Q 0,1–20. E ajoute une cloche à 1 kHz et ouvre l'onglet EQ.
- Annuler : un glisser ou une rafale de molette = une étape.
- Exports : `soundtuning-AAAAMMJJ-HHMMSS.txt` (liste lisible) et `.json` (filtres de la banque active) dans `~/Documents/SoundTuning`.
- Tests : critère « EQ » de la spec côté app (cloche +6 dB + EQ −6 dB → plat ±0,1 dB), glisser, molette, double-clic, exports, annulations.
- Revue `ecc:rust-reviewer` de la phase 6. Corrigé : sélection jamais posée sur un indice hors limites (glisser périmé après changement de banque, suppression ou annulation) et glisser annulé si la poignée disparaît ; chaque nouveau glisser de poignée ouvre une nouvelle étape d'annulation ; double-clic sur une poignée n'ajoute plus de filtre ; clic dans le vide désélectionne ; la poignée garde le décalage de prise (ne saute plus sous le curseur) ; annuler/rétablir conserve la vue d'EQ courante ; trace inversée : repli sur la moyenne globale si la bande 200 Hz–5 kHz est vide.
- Non corrigé, justifié : 4 à 5 vecteurs de 957 points alloués par image en mode EQ (≈ 40 Ko, mesuré négligeable devant le budget de 1 ms ; la courbe d'EQ elle-même est en cache) ; fs de repli 48 kHz tant qu'aucune mesure n'a donné la sienne (la courbe d'EQ seule reste utile) ; noms d'export déjà uniques (`create_new` + suffixe `-2`, `-3`…), contrairement à ce que suppose la revue.

### D-034 — Finition et performance (phase 7, 2026-10-26)
- Profil (`audio-performance-debug`, boucle `ecc:benchmark-optimization-loop`) : ligne de base mesurée, aucune variante nécessaire, tous les budgets tenus avec une large marge.

| Mesure | Médiane | Budget | Charge estimée |
|---|---|---|---|
| `engine_update` (FFT 32k ×2, accumulateurs, cohérence, lissage) | 0,62 ms | 2 ms | 0,62 ms / 42,7 ms par pas ≈ 1,5 % d'un cœur |
| `delay_search` (FFT 131072) | 1,78 ms | 15 ms | ≤ 2/s ≈ 0,4 % |
| `optimal_alignment` | 2,96 ms | 50 ms | au relâchement de la bande / changement de trace |
| `plot_frame_prep` | 0,053 ms | 1 ms | — |
| `ui_frame_cpu_frozen/traces` (image complète egui hors GPU, 6 traces) | 0,19 ms | — | 60 i/s ≈ 1,2 % |
| `ui_frame_cpu_frozen/alignment` | 0,15 ms | — | ≈ 0,9 % |
| `ui_frame_cpu_frozen/eq` (8 filtres) | 0,25 ms | — | ≈ 1,5 % |

- Total estimé hors rendu GPU ≈ 3,5 % d'un cœur (< 10 % exigés) ; la mesure réelle (Moniteur d'activité) et le démarrage < 1 s restent dans `docs/MANUAL_TESTS.md`. Cadence ≥ 20 instantanés/s couverte par `snapshots_update_more_than_twenty_times_per_second_at_48_khz` et `threaded_engine_publishes_at_least_twenty_snapshots_per_second_in_real_time`.
- Nouveau bench `crates/app/benches/ui_frame.rs`, groupe `ui_frame_cpu_frozen` (informatif, sans budget dans la spec ; repère : < 4 ms pour garder 60 i/s avec marge) : affichage figé (le chemin de mise à jour des courbes est couvert par `plot_frame_prep`), coût CPU egui complet **y compris** la construction de l'arbre AccessKit, hors GPU.
- Revue `ecc:rust-reviewer` de la phase 7 : libellés posés par `Response::accessible_name` (plus de second événement AccessKit), glyphes de la bascule « Suivi » en constantes, nom du groupe de bench explicite. Non corrigé : les groupes du bench partagent un même harnais dans un ordre fixe (Traces → Alignement → EQ), assumé.
- Accessibilité : libellés AccessKit obligatoires sur boutons, cases, champs numériques et listes (utilitaires `spin_label`, `combo_label`), vérifiés par test dans les trois onglets.

### D-035 — Préparation Windows, CI et packaging (phase 8, 2026-10-26)
- Packaging : `cargo-packager` 0.11.8 (D-030), configuration dans `crates/app/Cargo.toml`. macOS : `SoundTuning.app` (identifiant `app.soundtuning.desktop`, macOS 11 minimum) ; `crates/app/packaging/macos/Info.plist` est fusionné dans le plist généré et apporte `NSMicrophoneUsageDescription` (texte français), `CFBundleDevelopmentRegion = fr`, `NSHighResolutionCapable`. Windows : installeur NSIS `soundtuning_<version>_x64-setup.exe`. Aucune signature ni notarisation (spec).
- Vérifié localement : `.app` construit (`target/tools/bin/cargo-packager --release --formats app`, outil installé dans `target/tools`, rien hors du dépôt), `Info.plist` contrôlé avec `plutil`, binaire du bundle exécutable (`--help`), archive `SoundTuning-macos.zip` (7,5 Mo) via `ditto`. Test `packaging.rs` : la clé `NSMicrophoneUsageDescription` et le chemin du plist restent présents.
- Pas de `.dmg` : sa création par `cargo-packager` pilote le Finder par AppleScript, qui échoue sans session graphique (constaté en local : « AppleEvent timed out ») et de façon aléatoire sur les runners ; la CI publie le `.app` zippé avec `ditto`.
- L'installeur NSIS n'est pas construit en local (pas de Windows) : il l'est par le job CI `package` sur `windows-latest`, procédure de test manuelle dans `docs/WINDOWS.md`.
- CI GitHub Actions (`.github/workflows/ci.yml`) : `lint` (fmt, clippy `-D warnings`, macOS) ; `test` (build release, tests, compilation des benchs, macOS et Windows ; tests d'interaction `egui_kittest` sans GPU sur les deux, captures wgpu réservées à macOS) ; `windows-cross-check` (cible `x86_64-pc-windows-msvc` depuis macOS, comme `verify`) ; `package` (après `lint` et `test`, artefacts `.zip` macOS et `-setup.exe` Windows). Aucun remote n'est configuré : la CI tournera au premier push de l'utilisateur.
- Feature `asio` : documentée (SDK Steinberg à fournir, `CPAL_ASIO_DIR`, LLVM), exclue de la CI et de `verify` (SDK non redistribuable).
- Revue `ecc:rust-reviewer` de la phase 8 (aucun point bloquant) : version de `cargo-packager` épinglée (0.11.8) en CI ; CI déclenchée sur `main` et sur les pull requests, avec annulation des exécutions concurrentes, permissions en lecture seule et rétention des artefacts de 14 jours ; build release redondant retiré du job `test` (le job `package` le fait) ; test de packaging lisant le chemin du plist dans le manifeste et vérifiant le fichier. Reportés : icône d'application (icône par défaut), signature ad hoc ; quarantaine Gatekeeper documentée dans `docs/MANUAL_TESTS.md`. Le job `windows-cross-check` est conservé pour garder la parité avec `verify`.
- `bench = false` sur les cibles lib et bin des quatre crates : la commande de la skill `verify` (`cargo bench --workspace --benches -- --noplot`) incluait sinon le harnais libtest de ces cibles, qui refuse `--noplot`. Seuls les benchs criterion de `benches/` sont exécutés.
- `crates/app/tests/snapshots/` est ignoré par git : les captures y sont réécrites à chaque `cargo test` (nom du fichier de test et heure des traces), ce qui salissait l'arbre après chaque `verify`. Les captures de référence versionnées sont les copies de `docs/ui-review/phase-N/`.

### D-036 — Réglages audio et termes d'EQ en anglais (demande de l'utilisateur, 2026-10-26)
- EQ : types de filtres en anglais, vocabulaire usuel des processeurs (PEQ, Low shelf, High shelf, Low cut, High cut, Notch), ainsi que « Global bypass » et « Bypass ». Exception assumée à la règle « interface en français » ; le reste de l'interface reste en français. Les exports texte utilisent les mêmes noms.
- Réglages audio : bouton « Audio » dans la barre inférieure et raccourci ⌘, / Ctrl+, ; page « Réglages audio » dans le panneau latéral (pas de fenêtre modale, la mesure continue) : entrée, sortie, fréquence d'échantillonnage (celles de l'entrée), taille de tampon, référence interne ou rebouclée, canal du micro, canal de référence ou sortie du bruit rose. Les modifications restent en brouillon ; « Appliquer les réglages » enregistre la configuration et relance la mesure si elle tourne. Canaux, fréquence et sortie du générateur sont ramenés dans les limites du périphérique choisi.

### D-037 — Auto EQ façon REW (demande de l'utilisateur, 2026-10-26)
- Section repliable « Auto EQ » de l'onglet EQ, appliquée à la mesure choisie dans la liste « Mesure en direct / trace ». Paramètres mémorisés dans la session : plage (défaut 40 Hz–12 kHz), max boost (défaut +3 dB, 0 = coupures seules), max cut (défaut −12 dB), max filtres (défaut 8, ≤ 16). « Calculer l'auto EQ » remplace les filtres de la banque active (annulable).
- Algorithme (`dsp_core::auto_eq`, pur, testé) : niveau cible = médiane de la réponse dans la plage (insensible aux résonances) ; écart lissé sur ±1/12 d'octave ; placement glouton d'un PEQ au plus grand écart corrigeable (gain borné par max boost / max cut, Q tiré de la largeur à mi-hauteur de l'écart, Q 0,3–10), au moins 1/6 d'octave entre filtres ; affinage par descente de coordonnées (fréquence, gain, Q) minimisant l'écart RMS. Le max boost est une contrainte stricte sur la courbe d'EQ **cumulée** (gains de boost réduits puis mouvements refusés s'ils la dépassent), comme l'« overall max boost » de REW. Filtres < 0,2 dB retirés, triés par fréquence.
- Saisie numérique : tous les champs acceptent le format affiché (virgule décimale, signe −, unités, « k » pour kHz), via `text::parse_number`.

### D-038 — Sortie automatique et négociation de la fréquence d'échantillonnage (demande de l'utilisateur, 2026-10-26)
- Constat : sans carte son (micro intégré + haut-parleurs du portable), la mesure échouait avec « no output configuration supports 48000 Hz ». Cause : sans sortie choisie, le générateur était ouvert sur le périphérique **d'entrée** ; un micro seul n'a aucune configuration de sortie.
- Sortie « Automatique » (`output_device_id = None`) : le périphérique d'entrée s'il a une configuration de sortie utilisable avec assez de canaux pour le générateur (carte son duplex, pilote ASIO), sinon la sortie par défaut du système. Même règle dans `audio-io` (`OutputCandidate::resolve`) et dans l'interface (`device_choice::resolved_output`). Le libellé affiche la sortie retenue (« Automatique — … »).
- Fréquence : en référence interne, l'entrée et la sortie doivent tourner à la même fréquence (la référence est le signal émis, aligné échantillon par échantillon ; pas de rééchantillonnage, qui dégraderait la justesse). Ordre d'essai : fréquence demandée, puis 48 ; 44,1 ; 96 ; 88,2 kHz, parmi celles que les deux périphériques acceptent avec assez de canaux et un format d'échantillons pris en charge (`negotiate_sample_rate`). Si la fréquence change, la mesure démarre quand même et une notification l'indique ; la fréquence choisie par l'utilisateur reste enregistrée (elle redevient effective quand le matériel la permet).
- Latence calibrée : la clé est construite sur la route **réellement ouverte** (`AudioRoute` : identifiants d'entrée et de sortie résolus, fréquence effective, tampon), et non sur les réglages. Changer la sortie par défaut du système ou basculer en 44,1 kHz ne réutilise donc jamais la latence d'un autre trajet. Les calibrages existants d'une carte choisie explicitement gardent la même clé. Aucune fréquence commune : erreur en français listant les fréquences de chaque côté et proposant une autre sortie ou la référence rebouclée.
- Référence rebouclée : la sortie du générateur (facultative) n'a pas besoin de la même fréquence que l'entrée ; elle prend celle de l'entrée si possible, sinon sa propre fréquence préférée.
- Réglages audio : la liste des fréquences est l'intersection entrée ∩ sortie en référence interne ; `DeviceInfo` sépare les fréquences d'entrée et de sortie. Écart assumé : ces fréquences sont calculées pour au moins un canal ; `audio-io` exige en plus les canaux choisis et renégocie ou signale l'erreur au démarrage.
- CLI `record` : même résolution ; une ligne signale la fréquence adoptée si elle diffère de `--sample-rate`.
- Erreurs `AudioIoError` traduites pour l'interface (`text::audio_error`) au lieu du message anglais brut.

### D-039 — ASIO sous Windows (demande de l'utilisateur, 2026-10-26)
- Les périphériques de **tous** les pilotes disponibles (`cpal::available_hosts`) sont listés : WASAPI et, dans un build `--features asio`, chaque pilote ASIO installé, affiché « Nom (ASIO) ». L'identifiant cpal enregistré contient le pilote (`asio:…`, `wasapi:…`), l'ouverture passe donc par le bon hôte sans réglage supplémentaire. Choisir une entrée ASIO remet la sortie sur « Automatique » si la sortie choisie dépend d'un autre pilote, donc sur le même pilote ASIO (duplex, une seule horloge).
- Formats d'échantillons acceptés en plus : `I24` et `F64` (pilotes ASIO en 24 bits ou flottant 64 bits).
- Un pilote ASIO ouvert par la mesure n'est plus énumérable (cpal s'arrête sur « DriverAlreadyExists ») : pendant la mesure, les périphériques sélectionnés absents de la nouvelle liste sont repris de la liste précédente pour ne pas vider les réglages.
- Build : `asio-sys` 0.4 télécharge le SDK ASIO de Steinberg pendant la compilation (ou utilise `CPAL_ASIO_DIR`) et a besoin de LLVM (`bindgen`). La CI construit l'installeur Windows **avec** ASIO (`cargo build --release -p app --features asio`, LLVM préinstallé sur `windows-latest`) et vérifie la compilation ASIO dans le job `test` Windows. `verify` et le contrôle croisé depuis macOS restent sans `asio` (le script de build exige un hôte Windows).
- `before-packaging-command` retiré du manifeste : il recompilait sans la feature et écrasait le binaire ASIO. La compilation release est désormais une étape explicite avant `cargo packager` (CI et documentation).
- Licence : le binaire ASIO intègre du code du SDK Steinberg ; ses conditions de licence s'appliquent à toute diffusion de l'installeur (à vérifier avant une diffusion publique). Usage interne : pas d'impact identifié.
- Non testable ici (pas de PC Windows ni de pilote ASIO) : procédure dans `docs/WINDOWS.md`.

### D-040 — Vues Phase et réponse impulsionnelle, superposition, plein écran (demande de l'utilisateur, 2026-10-26)
- Hors périmètre V1 dans la spec (§ « Hors périmètre V1 »), ajoutées à la demande de l'utilisateur.
- Phase : argument de H1 (après compensation du délai, donc phase « alignée »), lissée en complexe comme la réponse, masquée sous le seuil de cohérence comme la magnitude, repliée à ±180° (le tracé est coupé à chaque saut > 180° au lieu d'un trait vertical). Axe fréquence partagé avec la magnitude (zoom, plages 1–4). Traces : phase tirée de leur réponse complexe ; moyennes de traces sans phase (moyenne en puissance).
- Réponse impulsionnelle : IFFT(Gxy compensé / Gxx) sur N points à chaque instantané (bins avec Gxx sous le seuil mis à zéro), recentrée : t = 0 = délai appliqué (axe « ms · 0 = délai appliqué ») ; échelle linéaire normalisée au plus grand pic affiché, ou dB (20·log10 |h| relatif, plancher −60 dB, sans enveloppe de Hilbert). Réticule : temps et distance (vitesse du son selon la température). Traces : fenêtre −10 / +100 ms enregistrée à la capture.
- Disposition : « Vue » principale (Magnitude / Phase / IR) et « Superposer » = deuxième vue empilée au-dessus (Phase au-dessus de la magnitude par défaut, convention des analyseurs double FFT), mémorisée dans la configuration. Interprétation retenue de « superposer » : deux tracés empilés (axes distincts), pas deux grandeurs sur les mêmes axes.
- Plein écran (Maj+F, Échap pour sortir, bouton « Plein écran ») : masque le panneau latéral et la barre inférieure, garde la barre supérieure (délai, état) et passe la fenêtre en plein écran ; resynchronisé si le système quitte le plein écran.
- Raccourcis : M, H (pHase, P étant déjà le panneau), I, V, Maj+F.
- Coût : `engine_update` 0,61 → 1,02 ms (budget 2 ms ; IFFT N + capture du spectre, ≈ 2,4 % d'un cœur au pas par défaut) ; image complète hors GPU : Phase sur Magnitude 0,30 ms, Magnitude sur IR 0,69 ms (enveloppe IR + 4 courbes), soit ≈ 4 % d'un cœur à 60 i/s dans le pire cas.

### D-041 — Changer la définition (lissage) d'une trace après capture (demande de l'utilisateur, 2026-10-26)
- Exigence : une trace re-lissée doit être identique à une mesure faite directement avec ce lissage (justesse avant tout), pas un re-lissage approché d'une courbe déjà lissée.
- Méthode : le lissage du projecteur est la moyenne, sur l'axe des bins, de l'interpolation linéaire des valeurs (intégrale / largeur). Les intégrales sont additives : à la capture, on enregistre pour chaque cellule de 1/96 d'octave entre deux points de grille consécutifs (16 cellules de marge de chaque côté) les intégrales utilisées par le projecteur (|H|²·w, w, γ²·w, w, Re/Im Gxy, Gxx, largeur en bins), plus les valeurs ponctuelles du réglage « aucun ». Un lissage 1/k somme 2·(48/k) cellules : résultat identique au projecteur à 1e-9 près (test `dsp-core` pour les 6 lissages, FFT 8k/32k, 32 / 44,1 / 48 kHz ; test moteur : courbe re-projetée = courbe live à 1e-9 dB). Lissages dont les bords ne tombent pas sur des cellules : refusés.
- Interface : ligne « Définition » de chaque trace (liste aucun / 1/48 … 1/3), annulable. Traces des sessions précédentes et moyennes de traces : définition fixe (infobulle explicative).
- Taille : ≈ 380 Ko de JSON par trace (spectre + IR), soit ≈ 4 Mo pour 10 traces et ≈ 24 Mo au maximum de 64 traces ; la sérialisation se fait sur le thread d'écriture de la session, jamais sur celui de l'interface. Piste si besoin : format binaire ou compression.
- `serde` ajouté à `dsp-core` (dérivation seulement, déjà dans la pile) pour stocker `CapturedSpectrum` ; les intégrales non finies sont remises à 0 à la capture (JSON n'encode pas NaN/Inf).

### D-042 — Recherche de mise à jour au lancement, sans dépendre du réseau (demande de l'utilisateur, 2026-10-26)
- Au lancement (et via la palette : « Rechercher les mises à jour »), un thread `update-check` interroge `https://api.github.com/repos/Fofonew/SoundTuning/releases/latest` (délais : 4 s pour la connexion, 10 s au total) ; l'interface lit le résultat par un canal, sans jamais attendre. Hors connexion, proxy bloquant ou réponse illisible : rien ne s'affiche au lancement (journal `info` seulement), le logiciel fonctionne normalement ; la recherche manuelle affiche une notification « injoignable ». 404 (aucune release publiée) = à jour.
- Nouvelle version (tag `vX.Y.Z` > version compilée, brouillons et préversions ignorés) : bandeau non bloquant « SoundTuning X.Y.Z est disponible » avec « Télécharger » (ouvre la page de la release dans le navigateur) et « Plus tard » (masque jusqu'au prochain lancement). Pas de téléchargement ni d'installation automatiques (choix de l'utilisateur) : l'installation se fait hors mesure. Le lien n'est ouvert que s'il pointe vers `https://github.com/Fofonew/SoundTuning/releases/`, sinon la page des releases.
- Dépendance hors pile : `ureq` 3.5 (client HTTP bloquant, petit, sans runtime async), fonctionnalité `native-tls-no-default` : TLS du système (Security.framework, SChannel) et certificats racines du système, donc compatible avec les proxys d'entreprise à certificat maison, et rien à compiler en C (le contrôle croisé Windows depuis macOS reste possible, ce que `ring`/rustls n'aurait pas permis). Version comparée par un analyseur maison de `X.Y.Z` (pas de dépendance `semver`).
- Tests : aucune requête réseau ; `UpdateSource::Offline` et `UpdateSource::Fixed` simulent les réponses (tests unitaires et `egui_kittest`).
- Le dépôt doit être public : un dépôt privé renvoie 404 sans jeton, et embarquer un jeton dans l'exécutable le rendrait public (choix de l'utilisateur : rendre le dépôt public).

### D-043 — Publication des versions (2026-10-26)
- `scripts/release.sh X.Y.Z` (sur `main`, arbre propre) : met la version du workspace à jour, commit « version X.Y.Z », tag annoté `vX.Y.Z`, sans rien pousser. Les dépendances internes n'ont plus de numéro de version (chemins seuls, crates non publiées) pour que le changement de version ne casse pas la résolution.
- Pousser le tag déclenche `.github/workflows/release.yml` : vérifie que le tag égale la version de `Cargo.toml`, relance toute la CI (`ci.yml` devenu réutilisable : lint, tests macOS et Windows, contrôle croisé, paquets), puis publie la release GitHub avec `SoundTuning-X.Y.Z-macos.zip` et `SoundTuning-X.Y.Z-windows-x64-setup.exe` (notes générées depuis les commits). Versions `X.Y.Z` uniquement (pas de préversion : la comparaison de versions n'en gère pas).
- Paquets non signés (SmartScreen, Gatekeeper) : inchangé, voir D-035 et `docs/WINDOWS.md`.

### D-044 — Petites fenêtres et gel après réduction sous Windows (signalé par l'utilisateur, 2026-10-26)
- Constat : sous Windows à 125–150 % d'échelle, la fenêtre « par défaut » (1440 × 900 points) ne tient pas à l'écran (1920 × 1080 à 150 % = 1280 × 720 points, barre des tâches comprise) ; la barre inférieure se chevauchait (« Réf. interne » sur « Commandes ») et le bas des réglages audio était coupé. Reproduit avec `egui_kittest` à 1280 × 680 et 1,25 de facteur d'échelle.
- Corrections : barre inférieure en 4 densités choisies d'après les largeurs mesurées (raccourcis des boutons discrets masqués, puis tous les raccourcis, puis « Réf. » et « Commandes », toujours accessibles par la palette et ⌘K) ; barre supérieure compacte (nom du logiciel et raccourcis masqués) ; réglages audio : boutons et aide fixés en bas, champs dans une zone défilante ; fenêtre agrandie (maximisée) au lancement si elle dépasse l'écran ; taille minimale abaissée de 1280 × 720 à 1024 × 600. Captures : `docs/ui-review/fenetre-reduite/`.
- Gel et fenêtre noire après réduction : non reproductible ici (macOS, rendu hors écran). Hypothèse : le moteur Vulkan choisi par défaut par wgpu sur certains pilotes Windows, connu pour bloquer à la restauration d'une fenêtre réduite. Sous Windows, le rendu passe donc par DirectX 12 (natif, présent sur Windows 10/11), sauf si la variable `WGPU_BACKEND` impose un autre moteur (`WGPU_BACKEND=vulkan` ou `gl` pour comparer). Pas de repli automatique vers Vulkan (suspect) : DirectX 12 existe sur tout Windows 10/11, au besoin par l'adaptateur logiciel WARP. À confirmer par le test manuel ; si le gel persiste, essayer `WGPU_BACKEND=gl` et le signaler.

### D-045 — Alignement façon SysTune « Delay Analysis » (demande de l'utilisateur, 2026-10-26)
- Source : manuel AFMG SysTune 1.3, § 5.7 « Delay Analysis » (lu le 2026-10-26).
- Traces : « Reference » et « Delay » (la trace Delay est celle que l'on retarde) au lieu de « Trace A / B ». Convention de la spec inchangée : Reference = A, Delay = B, Δ > 0 = retarder Delay, Δ < 0 = retarder Reference ; la polarité s'applique à Delay.
- Quatre solutions affichées en colonnes, comme SysTune : **Pos** (niveau sommé moyen maximal, polarité normale), **Neg** (idem, polarité inversée), **Flat** (somme la plus plate, polarité indiquée), **Custom** (délai et polarité saisis, pas de 0,01 ms). Barres sous Pos et Neg : puissance relative au meilleur des deux. Au calcul, Pos ou Neg (la plus forte) est sélectionnée, sauf si l'utilisateur a choisi Flat ou Custom.
- Prédiction : les quatre courbes de sommation sont tracées (couleur de leur colonne) ; la solution sélectionnée en trait épais sur tout le spectre, les autres en trait fin dans la plage seulement. Custom est recalculé à chaque modification (temps réel), Pos / Neg / Flat à chaque changement de traces ou de plage. Un seul balayage calcule les trois solutions (`analyze_alignment`) : 3,1 ms en Low (bench `optimal_alignment`, budget 50 ms), ≈ 23 ms estimés en Full (957 points).
- Plages : **Low** 50–200 Hz, **Mid** 3 octaves centrées sur 1 kHz, **Full** 20 Hz–20 kHz (définitions SysTune), **Custom** = point de raccord (Hz) + largeur (octaves, 1/3 à 6), aussi réglable par Maj + glisser sur le tracé. Écart assumé avec la spec (§ délai optimal, bande par défaut 60–150 Hz) : la plage par défaut de l'interface est Low (50–200 Hz) comme SysTune ; le critère d'acceptation de `dsp-core` reste testé tel quel avec 60–150 Hz.
- Ancienne sauvegarde de session : l'ancien réglage (traces A/B, bande, délai appliqué) n'est pas repris (champs ignorés au chargement) ; les traces sont conservées.
- Non repris de SysTune : calcul sur réponses fenêtrées (WINDOWED), panneau graphique séparé (les sommes sont sur le tracé principal).

### D-046 — Termes techniques en anglais (demande de l'utilisateur, 2026-10-26)
- Boutons et étiquettes : Start / Stop, Freeze / Unfreeze, Capture, Peak (recherche du délai, ex-« Trouver »), Insert (ex-« Appliquer » le délai, terme de Smaart), Track (suivi), états Live / Warm-up / Frozen / Stopped, Ref, Mic, CLIP, Delay, Avg, Smoothing, Coh γ², Pink noise, Coherence, View, Top, Sample rate, Buffer, Internal / Loopback, onglet Alignment, Delay Analysis (Reference, Delay, Range, Crossover, Pos, Neg, Flat, Custom). Termes d'EQ déjà en anglais (D-034).
- Phrases (aides, bandeaux, notifications, palette) : en français, avec le terme anglais du bouton (« Peak : trouver le délai »), pour que la palette retrouve les deux.
- « Insert » (Smaart) retenu par l'utilisateur pour appliquer le délai.

### D-047 — Icônes (demande de l'utilisateur, 2026-10-26)
- Police Phosphor Regular (licence MIT, `crates/app/assets/fonts/Phosphor-MIT.txt`) embarquée comme les autres polices, famille dédiée `icons` (pas de repli sur le texte). Le crate `egui-phosphor` n'est pas utilisé : sa version 0.14 dépend d'egui 0.36, l'application est en 0.37.
- Boutons en icône (32 × 32 px, libellé et raccourci en infobulle au survol, libellé AccessKit inchangé) : Superposer, Plein écran / Quitter, Palette de commandes, Réglages audio, Panneau, fermeture des réglages audio. Les actions principales (Start, Freeze, Capture, Peak, Insert) gardent un libellé texte : action fréquente, lecture à 1 m.
