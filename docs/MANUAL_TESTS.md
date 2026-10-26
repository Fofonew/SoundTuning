# Tests manuels

Ce que les tests automatisés ne peuvent pas couvrir : matériel réel, perception visuelle, performance ressentie.
Cocher, dater et noter les observations. Matériel de référence : UAX 2-2, micro de mesure sur l'entrée 1.

## Matériel et audio

- [ ] La UAX 2-2 apparaît dans la liste des périphériques avec ses canaux d'entrée et de sortie (noter le nombre réel).
- [ ] Première ouverture sur macOS : la demande d'autorisation micro s'affiche ; si elle est refusée, l'alerte « autorisation micro manquante » explique où la rétablir.
- [ ] Vu-mètres référence et mesure réagissent, crête et témoin d'écrêtage corrects.
- [ ] Générateur bruit rose sur la sortie gauche, puis sur la sortie droite, niveau réglable, sans clic au démarrage ni à l'arrêt.
- [ ] Référence interne : calibrage de latence avec un câble sortie → entrée ; noter la latence mesurée.
- [ ] Référence interne : la latence calibrée reste identique (à ±1 échantillon) après arrêt/redémarrage du flux et après redémarrage de l'application.
- [ ] Référence interne, micro à distance connue (ex. 3,43 m) : distance affichée cohérente (±5 cm).
- [ ] Référence rebouclée : sortie console sur une entrée, mesure sur la musique, cohérence et délai stables.
- [ ] Référence interne avec deux périphériques distincts (micro du Mac + sortie UAX) : le délai mesuré reste stable sur 5 min (dérive d'horloge éventuelle) ; noter la dérive observée.
- [ ] Débrancher la carte pendant la mesure : alerte non bloquante ; la rebrancher : reprise en un clic.
- [ ] `soundtuning devices` liste la UAX 2-2 avec son identifiant `coreaudio:…`.
- [ ] Enregistrement CLI de 10 s, référence interne : `soundtuning record --output test.wav --device <id UAX> --generator-channel 1` ; le bruit rose sort sur la sortie 1 et le WAV contient le générateur (canal 1) et le micro (canal 2), avec le micro en retard de la latence de la carte + distance.
- [ ] Enregistrement CLI, référence rebouclée : `--loopback-channel 2` ; le WAV contient l'entrée 2 (référence) et l'entrée 1 (micro).
- [ ] `soundtuning --input-file test.wav` : la fenêtre s'ouvre et les vu-mètres bougent en boucle.
- [ ] Fenêtre minimale (phase 0) : Démarrer / Arrêter ouvre et ferme les flux sans erreur, vu-mètres référence (générateur) et mesure (micro) cohérents.

- [ ] Bouton « Audio » (⌘,) : changer d'entrée, de sortie et de canaux, « Appliquer les réglages » : la mesure repart sur la nouvelle carte, les vu-mètres suivent ; les réglages sont retrouvés au lancement suivant.
- [ ] Sans carte son : entrée « Micro MacBook », sortie « Automatique — Haut-parleurs MacBook » : la mesure démarre, le bruit rose sort des haut-parleurs. Idem avec un casque, des AirPods (sortie seule) ou un écran HDMI en sortie.
- [ ] Sortie à 44,1 kHz seulement (forcer le format dans « Configuration audio et MIDI ») avec la fréquence réglée sur 48 kHz : la mesure démarre à 44,1 kHz et une notification « Fréquence adaptée à 44,1 kHz » s'affiche.
- [ ] Entrée et sortie sans fréquence commune : message en français listant les fréquences de chaque côté, sans plantage.

## Traces et sessions

- [ ] S capture une trace nommée « Trace N — HH:MM » à l'heure locale ; double-clic sur le nom pour renommer ; Cmd+Z / Maj+Cmd+Z.
- [ ] Quitter l'application (ou la forcer à quitter) puis relancer : les traces sont restaurées.
- [ ] Export CSV ouvert dans un tableur (séparateur « ; ») : fréquences, niveaux et cohérence par trace.

- [ ] F2 renomme la trace sélectionnée, Échap annule ; la case « Sél. » sélectionne sans renommer.
- [ ] Session : rendre `session.json` illisible (texte quelconque) puis relancer : notification « Session illisible mise de côté… », fichier `session.corrupt-….json` présent, dernière sauvegarde valide restaurée.

## Délais et alignement

- [ ] Au démarrage de la mesure, le délai est trouvé et appliqué sans intervention (référence interne calibrée et rebouclée).
- [ ] Déplacer le micro d'environ 1 m pendant la mesure : notification « Délai changé : … — Appliquer » ; A applique le nouveau délai ; la notification ne se répète pas tant que le micro reste à la nouvelle place.
- [ ] Touche T (ou bouton « Suivi ») : en suivi, le délai suit le micro sans notification ; désactivé, il reste figé.
- [ ] ⇧← / ⇧→ et boutons − / + : le délai affiché varie de 0,01 ms par appui, la courbe de phase/le niveau des aigus réagit.
- [ ] Alignement tête / sub réel : capturer chaque source seule, choisir A et B, bande 60–150 Hz à la souris (Maj + glisser) : la valeur proposée, recopiée dans le processeur, donne une somme mesurée conforme à la somme prédite (±1 dB dans la bande).

## EQ virtuel

- [ ] Onglet EQ : double-clic sur le tracé ajoute une cloche à l'endroit cliqué ; glisser une poignée déplace fréquence et gain ; molette sur une poignée change le Q ; aucune saccade des courbes pendant l'édition.
- [ ] Recopier les filtres exportés (texte) dans un processeur réel : la mesure après correction suit le « Résultat prédit » (±1 dB hors zones de faible cohérence).
- [ ] Banques A/B, bypass global et par filtre : bascule instantanée de la courbe prédite.
- [ ] Auto EQ sur une trace réelle (salle) : les PEQ proposés respectent max boost / max cut / max filtres ; recopiés dans le processeur, la mesure suit le résultat prédit ; comparer avec l'auto EQ de REW sur la même mesure.
- [ ] Saisie au clavier dans les champs (ex. « 1,2k », « -4,5 ») : la valeur est prise en compte.

## Vues Phase, IR et définition des traces

- [ ] H : vue Phase ; avec le délai appliqué, la phase d'une enceinte seule reste proche de 0° dans le médium et sans traits verticaux aux repliements ±180°.
- [ ] I : vue IR ; pic à 0 ms après « Appliquer » (A) ; en déplaçant le micro de 1 m, le pic se décale d'environ 2,9 ms (lecture du réticule : temps et distance) si le délai n'est pas ré-appliqué.
- [ ] Échelle « dB » de l'IR : décroissance lisible jusqu'à −60 dB dans une salle.
- [ ] V : Phase au-dessus de la magnitude ; zoom ou plage (1–4) dans un tracé → l'autre suit.
- [ ] Maj+F : plein écran, panneau et barre inférieure masqués ; Échap ou le bouton vert de macOS en sortent et tout réapparaît.
- [ ] Capturer une trace en 1/48, la passer en 1/6 (ligne « Définition ») : identique à une mesure live en 1/6 sur le même signal (comparer avec le lissage de la barre inférieure en 1/6) ; Cmd+Z revient à 1/48.
- [ ] Session de 20 traces : la sauvegarde automatique reste invisible (pas d'à-coup du tracé).

## Delay Analysis (alignement façon SysTune)

- [ ] Sub et tête captés séparément (S), onglet Alignment : Reference = tête, Delay = sub ; Pos, Neg et Flat s'affichent avec leurs courbes de sommation ; la plus forte de Pos / Neg est sélectionnée.
- [ ] Recopier la solution dans le processeur (délai et polarité indiqués sous « Réglage à recopier »), mesurer les deux sources ensemble : la mesure suit la courbe prédite dans la plage (± 1 dB).
- [ ] Custom : saisir un délai et inverser la polarité ; la courbe Custom suit immédiatement.
- [ ] Range : Low, Mid, Full, Custom (Crossover en Hz + largeur) ; Maj + glisser sur le tracé passe en Custom.
- [ ] Les boutons en icône (Superposer, Plein écran, Commandes, Audio, Panneau) affichent leur libellé au survol.

## Mises à jour et petites fenêtres (Windows)

- [ ] Hors connexion (Wi-Fi coupé) : SoundTuning démarre normalement, aucun message ; palette › « Rechercher les mises à jour » affiche « Mises à jour injoignables (hors connexion ?) ».
- [ ] Connecté, après publication d'une version plus récente (`docs/RELEASE.md`) : le bandeau « SoundTuning X.Y.Z est disponible » s'affiche ; « Télécharger » ouvre la page de la release ; « Plus tard » le masque.
- [ ] Connecté, version à jour : rien au lancement ; recherche manuelle : « SoundTuning X.Y.Z est à jour. ».
- [ ] Derrière un proxy d'entreprise : même comportement (sinon noter le message du journal).
- [ ] Windows à 125 % et 150 % d'échelle : au premier lancement, la fenêtre tient dans l'écran (agrandie si besoin) ; barres haute et basse sans chevauchement ; « Audio » : boutons « Appliquer les réglages » et « Actualiser » visibles, champs défilants.
- [ ] Windows : réduire la fenêtre (bouton « Réduire ») pendant une mesure, attendre 30 s, la restaurer : l'affichage reprend à la bonne taille, sans gel ni zone noire. Si le problème persiste : relancer avec `set WGPU_BACKEND=gl` puis `soundtuning.exe` et noter le résultat.
- [ ] Redimensionner jusqu'à la taille minimale (1024 × 600 points) : aucun chevauchement.

## Packaging

- [ ] Télécharger l'artefact CI `soundtuning-macos-latest` (`SoundTuning-macos.zip`), décompresser : l'application n'est pas signée, macOS la met en quarantaine (« endommagée ») ; l'ouvrir par clic droit → Ouvrir, ou `xattr -cr SoundTuning.app`. Au premier démarrage de la mesure, macOS demande l'accès au micro avec le texte de `NSMicrophoneUsageDescription`.
- [ ] Windows : installeur `soundtuning_0.1.0_x64-setup.exe` (artefact `soundtuning-windows-latest`, compilé avec ASIO), voir `docs/WINDOWS.md`.

## Performance

- [ ] Démarrage jusqu'à la première image utilisable < 1 s (release, Apple Silicon).
- [ ] CPU < 10 % d'un cœur en mesure continue, réglages par défaut (Moniteur d'activité).
- [ ] Courbes fluides, sans saccade au zoom, au survol et en édition d'EQ.

## Visuel

- [ ] Première ouverture : l'assistant s'affiche (carte son → référence et canaux avec vu-mètres → test), puis ne réapparaît plus au lancement suivant.
- [ ] Tracé : zoom molette (fréquence), Alt + molette (dB), pincement sur le pavé tactile, glisser pour déplacer, double-clic pour réinitialiser ; aucune saccade.
- [ ] Cmd + / Cmd − : l'interface s'agrandit / se réduit proprement.
- [ ] Navigation au clavier (Tab) : chaque bouton reçoit un contour de focus visible ; VoiceOver annonce les boutons et vu-mètres.

- [ ] Thème Nuit lisible à 1 m en régie sombre, sans éblouissement.
- [ ] Thème Plein jour lisible en extérieur au soleil.
- [ ] Délai en gros chiffres lisible à 2 m.
- [ ] Traces distinguables par une personne daltonienne (ou simulateur de daltonisme).
