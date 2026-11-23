# SoundTuning sous Windows — préparation et procédure de test

SoundTuning est développé sur macOS. La compilation Windows est vérifiée à chaque phase (`cargo check --target x86_64-pc-windows-msvc`) et par la CI GitHub Actions (`windows-latest` : build release + tests). Ce document décrit ce qu'il reste à vérifier sur un vrai PC.

## Prérequis sur le PC de test

1. Windows 10 ou 11, 64 bits.
2. Rust stable (`rustup`, toolchain `stable-x86_64-pc-windows-msvc`).
3. Visual Studio Build Tools (charge de travail « Développement Desktop en C++ ») pour l'éditeur de liens MSVC.
4. Pilote de la carte son (UAX 2-2 ou autre) installé ; pour l'ASIO, le pilote ASIO du constructeur.

## Compilation

```powershell
git clone <dépôt> SoundTuning
cd SoundTuning
cargo build --release
cargo test --workspace
.\target\release\soundtuning.exe devices
.\target\release\soundtuning.exe
```

### ASIO (latence minimale)

L'installeur produit par la CI est compilé **avec** ASIO : les pilotes ASIO installés apparaissent dans la liste des périphériques, suffixés « (ASIO) », à côté des périphériques WASAPI.

Pour compiler soi-même avec ASIO :

1. Installer LLVM (pour `bindgen`) : `winget install LLVM.LLVM`, puis vérifier `clang --version` (au besoin : `$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"`). Sans rien installer sur le système (méthode de la CI, D-063) : `python -m pip install --target target\tools\libclang libclang==18.1.1`, puis `$env:LIBCLANG_PATH = "$PWD\target\tools\libclang\clang\native"`.
2. Compiler avec la feature ; le SDK ASIO de Steinberg est téléchargé automatiquement pendant la compilation (ou, hors ligne, décompressé à la main et désigné par `$env:CPAL_ASIO_DIR = "C:\SDK\asiosdk"`) :

```powershell
cargo build --release -p app --features asio
```

La feature `asio` du crate `app` active `audio-io/asio`, qui active `cpal/asio`. Le binaire intègre du code du SDK Steinberg : vérifier ses conditions de licence avant toute diffusion publique de l'installeur.

### Commandes dans un terminal

En release, `soundtuning.exe` est une application graphique (sans fenêtre de console au lancement, D-059). Depuis PowerShell ou cmd, la sortie de `devices`, `record` et `--help` s'affiche bien dans le terminal, mais l'invite revient avant la fin de la commande. Pour attendre la fin, rediriger la sortie :

```powershell
.\target\release\soundtuning.exe devices | Out-Host
.\target\release\soundtuning.exe record --seconds 10 --output test.wav | Out-Host
```

## Session du 2026-11-23 (PC Windows 11, RTX 5070 Ti, écran 3440 × 1440 à 165 Hz)

Vérifié automatiquement :
- `cargo build --release`, `cargo clippy -D warnings`, `cargo fmt --check` : au vert ; `cargo test --workspace` : 336 tests (les 13 tests de captures restent réservés à macOS).
- Benchs (médianes) : `engine_update` 0,82 ms, `delay_search` 0,91 ms, `plot_frame_prep` 0,087 ms, `optimal_alignment` 2,63 ms, `ui_frame_cpu_frozen` 0,35–0,48 ms : dans les budgets.
- `soundtuning.exe devices` : 11 périphériques WASAPI listés (Realtek, casque, Voicemeeter, NVIDIA HDMI…).
- Mode fichier (`--input-file`, WAV synthétique 48 kHz) : fenêtre prête en 0,47 s ; mesure, délai trouvé.
- Carte branchée : Swissonic UA-2X2 (pilote générique de Windows, sans ASIO). Flux WASAPI réels ouverts avec `record` (rebouclée et interne), fréquence adaptée à 44,1 kHz, même carte reconnue (D-062).
- `soundtuning.exe` n'a plus besoin du Visual C++ Redistributable : runtime intégré (D-062).
- Corrigé pendant la session : fenêtre de console au lancement (D-059), icône et nom du `.exe` (D-059), interface redessinée à 165 images/s et CPU à 20 % (D-060 : plafond 60 fps, option 30 fps, ≈ 8,6 % en mode fichier). Ajouté : vu-mètre dans les boutons de canal (D-061).

## Procédure de test (à cocher sur le PC)

### Démarrage et interface
- [ ] `soundtuning.exe` s'ouvre en moins d'une seconde (build release) ; l'assistant de premier lancement s'affiche.
- [ ] Lancé par double-clic dans l'Explorateur ou depuis le menu Démarrer : aucune fenêtre de console noire ; l'icône « Courbe » apparaît sur le `.exe`, dans la barre des tâches et dans Alt+Tab ; le Gestionnaire des tâches affiche « SoundTuning ».
- [ ] Dans un terminal : `soundtuning.exe devices` affiche la liste (l'invite peut revenir avant, voir « Commandes dans un terminal »).
- [ ] Tools → Frame Rate : 60 fps par défaut ; en 30 fps, les courbes restent fluides au zoom et la charge CPU baisse.
- [ ] Vu-mètre dans les boutons de canal : le bouton Signal Channel sélectionné se remplit selon le niveau du micro, jaune au-dessus de −6 dBFS, rouge (bordure comprise) à l'écrêtage ; idem pour le bouton de Reference Channel sélectionné (OUT ou 1/2).
- [ ] Polices Inter et JetBrains Mono correctes (accents français, symboles ≈ · − γ²).
- [ ] Raccourcis : `Ctrl+K` (palette), `Ctrl+Z` / `Ctrl+Maj+Z`, `Ctrl +` / `Ctrl −` (zoom de l'interface) ; les libellés affichent « Ctrl+K » et non « ⌘K ».
- [ ] Thèmes Nuit / Plein jour (L) ; mise à l'échelle Windows 125 % et 150 % sans flou ni chevauchement.

### Audio WASAPI (par défaut)
- [ ] `soundtuning.exe devices` liste la carte avec ses entrées et sorties.
- [ ] Référence interne : bruit rose sur la sortie choisie, micro sur l'entrée 1, vu-mètres cohérents, délai trouvé et appliqué.
- [ ] Câble sortie → entrée en référence interne : Peak donne la latence de la carte, stable à ±1 échantillon sur 3 redémarrages (pas de calibrage, D-055).
- [ ] Entrée et sortie de la même carte (ex. « Entrée ligne (UX2-2) » et « Haut-parleurs (UX2-2) ») : pas de bandeau « horloges distinctes » (D-056). Avec deux cartes différentes, le bandeau apparaît et « Masquer » le fait disparaître pour cette paire de cartes.
- [ ] Référence rebouclée : mesure sur la musique avec la console sur l'entrée 2.
- [ ] Débrancher la carte pendant la mesure : bandeau « Carte son déconnectée », reprise en un clic après rebranchement.
- [ ] CPU < 10 % d'un cœur en mesure continue (Gestionnaire des tâches, onglet Détails).

### Audio ASIO (installeur de la CI ou build `--features asio`)
- [ ] Le pilote ASIO de la carte apparaît dans « Réglages audio » sous la forme « Nom (ASIO) ».
- [ ] Choisir l'entrée ASIO : la sortie passe sur « Automatique — Nom (ASIO) » ; la mesure démarre.
- [ ] Mesure en référence interne et rebouclée identique au test WASAPI ; délai trouvé par Peak plus faible qu'en WASAPI (latence ASIO plus courte).
- [ ] Taille de tampon : changer le tampon dans le panneau du pilote ASIO, relancer la mesure, aucun craquement.
- [ ] Format 24 bits : si le pilote travaille en 24 bits, les vu-mètres et la mesure restent corrects (niveaux identiques à WASAPI à ±0,5 dB).
- [ ] Pendant une mesure ASIO, ouvrir « Réglages audio » : l'entrée et la sortie ASIO restent sélectionnées.

### Sans carte son
- [ ] Micro intégré du portable et haut-parleurs (ou casque, sortie HDMI, Bluetooth) : sortie « Automatique », la mesure démarre ; si la fréquence demandée n'est pas disponible, une notification indique la fréquence adoptée.

### Fichiers
- [ ] Configuration et session dans `%APPDATA%\SoundTuning\` ; relancer l'application restaure traces, EQ et réglages.
- [ ] Export CSV et export des filtres dans `Documents\SoundTuning\`.
- [ ] `soundtuning.exe record --seconds 10 --output test.wav` puis `soundtuning.exe --input-file test.wav`.

## Packaging

Outil : `cargo-packager` 0.11.8 (choix et justification : `docs/DECISIONS.md`, D-030 et D-035), configuré dans `crates/app/Cargo.toml` (`[package.metadata.packager]`). La CI GitHub Actions (job `package`) produit l'installeur à chaque push et le publie en artefact `soundtuning-windows-latest`.

Sur le PC de test, depuis la racine du dépôt :

```powershell
cargo install cargo-packager --locked --version 0.11.8
cargo build --release -p app --features asio
cd crates\app
cargo packager --release --formats nsis
```

La compilation release précède `cargo packager`, qui empaquette le binaire déjà construit (sans `--features asio`, l'installeur est sans ASIO).

L'installeur est écrit dans `target\packages\soundtuning_<version>_x64-setup.exe` (produit et vérifié le 2026-11-23 : `soundtuning_0.2.3_x64-setup.exe`, 7,3 Mo, sans ASIO). Pour installer `cargo-packager` dans le dépôt plutôt que dans `~\.cargo` : `cargo install cargo-packager --locked --version 0.11.8 --root target\tools`, puis `..\..\target\tools\bin\cargo-packager.exe --release --formats nsis` depuis `crates\app`.

- [ ] L'installeur s'exécute, installe SoundTuning dans le menu Démarrer et le désinstalle proprement.
- [ ] L'application installée démarre, retrouve sa configuration dans `%APPDATA%\SoundTuning\`.

Le binaire et l'installeur ne sont pas signés : Windows SmartScreen affichera un avertissement à la première exécution (« Informations complémentaires » → « Exécuter quand même »).
