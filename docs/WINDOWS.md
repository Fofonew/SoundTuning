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

### Variante ASIO (latence minimale)

ASIO n'est pas compilé par défaut : il dépend du SDK ASIO de Steinberg, non redistribuable.

1. Télécharger le SDK ASIO depuis le site développeur de Steinberg (compte gratuit) et le décompresser, par exemple dans `C:\SDK\asiosdk`.
2. Installer LLVM (pour `bindgen`) : `winget install LLVM.LLVM`, puis vérifier `clang --version`.
3. Déclarer le chemin du SDK puis compiler avec la feature :

```powershell
$env:CPAL_ASIO_DIR = "C:\SDK\asiosdk"
cargo build --release --features asio
```

La feature `asio` du crate `app` active `audio-io/asio`, qui active `cpal/asio`. Sans le SDK, la compilation avec `--features asio` échoue : c'est attendu.

## Procédure de test (à cocher sur le PC)

### Démarrage et interface
- [ ] `soundtuning.exe` s'ouvre en moins d'une seconde (build release) ; l'assistant de premier lancement s'affiche.
- [ ] Polices Inter et JetBrains Mono correctes (accents français, symboles ≈ · − γ²).
- [ ] Raccourcis : `Ctrl+K` (palette), `Ctrl+Z` / `Ctrl+Maj+Z`, `Ctrl +` / `Ctrl −` (zoom de l'interface) ; les libellés affichent « Ctrl+K » et non « ⌘K ».
- [ ] Thèmes Nuit / Plein jour (L) ; mise à l'échelle Windows 125 % et 150 % sans flou ni chevauchement.

### Audio WASAPI (par défaut)
- [ ] `soundtuning.exe devices` liste la carte avec ses entrées et sorties.
- [ ] Référence interne : bruit rose sur la sortie choisie, micro sur l'entrée 1, vu-mètres cohérents, délai trouvé et appliqué.
- [ ] Calibrage de latence (câble sortie → entrée) : valeur stable à ±1 échantillon sur 3 redémarrages.
- [ ] Référence rebouclée : mesure sur la musique avec la console sur l'entrée 2.
- [ ] Débrancher la carte pendant la mesure : bandeau « Carte son déconnectée », reprise en un clic après rebranchement.
- [ ] CPU < 10 % d'un cœur en mesure continue (Gestionnaire des tâches, onglet Détails).

### Audio ASIO (build `--features asio`)
- [ ] Le pilote ASIO apparaît dans la liste des périphériques.
- [ ] Mesure en référence interne et rebouclée identique au test WASAPI ; latence calibrée plus faible.

### Fichiers
- [ ] Configuration et session dans `%APPDATA%\SoundTuning\` ; relancer l'application restaure traces, EQ et réglages.
- [ ] Export CSV et export des filtres dans `Documents\SoundTuning\`.
- [ ] `soundtuning.exe record --seconds 10 --output test.wav` puis `soundtuning.exe --input-file test.wav`.

## Packaging

Outil : `cargo-packager` 0.11.8 (choix et justification : `docs/DECISIONS.md`, D-030 et D-035), configuré dans `crates/app/Cargo.toml` (`[package.metadata.packager]`). La CI GitHub Actions (job `package`) produit l'installeur à chaque push et le publie en artefact `soundtuning-windows-latest`.

Sur le PC de test, depuis la racine du dépôt :

```powershell
cargo install cargo-packager --locked
cd crates\app
cargo packager --release --formats nsis
```

L'installeur est écrit dans `target\packages\soundtuning_0.1.0_x64-setup.exe`.

- [ ] L'installeur s'exécute, installe SoundTuning dans le menu Démarrer et le désinstalle proprement.
- [ ] L'application installée démarre, retrouve sa configuration dans `%APPDATA%\SoundTuning\`.

Le binaire et l'installeur ne sont pas signés : Windows SmartScreen affichera un avertissement à la première exécution (« Informations complémentaires » → « Exécuter quand même »).
