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
