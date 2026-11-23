# Reprise sur PC Windows

Fichier de passation pour une session Claude Code lancée sur le PC Windows de l'utilisateur. Une partie des retours du 2026-11-23 ne peut être ni codée ni testée depuis macOS : elle est décrite ici.

## Avant de commencer

1. Lire `CLAUDE.md` (règles non négociables, conventions, section « État »), puis `docs/SPEC.md`, puis ce fichier en entier.
2. Prérequis et compilation : `docs/WINDOWS.md` (Rust MSVC, LLVM pour ASIO, `cargo build --release -p app --features asio`).
3. Les mêmes règles s'appliquent :
   - pas d'`unwrap` ni d'`expect` hors des tests, pas d'`unsafe` dans nos crates ;
   - tous les textes de l'interface dans `crates/app/src/text.rs` ;
   - choix non trivial → entrée dans `docs/DECISIONS.md` ;
   - skill `verify` avant chaque commit ;
   - commits locaux « suite : … », sans ligne d'attribution ;
   - **ne rien pousser sans l'accord de l'utilisateur**.
4. Une étape demande-t-elle une décision de l'utilisateur ? Exposer les options et lui demander, sans trancher seul.

## Retours de l'utilisateur (2026-11-23), cités tels quels

> au niveaux des configs de peripherique, avec ma carte son, j'avais juste entre ligne (ux2-2) alors que sur weindows j'ai chaque entre et sortie en deux fois ou trois, avec (wave) (dsound) (asio), pareil quand on choisie asio, les autres soft propose un bouton pour ouvrir le asio control panel, la rien ne marche, pense aussi aux soft qui propose en exclusive mode, et quand je tester avec mon in et out sur le meme periph j'avais un warning comme quoi la clock interne etait pas sync car periph different alors que cetait le meme, et aussi il fait pouvoir masquer cette popup
>
> ensuite le peak delais est pas bon, sur systune il est toujours en partant de zero […] je veux comme systune, aussi on peux pas regler facilement le volume de sortie, par default il doit etre comme systune a -3 ou -6 ou meme 0 […] et rajouter aussi des jauges dynamique pour voir le signal en input si on sature ou pas

Carte son de l'utilisateur : Tascam UX2 (nom Windows « UX2-2 »), pilotes WASAPI et ASIO du constructeur.

## Déjà fait depuis macOS (à vérifier sur le PC)

| Retour | Ce qui a été fait | À vérifier sous Windows |
|---|---|---|
| Peak doit partir de zéro comme SysTune | D-055 : plus de calibrage, délai total affiché | Câble sortie → entrée : Peak donne la latence de la carte |
| Faux avertissement « horloges distinctes » avec entrée et sortie sur la même carte | D-056 : `audio_io::shares_hardware` compare le nom entre parenthèses (« (UX2-2) ») | Voir tâche 1 |
| Pouvoir masquer ce bandeau | D-056 : bouton « Masquer », mémorisé pour la paire entrée / sortie | Avec deux cartes différentes |
| Volume de sortie difficile à régler, défaut −3 / −6 / 0 | D-057 : curseur de −60 à 0 dBFS, −6 par défaut | Réglage pendant la mesure |
| Jauges d'entrée pour régler le gain | D-058 : jauges Signal et Reference dans Input, CLIP tenu 2 s | Saturer l'entrée de l'UX2 |
| Pilote affiché pour chaque périphérique | Hors CoreAudio, chaque entrée affiche « Nom (WASAPI) » ou « Nom (ASIO) » | Voir tâche 1 |

## Tâches à faire sur le PC

### ~~1. Liste des périphériques et détection « même carte »~~ — fait le 2026-11-23 (D-062)

La carte de l'utilisateur est une **Swissonic UA-2X2** (pas une Tascam), sur le pilote générique de Windows, sans pilote ASIO. Noms WASAPI : « Ligne (UA-2X2) » (entrée, 44,1 kHz seulement) et « Haut-parleurs (UA-2X2) » (sortie). Même carte reconnue (`uses_separate_clocks=false`), test unitaire ajouté. Seuls pilotes ASIO présents sur le PC : Ableton Move et Ableton Push. La tâche 2 demande donc d'abord un pilote ASIO pour la carte (voir D-062).

Texte d'origine de la tâche 1 :

- Lancer `soundtuning devices` (ou `cargo run --release -p app --features asio -- devices`) et copier la sortie : identifiants, noms, pilotes, canaux.
- Vérifier que l'UX2 apparaît une fois en WASAPI (entrée et sortie, deux points d'accès) et une fois en ASIO.
- Référence interne avec l'entrée et la sortie WASAPI de l'UX2 : le bandeau « horloges distinctes » ne doit pas apparaître. S'il apparaît, les noms ne suivent pas le motif « … (UX2-2) ». Adapter alors `shares_hardware` (`crates/audio-io/src/devices.rs`) avec les vrais noms, et ajouter un test unitaire avec ces noms.
- MME (« WAVE ») et DirectSound : `cpal` 0.18.2 ne les propose pas (seulement WASAPI et ASIO sous Windows). WASAPI les remplace (même matériel, latence plus faible). Ne pas les ajouter sans l'accord de l'utilisateur ; l'expliquer s'il le redemande.

### 2. « Quand on choisit ASIO, rien ne marche »

Cause inconnue : à diagnostiquer avant toute correction.

- Lancer l'application depuis un terminal (les logs `tracing` sortent sur la console), choisir l'entrée UX2 (ASIO), puis Appliquer. Copier les erreurs et le bandeau affiché.
- Pistes, à vérifier une à une :
  - **Même pilote** : avec ASIO, l'entrée et la sortie doivent venir du même pilote. `select_input` (`crates/app/src/ui/audio_panel.rs`) vide une sortie d'un autre pilote ; vérifier que la sortie passe bien sur « Automatique — UX2 (ASIO) ».
  - **Fréquence** : un pilote ASIO n'a qu'une fréquence active. Vérifier `negotiate_sample_rate` et `set_sample_rate` (côté `cpal`), et la fréquence réglée dans le panneau Tascam.
  - **Format** : le format d'échantillon ASIO de l'UX2 (souvent I32 ou I24) doit être accepté par `sample_format_preference` (`crates/audio-io/src/devices.rs`).
  - **Tampon** : avec `buffer_frames` différent de la taille imposée par le pilote, l'ouverture peut échouer. Essayer « Automatique ».
  - **Pilote occupé** : un seul programme peut ouvrir un pilote ASIO à la fois (fermer SysTune, le DAW, etc.).
- Corriger, ajouter un test quand c'est possible sans matériel (logique de choix pure), et noter la cause dans `docs/DECISIONS.md`.

### 3. Bouton « ASIO Control Panel »

- Contrainte technique : `asio-sys` 0.4.0, utilisé par `cpal` 0.18.2, n'expose pas `ASIOControlPanel()`. Ses bindings bindgen ne l'ont pas dans leur liste autorisée (`allowlist_function`, voir son `build.rs`). Pas d'appel possible sans code `unsafe` (FFI), interdit dans nos crates.
- Options à présenter à l'utilisateur avant de coder :
  - a. **Correctif local d'`asio-sys`** via `[patch.crates-io]` : copie de la crate dans `vendor/asio-sys` ; ajout de `.allowlist_function("ASIOControlPanel")` et d'une méthode sûre `Driver::open_control_panel()`. L'`unsafe` resterait dans la crate tierce corrigée, pas dans nos crates. Coût : maintenance à chaque mise à jour de `cpal`. Côté application : bouton « ASIO Control Panel » dans les réglages audio, visible si le périphérique choisi est ASIO ; à l'appel, `asio_sys::Asio::new().loaded_driver()` ou `load_driver(nom)`.
  - b. **Contribution amont** : demander ou proposer l'ajout dans `cpal`/`asio-sys`, puis attendre une version publiée.
  - c. **Sans code** : expliquer d'ouvrir le panneau du constructeur (« US-2x2 & US-4x4 Settings Panel » pour l'UX2). Ajouter une aide dans les réglages audio.
- Tester avec l'UX2 : le panneau s'ouvre, la taille de tampon changée y est reprise au redémarrage de la mesure.

### 4. Mode exclusif WASAPI

- `cpal` 0.18.2 n'ouvre WASAPI qu'en mode partagé : aucune option exclusive (vérifié dans `src/host/wasapi`).
- Options à présenter à l'utilisateur :
  - a. Vérifier si une version plus récente de `cpal` ajoute le mode exclusif (`cargo search cpal`, changelog). Si oui : mettre à jour par `cargo add`, puis ajouter l'option « Mode exclusif » dans les réglages audio (Windows seulement).
  - b. Correctif local de `cpal` (`AUDCLNT_SHAREMODE_EXCLUSIVE`) : lourd et risqué.
  - c. Ne pas le faire : ASIO donne déjà la latence minimale et l'accès direct.
- Si c'est implémenté : tester la latence (Peak sur câble), l'absence de craquements et le comportement quand un autre programme tient déjà la carte.

### ~~5. Icône de `soundtuning.exe` (reste de D-054)~~ — fait le 2026-11-23 (D-059)

Fait dans la première session Windows : `crates/app/build.rs` avec `winresource` (hôte et cible Windows seulement), icône et métadonnées « SoundTuning » vérifiées sur le `.exe`. Reste à voir : l'icône dans le menu Démarrer après installation, et la CI Windows.

Même session (D-059 à D-061) : plus de fenêtre de console au lancement, affichage plafonné à 60 fps (option 30 fps), vu-mètre dans les boutons de canal. La Tascam UX2 n'était pas branchée (absente de `soundtuning.exe devices`) : les tâches 1 à 4 restent à faire avec la carte. ASIO pas encore compilé sur ce PC (LLVM absent).

Texte d'origine de la tâche 5 :

- L'exe et les raccourcis du menu Démarrer gardent l'icône par défaut. Ajouter `winresource` en dépendance de build (`cargo add --build winresource -p app`). Écrire un `build.rs` qui n'agit que si l'hôte et la cible sont Windows (`cfg!(windows)` et `CARGO_CFG_TARGET_OS`), pour ne pas casser `cargo check --target x86_64-pc-windows-msvc` depuis macOS. Icône : `crates/app/assets/icon/SoundTuning.ico`.
- Vérifier l'icône dans l'Explorateur, dans le menu Démarrer après installation, et que la CI Windows passe.

## Fin de session

- Mettre à jour la section « État » de `CLAUDE.md` et ce fichier : barrer ce qui est fait, noter ce qui reste.
- Cocher ce qui a été testé dans `docs/WINDOWS.md` et `docs/MANUAL_TESTS.md`.
- Lancer le skill `verify`, faire un commit local, puis demander à l'utilisateur s'il veut pousser ou publier une version (`scripts/release.sh`, `docs/RELEASE.md`).
