# Publier une version

Pré-requis : dépôt GitHub public (sinon le logiciel installé ne voit pas les releases, D-042), branche `main` à jour et CI au vert.

1. Choisir le numéro `X.Y.Z` (supérieur à la version en cours, sinon personne ne se verra proposer la mise à jour).
2. Lancer :
   ```bash
   scripts/release.sh X.Y.Z
   ```
   Le script met la version à jour dans `Cargo.toml`, crée le commit « version X.Y.Z » et le tag `vX.Y.Z` en local. Il ne pousse rien.
3. Publier :
   ```bash
   git push origin main && git push origin vX.Y.Z
   ```
4. Le workflow « Release » (onglet Actions) vérifie le numéro, relance toute la CI, puis crée la release avec :
   - `SoundTuning-X.Y.Z-macos.zip` (application macOS) ;
   - `SoundTuning-X.Y.Z-windows-x64-setup.exe` (installeur Windows, avec ASIO).
5. Au lancement suivant, chaque SoundTuning connecté à internet affiche « SoundTuning X.Y.Z est disponible » avec un bouton « Télécharger ». Hors connexion, rien ne s'affiche et le logiciel fonctionne normalement.

Installer la mise à jour : Windows, lancer le nouvel installeur (il remplace l'ancienne version) ; macOS, remplacer `SoundTuning.app` dans Applications. Les réglages et la session sont conservés (dossier de configuration de l'utilisateur).

En cas d'échec du workflow : supprimer le tag (`git push --delete origin vX.Y.Z` puis `git tag -d vX.Y.Z`), corriger, recommencer avec le même numéro.
