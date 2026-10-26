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
- [ ] Débrancher la carte pendant la mesure : alerte non bloquante ; la rebrancher : reprise en un clic.
- [ ] Enregistrement CLI de 10 s : le WAV contient bien référence et mesure.

## Performance

- [ ] Démarrage jusqu'à la première image utilisable < 1 s (release, Apple Silicon).
- [ ] CPU < 10 % d'un cœur en mesure continue, réglages par défaut (Moniteur d'activité).
- [ ] Courbes fluides, sans saccade au zoom, au survol et en édition d'EQ.

## Visuel

- [ ] Thème Nuit lisible à 1 m en régie sombre, sans éblouissement.
- [ ] Thème Plein jour lisible en extérieur au soleil.
- [ ] Délai en gros chiffres lisible à 2 m.
- [ ] Traces distinguables par une personne daltonienne (ou simulateur de daltonisme).
