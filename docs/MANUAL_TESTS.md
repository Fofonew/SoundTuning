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

## Performance

- [ ] Démarrage jusqu'à la première image utilisable < 1 s (release, Apple Silicon).
- [ ] CPU < 10 % d'un cœur en mesure continue, réglages par défaut (Moniteur d'activité).
- [ ] Courbes fluides, sans saccade au zoom, au survol et en édition d'EQ.

## Visuel

- [ ] Thème Nuit lisible à 1 m en régie sombre, sans éblouissement.
- [ ] Thème Plein jour lisible en extérieur au soleil.
- [ ] Délai en gros chiffres lisible à 2 m.
- [ ] Traces distinguables par une personne daltonienne (ou simulateur de daltonisme).
