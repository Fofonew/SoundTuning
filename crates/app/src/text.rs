pub const APP_NAME: &str = "SoundTuning";

pub const CLI_ABOUT: &str =
    "Mesure acoustique en double FFT pour le calage de systèmes de sonorisation.";
pub const CLI_INPUT_FILE_HELP: &str =
    "Fichier WAV 2 canaux (référence, mesure) utilisé à la place de la carte son.";
pub const CLI_DEVICES_ABOUT: &str = "Liste les périphériques audio et leurs canaux.";
pub const CLI_RECORD_ABOUT: &str =
    "Enregistre référence et mesure dans un fichier WAV 2 canaux (10 s par défaut).";
pub const CLI_SECONDS_HELP: &str = "Durée de l'enregistrement en secondes.";
pub const CLI_OUTPUT_HELP: &str = "Fichier WAV de sortie.";
pub const CLI_DEVICE_HELP: &str = "Identifiant du périphérique d'entrée (voir « devices »).";
pub const CLI_OUTPUT_DEVICE_HELP: &str = "Identifiant du périphérique de sortie du générateur (par défaut : celui d'entrée s'il a des sorties, sinon la sortie du système).";
pub const CLI_SAMPLE_RATE_HELP: &str = "Fréquence d'échantillonnage en Hz.";
pub const CLI_MEASUREMENT_CHANNEL_HELP: &str = "Entrée du micro de mesure (à partir de 1).";
pub const CLI_LOOPBACK_CHANNEL_HELP: &str =
    "Entrée de la référence rebouclée (à partir de 1). Absente : référence interne (générateur).";
pub const CLI_GENERATOR_CHANNEL_HELP: &str = "Sortie du générateur de bruit rose (à partir de 1).";
pub const CLI_LEVEL_HELP: &str = "Niveau du générateur en dBFS.";

pub const DEVICES_NONE: &str = "Aucun périphérique audio trouvé.";
pub const DEFAULT_INPUT_MARK: &str = "entrée par défaut";
pub const DEFAULT_OUTPUT_MARK: &str = "sortie par défaut";

pub const KEY_SPACE: &str = "Espace";
pub const KEY_CTRL: &str = "Ctrl";
pub const KEY_SHIFT: &str = "Maj";

pub const ACTION_TOGGLE_MEASUREMENT: &str = "Start / Stop Analysis (mesure continue)";
pub const ACTION_FIND_DELAY: &str = "Peak : trouver le délai et le reporter dans Delay Offset";
pub const ACTION_APPLY_DELAY: &str = "Reporter le dernier délai trouvé dans Delay Offset";
pub const ACTION_CAPTURE_TRACE: &str = "Capture Measurement : mémoriser une trace (overlay)";
pub const ACTION_TOGGLE_FREEZE: &str = "Freeze / Unfreeze (figer l'affichage)";
pub const ACTION_RANGE_FULL: &str = "Plage : plein spectre";
pub const ACTION_RANGE_SUB: &str = "Plage : sub (20–200 Hz)";
pub const ACTION_RANGE_MID: &str = "Plage : médium (150 Hz–4 kHz)";
pub const ACTION_RANGE_HIGH: &str = "Plage : aigu (2–20 kHz)";
pub const ACTION_RESET_VIEW: &str = "Réinitialiser le zoom";
pub const ACTION_TOGGLE_THEME: &str = "Thème Nuit / Plein jour";
pub const ACTION_OPEN_PALETTE: &str = "Palette de commandes";
pub const ACTION_UNDO: &str = "Annuler";
pub const ACTION_REDO: &str = "Rétablir";
pub const ACTION_TOGGLE_SIDE_PANEL: &str = "Afficher / masquer le panneau Measure";
pub const ACTION_TOGGLE_GENERATOR: &str = "Play Signal (pink noise) on / off";
pub const ACTION_CALIBRATE_LATENCY: &str = "Calibrate : calibrer la latence de la carte";
pub const ACTION_ADD_EQ_FILTER: &str = "Ajouter un filtre d'EQ";
pub const ACTION_EXPORT_SESSION: &str = "Exporter (CSV, filtres)";
pub const ACTION_RESTART_MEASUREMENT: &str = "Relancer la mesure";

pub const BUTTON_START: &str = "Start";
pub const BUTTON_STOP: &str = "Stop";
pub const BUTTON_CAPTURE: &str = "Capture";
pub const BUTTON_FREEZE: &str = "Freeze";
pub const BUTTON_UNFREEZE: &str = "Unfreeze";
pub const BUTTON_FIND_DELAY: &str = "Peak";
pub const BUTTON_APPLY_DELAY: &str = "Insert";
pub const BUTTON_PALETTE: &str = "Commands";
pub const BUTTON_AUDIO_SETTINGS: &str = "Tools";
pub const ACTION_OPEN_AUDIO_SETTINGS: &str = "Réglages audio (entrée, sortie, canaux)";
pub const AUDIO_SETTINGS_TITLE: &str = "Réglages audio";
pub const AUDIO_SETTINGS_CLOSE: &str = "Fermer les réglages audio";
pub const AUDIO_INPUT_DEVICE: &str = "Entrée (micro et référence)";
pub const AUDIO_OUTPUT_DEVICE: &str = "Sortie (pink noise)";
pub const AUDIO_SAMPLE_RATE: &str = "Sample rate";
pub const AUDIO_BUFFER: &str = "Buffer";
pub const AUDIO_BUFFER_AUTO: &str = "Automatique";
pub const AUDIO_REFERENCE_MODE: &str = "Reference";
pub const AUDIO_REFERENCE_INTERNAL: &str = "Internal";
pub const AUDIO_REFERENCE_LOOPBACK: &str = "Loopback";
pub const AUDIO_SYSTEM_INPUT: &str = "Entrée par défaut du système";
pub const AUDIO_AUTO_OUTPUT: &str = "Automatique";
pub const AUDIO_NO_COMMON_RATE: &str = "Aucune fréquence commune à cette entrée et à cette sortie : choisissez une autre sortie ou la référence rebouclée.";
pub const AUDIO_NO_OUTPUT: &str = "Choisissez une sortie pour envoyer le bruit rose.";
pub const AUDIO_APPLY: &str = "Appliquer les réglages";
pub const AUDIO_APPLY_HINT: &str =
    "Appliquer enregistre les réglages et relance la mesure si elle tourne.";
pub const AUDIO_FILE_MODE_NOTE: &str =
    "Mode fichier : ces réglages serviront au prochain lancement sur la carte son.";
pub const NOTIFY_AUDIO_APPLIED: &str = "Réglages audio appliqués";
pub const BUTTON_RETRY: &str = "Réessayer";
pub const BUTTON_RESUME: &str = "Reprendre";
pub const BUTTON_CALIBRATE: &str = "Calibrer";
pub const BUTTON_DISMISS: &str = "Fermer";
pub const BUTTON_NEXT: &str = "Suivant";
pub const BUTTON_BACK: &str = "Retour";
pub const BUTTON_FINISH_WIZARD: &str = "Start";
pub const BUTTON_SKIP_WIZARD: &str = "Passer";
pub const BUTTON_TOGGLE_PANEL: &str = "Measure";

pub const TAB_MEASURE: &str = "Measure";
pub const TAB_TOOLS: &str = "Tools";
pub const SECTION_INPUT: &str = "Input";
pub const SECTION_OUTPUT: &str = "Output";
pub const SECTION_PARAMETERS: &str = "Parameters";
pub const SIGNAL_CHANNEL: &str = "Signal Channel";
pub const REFERENCE_CHANNEL: &str = "Reference Channel";
pub const REFERENCE_CHANNEL_OUT: &str = "OUT";
pub const REFERENCE_CHANNEL_OUT_HINT: &str =
    "OUT : référence interne (le signal joué par la sortie)";
pub const CHANNEL_FILE_MODE: &str = "Mode fichier : canaux fixés par le fichier WAV";
pub const DELAY_OFFSET: &str = "Delay Offset [ms]";
pub const DELAY_OFFSET_HINT: &str =
    "Délai appliqué à la référence. Saisie, molette ou glisser ; Peak le trouve, Auto le suit.";
pub const BUTTON_AUTO: &str = "Auto";
pub const BUTTON_CALIBRATE_LATENCY: &str = "Calibrate";
pub const LATENCY_NOT_CALIBRATED: &str = "Latence non calibrée";
pub const LATENCY_NOT_NEEDED: &str = "Référence rebouclée : pas de calibrage";
pub const BUTTON_START_ANALYSIS: &str = "Start Analysis";
pub const BUTTON_STOP_ANALYSIS: &str = "Stop Analysis";
pub const BUTTON_PLAY_SIGNAL: &str = "Play Signal";
pub const OUTPUT_SIGNAL: &str = "Signal";
pub const OUTPUT_PINK_NOISE: &str = "Pink Noise";
pub const OUTPUT_LEVEL_LABEL: &str = "Niveau du pink noise (dBFS)";
pub const PLAY_CHANNEL: &str = "Play Channel";
pub const FFT_SIZE: &str = "FFT Size";
pub const AVERAGES: &str = "Averages";
pub const BUTTON_RESET_AVERAGES: &str = "Reset";
pub const RESET_AVERAGES_HINT: &str = "Reset : repartir d'une moyenne vide";
pub const BUTTON_CAPTURE_MEASUREMENT: &str = "Capture Measurement";
pub const TAB_OVERLAYS: &str = "Overlays";
pub const TAB_LEVELS: &str = "Levels";
pub const BUTTON_REMOVE: &str = "Remove";
pub const REMOVE_HINT: &str = "Remove : supprimer les overlays sélectionnés";
pub const AVERAGE_HINT: &str = "Average : moyenne des overlays sélectionnés (au moins deux)";
pub const SAVE_HINT: &str = "Save : enregistrer les overlays en CSV";
pub const OVERLAY_PROPERTIES: &str = "Propriétés";
pub const OVERLAY_PROPERTIES_GLYPH: &str = "···";
pub const OVERLAY_SELECT_HINT: &str = "Clic : sélectionner · double-clic : renommer";
pub const NOTIFY_OVERLAYS_REMOVED: &str = "Overlays supprimés";
pub const LEVEL_SIGNAL: &str = "Signal";
pub const LEVEL_REFERENCE: &str = "Reference";
pub const LEVEL_PEAK: &str = "Peak";
pub const LEVEL_RMS: &str = "RMS";
pub const LEVEL_UNIT: &str = "dBFS";
pub const GROUP_TRANSFER_FUNCTION: &str = "Transfer Function";
pub const GROUP_EXTERNAL: &str = "External";
pub const VIEW_VIRTUAL_EQ: &str = "Virtual EQ";
pub const ACTION_VIEW_VIRTUAL_EQ: &str = "Module Virtual EQ (Delay Analysis, Virtual EQ)";
pub const FULL_VIEW: &str = "Full View";
pub const FULL_VIEW_HINT: &str = "Full View : limites par défaut (ou double-clic sur le graphe)";
pub const DISPLAY_TITLE: &str = "Display";
pub const SHOW_COHERENCE: &str = "Show Coherence";
pub const COHERENCE_MASK: &str = "Mask γ²";
pub const COHERENCE_MASK_HINT: &str =
    "Seuil de cohérence : les points en dessous ne sont pas tracés";
pub const SMOOTHING: &str = "Smoothing";
pub const UNIT: &str = "Unit";
pub const MODULE_DELAY_ANALYSIS: &str = "Delay Analysis";
pub const MODULE_VIRTUAL_EQ: &str = "Virtual EQ";
pub const MOUSE_PEEK: &str = "Peek";
pub const MOUSE_DRAG: &str = "Drag";
pub const MOUSE_ZOOM: &str = "Zoom";
pub const MOUSE_PEEK_HINT: &str = "Peek : lire les valeurs, le glisser ne zoome pas";
pub const MOUSE_DRAG_HINT: &str = "Drag : glisser pour déplacer le graphe";
pub const MOUSE_ZOOM_HINT: &str =
    "Zoom : glisser pour zoomer (largeur = fréquence, hauteur = niveau), double-clic pour revenir";
pub const MOUSE_BAR_IDLE: &str = "—";
pub const SETUP_PREFIX: &str = "Setup:";
pub const SETUP_LIVE_SIGNAL: &str = "Live Signal";
pub const SETUP_FILE_SIGNAL: &str = "File";
pub const SETUP_PLAY_REFERENCE: &str = "Play+Reference Signal";
pub const SETUP_REFERENCE_LOOPBACK: &str = "Reference";
pub const SETUP_NO_DEVICE: &str = "carte par défaut";
pub const TOOLS_COLOR_SCHEME: &str = "Color Scheme";
pub const TOOLS_NIGHT: &str = "Night";
pub const TOOLS_DAYLIGHT: &str = "Daylight";
pub const TOOLS_GENERAL: &str = "General";
pub const TOOLS_CHECK_UPDATES: &str = "Check for Updates";
pub const ALIGNMENT_DELAY_MS: &str = "Delay [ms]";
pub const ALIGNMENT_INV_POL: &str = "Inv Pol";
pub const ALIGNMENT_DELAY_PICKER: &str = "Delay";
pub const ALIGNMENT_REFERENCE_PICKER: &str = "Reference";
pub const NOTIFY_DELAY_RESTORED: &str = "Delay Offset rétabli";

pub const STATUS_STOPPED: &str = "Stopped";
pub const STATUS_WARMING_UP: &str = "Warm-up";
pub const STATUS_RUNNING: &str = "Live";
pub const STATUS_FROZEN: &str = "Frozen";
pub const STATUS_ERROR: &str = "Erreur";
pub const STATUS_FILE_FINISHED: &str = "Fichier terminé";

pub const METER_REFERENCE: &str = "Ref";
pub const METER_MEASUREMENT: &str = "Mic";
pub const METER_CLIP: &str = "CLIP";

pub const DELAY_TITLE: &str = "Delay";
pub const DELAY_NONE: &str = "— ms";
pub const DELAY_CONFIDENCE: &str = "Confiance du délai";

pub const CHIP_FFT: &str = "FFT";
pub const CHIP_AVERAGING: &str = "Avg";
pub const CHIP_SMOOTHING: &str = "Smoothing";
pub const CHIP_COHERENCE: &str = "Coh γ²";
pub const CHIP_REFERENCE: &str = "Ref";
pub const CHIP_GENERATOR: &str = "Pink noise";
pub const AVERAGING_INFINITE: &str = "∞";
pub const SMOOTHING_NONE: &str = "off";
pub const REFERENCE_INTERNAL: &str = "internal";
pub const GENERATOR_OFF: &str = "off";

pub const TAB_TRACES: &str = "Traces";
pub const TAB_ALIGNMENT: &str = "Alignment";
pub const TAB_EQ: &str = "EQ";
pub const EMPTY_TRACES_TITLE: &str = "Aucun overlay";
pub const EMPTY_TRACES_BODY: &str = "Capture Measurement (S) mémorise la mesure en cours.";
pub const EMPTY_ALIGNMENT_TITLE: &str = "Aligner deux sources";
pub const EMPTY_ALIGNMENT_BODY: &str =
    "Capturez au moins deux traces (par exemple tête et sub), puis choisissez-les ici.";
pub const EMPTY_EQ_TITLE: &str = "EQ virtuel";
pub const EMPTY_EQ_BODY: &str = "Double-cliquez sur le tracé ou appuyez sur E pour ajouter un filtre. Aucun traitement n'est appliqué au son.";

pub const PLOT_EMPTY_TITLE: &str = "Prêt à mesurer";
pub const PLOT_EMPTY_BODY: &str = "Start Analysis (Espace) démarre la mesure continue.";
pub const PLOT_WARMING_UP: &str = "Remplissage de la fenêtre d'analyse…";
pub const COHERENCE_LABEL: &str = "Coherence";
pub const AXIS_DB: &str = "dB";
pub const COHERENCE_AXIS_MAX: &str = "1";
pub const COHERENCE_AXIS_MIN: &str = "0";
pub const LIVE_TRACE_LABEL: &str = "Live";
pub const PLOT_ACCESSIBLE_LABEL: &str = "Réponse en fréquence (magnitude) et cohérence";
pub const PHASE_PLOT_ACCESSIBLE_LABEL: &str = "Tracé de phase";
pub const DELAY_ANALYSIS_PLOT_LABEL: &str = "Delay Analysis : sommes Pos, Neg, Flat et Custom";
pub const IMPULSE_PLOT_ACCESSIBLE_LABEL: &str = "Tracé de la réponse impulsionnelle";
pub const AXIS_DEGREES: &str = "°";
pub const AXIS_MILLISECONDS: &str = "ms · 0 = délai appliqué";
pub const AXIS_AMPLITUDE: &str = "×";
pub const IMPULSE_EMPTY: &str = "Start Analysis (Espace) pour afficher la réponse impulsionnelle";
pub const VIEW_MAGNITUDE: &str = "Mag";
pub const VIEW_PHASE: &str = "Phs";
pub const VIEW_IMPULSE: &str = "IR";
pub const VIEW_IMPULSE_HINT: &str = "Réponse impulsionnelle";
pub const VIEW_MAIN_LABEL: &str = "View";
pub const VIEW_UPPER_LABEL: &str = "Top";
pub const VIEW_STACKED: &str = "Deux graphes";
pub const PANE_SPLITTER_LABEL: &str = "Séparation des deux vues";
pub const PANE_SPLITTER_HINT: &str =
    "Glisser pour redimensionner · double-clic : taille par défaut";
pub const VIEW_FULLSCREEN: &str = "Plein écran";
pub const VIEW_EXIT_FULLSCREEN: &str = "Quitter le plein écran";
pub const IMPULSE_SCALE_LINEAR: &str = "Lin";
pub const IMPULSE_SCALE_DB: &str = "dB";
pub const IMPULSE_SCALE_HINT: &str = "Échelle de la réponse impulsionnelle : linéaire ou en dB";
pub const ACTION_VIEW_MAGNITUDE: &str = "Graphe Mag (magnitude)";
pub const ACTION_VIEW_PHASE: &str = "Graphe Phs (phase)";
pub const ACTION_VIEW_IMPULSE: &str = "Graphe IR (réponse impulsionnelle)";
pub const ACTION_TOGGLE_STACKED_VIEWS: &str = "Un ou deux graphes";
pub const ACTION_TOGGLE_FULLSCREEN: &str = "Plein écran (graphes seuls)";
pub const ACTION_CHECK_FOR_UPDATES: &str = "Rechercher les mises à jour";
pub const ACTION_OPEN_UPDATE_PAGE: &str = "Télécharger la nouvelle version";
pub const ACTION_DISMISS_UPDATE: &str = "Masquer la nouvelle version";
pub const BUTTON_DOWNLOAD_UPDATE: &str = "Télécharger";
pub const BUTTON_LATER: &str = "Plus tard";
pub const NOTIFY_UPDATE_CHECKING: &str = "Recherche des mises à jour…";
pub const NOTIFY_UPDATE_UNREACHABLE: &str =
    "Mises à jour injoignables (hors connexion ?). SoundTuning fonctionne normalement.";

pub const BANNER_START_FAILED: &str = "Impossible de démarrer la mesure";
pub const BANNER_DISCONNECTED: &str =
    "Carte son déconnectée. Rebranchez-la puis reprenez la mesure.";
pub const BANNER_PERMISSION: &str = "Accès au micro refusé. Ouvrez Réglages Système › Confidentialité et sécurité › Micro, autorisez SoundTuning, puis réessayez.";
pub const BANNER_SILENT_INPUT: &str = "L'entrée micro est entièrement muette. Vérifiez le câble, le gain et l'autorisation micro de macOS (Réglages Système › Confidentialité et sécurité › Micro).";
pub const BANNER_SILENT_REFERENCE: &str = "Référence absente : aucun signal de référence. Vérifiez le générateur ou le câble de rebouclage de la console.";
pub const BANNER_LATENCY_NOT_CALIBRATED: &str = "Latence de la carte non calibrée : le délai et la distance incluent la latence de la carte. Reliez une sortie à une entrée puis calibrez.";
pub const BANNER_SEPARATE_CLOCKS: &str = "Entrée et sortie sur deux périphériques différents : horloges distinctes, mesure moins fiable.";
pub const BANNER_LOW_COHERENCE: &str = "Cohérence faible : rapprochez le micro, montez le niveau du signal ou réduisez le bruit ambiant.";
pub const BANNER_CLIPPING: &str = "Écrêtage : baissez le gain d'entrée ou le niveau du générateur.";
pub const BANNER_CALIBRATION_RUNNING: &str =
    "Calibrage en cours : la sortie du générateur doit être reliée à l'entrée de mesure.";
pub const BANNER_CALIBRATION_FAILED: &str =
    "Calibrage impossible : aucun signal fiable sur le rebouclage. Vérifiez le câble.";
pub const BANNER_FILE_FINISHED: &str = "Fin du fichier atteinte.";

pub const PALETTE_PLACEHOLDER: &str = "Rechercher une commande…";
pub const PALETTE_NO_MATCH: &str = "Aucune commande ne correspond.";

pub const WIZARD_TITLE: &str = "Bienvenue dans SoundTuning";
pub const WIZARD_STEP_DEVICE: &str = "Carte son";
pub const WIZARD_STEP_CHANNELS: &str = "Référence et canaux";
pub const WIZARD_STEP_TEST: &str = "Test";
pub const WIZARD_DEVICE_HELP: &str = "Choisissez la carte son qui reçoit le micro de mesure.";
pub const WIZARD_NO_DEVICE: &str =
    "Aucune carte son avec entrée n'a été trouvée. Branchez votre interface puis réessayez.";
pub const WIZARD_REFERENCE_INTERNAL: &str = "Référence interne (générateur)";
pub const WIZARD_REFERENCE_INTERNAL_HELP: &str =
    "Le logiciel envoie du bruit rose et s'en sert de référence. Idéal avant le concert.";
pub const WIZARD_REFERENCE_LOOPBACK: &str = "Référence rebouclée (console)";
pub const WIZARD_REFERENCE_LOOPBACK_HELP: &str =
    "La sortie de la console revient sur une entrée : mesure sur la musique pendant le concert.";
pub const WIZARD_MEASUREMENT_CHANNEL: &str = "Entrée du micro";
pub const WIZARD_LOOPBACK_CHANNEL: &str = "Entrée de la référence";
pub const WIZARD_GENERATOR_OUTPUT: &str = "Sortie du bruit rose";
pub const WIZARD_OUTPUT_LEFT: &str = "Gauche";
pub const WIZARD_OUTPUT_RIGHT: &str = "Droite";
pub const WIZARD_OUTPUT_BOTH: &str = "Les deux";
pub const WIZARD_TEST_HELP: &str = "Vérifiez que les vu-mètres bougent : le micro doit capter le bruit rose (ou la musique), la référence doit être présente.";
pub const WIZARD_TEST_START: &str = "Lancer le test (bruit rose)";
pub const WIZARD_REFRESH: &str = "Actualiser";
pub const WIZARD_PREVIEW_ERROR: &str = "La carte ne peut pas être ouverte";

pub const NOTIFY_DELAY_FOUND: &str = "Délai trouvé";
pub const NOTIFY_DELAY_APPLIED: &str = "Délai appliqué";
pub const NOTIFY_DELAY_DRIFT: &str = "Délai changé";
pub const NOTIFY_TRACKING_ON: &str = "Auto activé (suivi du délai)";
pub const NOTIFY_TRACKING_OFF: &str = "Auto désactivé (suivi du délai)";
pub const ACTION_TOGGLE_DELAY_TRACKING: &str = "Auto : suivi continu du délai";
pub const ACTION_NUDGE_DELAY_DOWN: &str = "Délai −0,01 ms";
pub const ACTION_NUDGE_DELAY_UP: &str = "Délai +0,01 ms";
pub const BUTTON_TRACKING: &str = "Auto";
pub const TRACKING_ON_GLYPH: &str = "●";
pub const TRACKING_OFF_GLYPH: &str = "○";
pub const BUTTON_NUDGE_DOWN: &str = "−";
pub const BUTTON_NUDGE_UP: &str = "+";
pub const KEY_ARROW_LEFT: &str = "←";
pub const KEY_ARROW_RIGHT: &str = "→";
pub const NOTIFY_THEME_NIGHT: &str = "Thème Nuit";
pub const NOTIFY_THEME_DAY: &str = "Thème Plein jour";
pub const NOTIFY_LATENCY_CALIBRATED: &str = "Latence de la carte calibrée";

pub const ERROR_SOURCE_UNAVAILABLE: &str = "source audio indisponible";

pub const CSV_FREQUENCY_HEADER: &str = "Fréquence (Hz)";
pub const TRACES_AVERAGE: &str = "Average";
pub const TRACES_EXPORT_CSV: &str = "Save";
pub const TRACE_DELETE: &str = "Supprimer la trace";
pub const TRACE_DELETE_GLYPH: &str = "×";
pub const NOTIFY_TRACE_DELETED: &str = "Trace supprimée";
pub const TRACE_TOGGLE_VISIBLE: &str = "Afficher / masquer";
pub const TRACE_VISIBLE_SHORT: &str = "V";
pub const TRACE_SELECT: &str = "Sélectionner pour la moyenne";
pub const TRACE_SELECT_SHORT: &str = "Sél.";
pub const TRACE_COLOR: &str = "Changer la couleur";
pub const TRACE_OFFSET: &str = "Offset";
pub const TRACE_SMOOTHING: &str = "Définition (lissage) de la trace";
pub const TRACE_SMOOTHING_SHORT: &str = "Smoothing";
pub const TRACE_SMOOTHING_FIXED: &str =
    "Trace capturée par une version précédente : définition fixe";
pub const TRACE_RENAME_HINT: &str = "Double-clic ou F2 pour renommer, Échap pour annuler";
pub const NOTIFY_NOTHING_TO_CAPTURE: &str = "Rien à capturer : démarrez la mesure d'abord.";
pub const NOTIFY_UNDONE: &str = "Action annulée";
pub const NOTIFY_REDONE: &str = "Action rétablie";
pub const NOTIFY_NOTHING_TO_UNDO: &str = "Rien à annuler";
pub const NOTIFY_AVERAGE_NEEDS_TWO: &str = "Sélectionnez au moins deux traces pour les moyenner.";
pub const NOTIFY_EXPORT_FAILED: &str = "Export impossible";
pub const EXPORT_DIRECTORY_NAME: &str = "SoundTuning";
pub const NOTIFY_TOO_MANY_TRACES: &str = "64 traces au maximum : supprimez-en une pour capturer.";
pub const NOTIFY_AVERAGE_INCOMPATIBLE: &str =
    "Moyenne impossible : les traces n'ont pas la même fréquence d'échantillonnage.";
pub const NOTIFY_SESSION_SAVE_FAILED: &str =
    "Sauvegarde de la session impossible : nouvel essai automatique.";
pub const NOTIFY_SESSION_CORRUPT_RESTORED: &str =
    "Session illisible mise de côté : dernière sauvegarde valide restaurée.";
pub const NOTIFY_SESSION_CORRUPT_EMPTY: &str =
    "Session illisible mise de côté : nouvelle session vide.";
pub const TRACE_RENAME: &str = "Renommer";

pub const RANGE_SEPARATOR: &str = "–";
pub const ALIGNMENT_TITLE: &str = "Delay Analysis";
pub const ALIGNMENT_REFERENCE: &str = "Reference";
pub const ALIGNMENT_DELAYED: &str = "Delay";
pub const ALIGNMENT_CHOOSE_TRACE: &str = "Choisir…";
pub const ALIGNMENT_RANGE: &str = "Range";
pub const ALIGNMENT_RANGE_LOW: &str = "Low";
pub const ALIGNMENT_RANGE_MID: &str = "Mid";
pub const ALIGNMENT_RANGE_FULL: &str = "Full";
pub const ALIGNMENT_RANGE_CUSTOM: &str = "Custom";
pub const ALIGNMENT_CROSSOVER: &str = "Crossover";
pub const ALIGNMENT_BANDWIDTH: &str = "Largeur";
pub const ALIGNMENT_CROSSOVER_LABEL: &str = "Fréquence de raccord (crossover)";
pub const ALIGNMENT_BANDWIDTH_LABEL: &str = "Largeur de la plage en octaves";
pub const ALIGNMENT_BAND_HINT: &str = "Maj + glisser sur le tracé : plage Custom.";
pub const ALIGNMENT_CHOICE_POSITIVE: &str = "Pos";
pub const ALIGNMENT_CHOICE_NEGATIVE: &str = "Neg";
pub const ALIGNMENT_CHOICE_FLAT: &str = "Flat";
pub const ALIGNMENT_CHOICE_CUSTOM: &str = "Custom";
pub const ALIGNMENT_UNITS: &str =
    "Delay [ms] > 0 : retarder la trace Delay · Inv Pol : inverser la trace Delay";
pub const ALIGNMENT_POLARITY_ROW: &str = "Pol";
pub const ALIGNMENT_POLARITY_NORMAL: &str = "+";
pub const ALIGNMENT_POLARITY_INVERTED: &str = "−";
pub const ALIGNMENT_CUSTOM_DELAY_LABEL: &str = "Delay Custom (ms, > 0 : retarder la trace Delay)";
pub const ALIGNMENT_CUSTOM_POLARITY_LABEL: &str = "Polarité inversée (Custom)";
pub const ALIGNMENT_APPLIED: &str = "Réglage à recopier dans le processeur";
pub const ALIGNMENT_APPLIED_SHORT: &str = "Processeur";
pub const ALIGNMENT_NO_DELAY: &str = "Aucun délai";
pub const ALIGNMENT_NO_SOLUTION: &str = "Pas de Pos / Neg / Flat : il faut la phase des deux traces dans la plage (une moyenne de traces n'en a pas). Custom reste disponible.";
pub const ALIGNMENT_SUM_SHOW_HINT: &str = "Clic : afficher sa somme Reference + Delay sur le tracé";
pub const ALIGNMENT_SUM_HIDE_HINT: &str = "Clic : masquer la somme du tracé";
pub const ALIGNMENT_POWER_HINT: &str =
    "Barres : niveau de la somme Pos et Neg, relatif au meilleur des deux.";
pub const EQ_FREQUENCY: &str = "Fréquence";
pub const EQ_GAIN: &str = "Gain";
pub const EQ_Q: &str = "Q";
pub const EQ_KIND: &str = "Type";
pub const EQ_SLOPE: &str = "Pente";
pub const EQ_TARGET_LABEL: &str = "Trace égalisée";
pub const AUTO_EQ_TITLE: &str = "Auto EQ";
pub const AUTO_EQ_RANGE: &str = "Plage";
pub const AUTO_EQ_RANGE_LOW: &str = "Auto EQ : fréquence basse";
pub const AUTO_EQ_RANGE_HIGH: &str = "Auto EQ : fréquence haute";
pub const AUTO_EQ_MAX_BOOST: &str = "Max boost";
pub const AUTO_EQ_MAX_CUT: &str = "Max cut";
pub const AUTO_EQ_MAX_FILTERS: &str = "Max filtres";
pub const AUTO_EQ_RUN: &str = "Calculer l'auto EQ";
pub const AUTO_EQ_HINT: &str =
    "Remplace les filtres de la banque active par des PEQ (Cmd/Ctrl+Z pour revenir).";
pub const NOTIFY_AUTO_EQ_NO_TARGET: &str =
    "Auto EQ : aucune mesure à égaliser (démarrez la mesure ou choisissez une trace).";
pub const NOTIFY_AUTO_EQ_NOT_ENOUGH: &str = "Auto EQ : pas assez de points valides dans la plage.";
pub const ALIGNMENT_POLARITY_SUFFIX: &str = " · Pol −";

pub fn format_signed_milliseconds(value: f64) -> String {
    let sign = if value > 0.0 { "+" } else { "" };
    format!("{sign}{}", format_milliseconds(value))
}

pub fn alignment_choice(choice: crate::alignment_state::AlignmentChoice) -> &'static str {
    use crate::alignment_state::AlignmentChoice;
    match choice {
        AlignmentChoice::Positive => ALIGNMENT_CHOICE_POSITIVE,
        AlignmentChoice::Negative => ALIGNMENT_CHOICE_NEGATIVE,
        AlignmentChoice::Flat => ALIGNMENT_CHOICE_FLAT,
        AlignmentChoice::Custom => ALIGNMENT_CHOICE_CUSTOM,
    }
}

pub fn alignment_range(range: crate::alignment_state::AlignmentRange) -> &'static str {
    use crate::alignment_state::AlignmentRange;
    match range {
        AlignmentRange::Low => ALIGNMENT_RANGE_LOW,
        AlignmentRange::Mid => ALIGNMENT_RANGE_MID,
        AlignmentRange::Full => ALIGNMENT_RANGE_FULL,
        AlignmentRange::Custom => ALIGNMENT_RANGE_CUSTOM,
    }
}

pub fn alignment_range_label(range: crate::alignment_state::AlignmentRange) -> String {
    format!("{ALIGNMENT_RANGE} {}", alignment_range(range))
}

pub fn alignment_sum_label(choice: crate::alignment_state::AlignmentChoice) -> String {
    format!("{} sum", alignment_choice(choice))
}

pub fn alignment_choice_summary(
    choice: crate::alignment_state::AlignmentChoice,
    setting: Option<(f64, bool)>,
) -> String {
    match setting {
        Some((delay_ms, inverted)) => format!(
            "{} : {} · Pol {}",
            alignment_choice(choice),
            format_signed_milliseconds(delay_ms),
            if inverted {
                ALIGNMENT_POLARITY_INVERTED
            } else {
                ALIGNMENT_POLARITY_NORMAL
            }
        ),
        None => alignment_choice(choice).to_owned(),
    }
}

pub fn alignment_choice_hint(
    choice: crate::alignment_state::AlignmentChoice,
    setting: Option<(f64, bool)>,
    sum_shown: bool,
) -> String {
    format!(
        "{}\n{}",
        alignment_choice_summary(choice, setting),
        if sum_shown {
            ALIGNMENT_SUM_HIDE_HINT
        } else {
            ALIGNMENT_SUM_SHOW_HINT
        }
    )
}

pub fn alignment_instruction(
    delay_ms: f64,
    inverted: bool,
    reference_name: &str,
    delayed_name: &str,
) -> String {
    let delay = if delay_ms.abs() < 0.005 {
        ALIGNMENT_NO_DELAY.to_owned()
    } else if delay_ms > 0.0 {
        format!(
            "Delay « {delayed_name} » : {}",
            format_milliseconds(delay_ms)
        )
    } else {
        format!(
            "Delay « {reference_name} » : {}",
            format_milliseconds(-delay_ms)
        )
    };
    if inverted {
        format!("{delay}{ALIGNMENT_POLARITY_SUFFIX} sur « {delayed_name} »")
    } else {
        delay
    }
}

pub const EQ_GLOBAL_BYPASS: &str = "Global bypass";
pub const EQ_VIEW_PREDICTED: &str = "Prédit";
pub const EQ_VIEW_ONLY: &str = "EQ seul";
pub const EQ_VIEW_INVERTED: &str = "Trace inversée";
pub const EQ_VIEW_INVERTED_SHORT: &str = "Inversée";
pub const EQ_TARGET_LIVE: &str = "Mesure en direct";
pub const EQ_EXPORT_TEXT: &str = "Exporter texte";
pub const EQ_EXPORT_JSON: &str = "Exporter JSON";
pub const EQ_BYPASS: &str = "Bypass";
pub const EQ_REMOVE: &str = "Retirer le filtre";
pub const EQ_CURVE_LABEL: &str = "EQ";
pub const EQ_PREDICTED_LABEL: &str = "Résultat prédit";
pub const EQ_INVERTED_LABEL: &str = "Trace inversée";
pub const NOTIFY_EQ_FULL: &str = "La banque contient déjà 16 filtres.";

pub fn eq_bank_label(index: usize) -> String {
    format!("Banque {}", bank_label(index))
}

pub fn filter_number(number: usize) -> String {
    format!("F{number}")
}

pub const FILTER_BELL: &str = "PEQ";
pub const FILTER_LOW_SHELF: &str = "Low shelf";
pub const FILTER_HIGH_SHELF: &str = "High shelf";
pub const FILTER_LOW_CUT: &str = "Low cut";
pub const FILTER_HIGH_CUT: &str = "High cut";
pub const FILTER_NOTCH: &str = "Notch";
pub const SLOPE_BW12: &str = "Butterworth 12 dB/oct";
pub const SLOPE_BW24: &str = "Butterworth 24 dB/oct";
pub const SLOPE_LR24: &str = "Linkwitz-Riley 24 dB/oct";
pub const SLOPE_LR48: &str = "Linkwitz-Riley 48 dB/oct";

pub fn trace_name(number: u64, clock: &str) -> String {
    format!("Trace {number} — {clock}")
}

pub fn average_trace_name(number: u64, clock: &str, member_count: usize) -> String {
    format!("Moyenne {number} ({member_count} traces) — {clock}")
}

pub fn eq_control_label(control: &str, filter_number: usize) -> String {
    format!("{control} du filtre {filter_number}")
}

pub fn format_sample_rate(sample_rate_hz: u32) -> String {
    format!(
        "{} kHz",
        format_decimal(f64::from(sample_rate_hz) / 1000.0, 1)
    )
}

pub fn format_buffer(frames: Option<u32>) -> String {
    frames.map_or_else(
        || AUDIO_BUFFER_AUTO.to_owned(),
        |frames| format!("{frames} éch."),
    )
}

pub fn parse_number(input: &str) -> Option<f64> {
    let lower = input.trim().to_lowercase();
    let multiplier = if lower.contains('k') { 1000.0 } else { 1.0 };
    let cleaned: String = lower
        .chars()
        .filter_map(|character| match character {
            '−' => Some('-'),
            ',' => Some('.'),
            '0'..='9' | '.' | '-' | '+' => Some(character),
            _ => None,
        })
        .collect();
    cleaned
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .map(|value| value * multiplier)
}

pub fn auto_eq_done(filter_count: usize, residual_db: f64) -> String {
    format!(
        "Auto EQ : {filter_count} PEQ, écart {} dB RMS",
        format_decimal(residual_db, 1)
    )
}

pub fn tracking_label(active: bool) -> String {
    let glyph = if active {
        TRACKING_ON_GLYPH
    } else {
        TRACKING_OFF_GLYPH
    };
    format!("{glyph} {BUTTON_TRACKING}")
}

pub fn band_label(low_hz: f64, high_hz: f64) -> String {
    format!(
        "{} {RANGE_SEPARATOR} {}",
        format_frequency(low_hz),
        format_frequency(high_hz)
    )
}

pub fn delay_drift(acoustic_ms: f64, drift_ms: f64) -> String {
    format!(
        "{NOTIFY_DELAY_DRIFT} : {} ({})",
        format_milliseconds(acoustic_ms),
        format_signed_milliseconds(drift_ms)
    )
}

pub fn format_csv_number(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}").replace('.', ",")
}

pub fn trace_control_label(control: &str, trace_name: &str) -> String {
    format!("{control} — {trace_name}")
}

pub fn csv_coherence_header(trace_name: &str) -> String {
    format!("{trace_name} γ²")
}

pub fn trace_captured(name: &str) -> String {
    format!("Trace capturée : {name}")
}

pub fn exported_to(path: &str) -> String {
    format!("Exporté : {path}")
}

pub fn bank_label(index: usize) -> &'static str {
    if index == 0 { "A" } else { "B" }
}

pub fn eq_export_header(bank_index: usize) -> String {
    format!("SoundTuning — EQ banque {}", bank_label(bank_index))
}

pub fn format_q(q: f64) -> String {
    format!("Q {}", format_decimal(q, 2))
}

pub fn format_octaves(octaves: f64) -> String {
    format!("{} oct.", format_decimal(octaves, 2))
}

pub fn eq_export_line(number: usize, filter: &crate::eq_state::StoredFilter) -> String {
    let mut parts = vec![
        format!("{number}."),
        filter_kind_label(filter.kind),
        format_frequency(filter.frequency_hz),
    ];
    if filter.kind.to_dsp().uses_gain() {
        parts.push(format_db(filter.gain_db));
    }
    if filter.kind.to_dsp().uses_q() {
        parts.push(format_q(filter.q));
    }
    if let Some(octaves) = filter.bandwidth_octaves() {
        parts.push(format!("({})", format_octaves(octaves)));
    }
    if filter.bypassed {
        parts.push("[bypass]".to_owned());
    }
    parts.join("  ")
}

pub fn filter_kind_label(kind: crate::eq_state::StoredFilterKind) -> String {
    use crate::eq_state::StoredFilterKind;
    match kind {
        StoredFilterKind::LowCut(slope) | StoredFilterKind::HighCut(slope) => {
            format!("{} {}", kind.label(), slope.label())
        }
        other => other.label().to_owned(),
    }
}

pub fn format_decimal(value: f64, decimals: usize) -> String {
    let formatted = format!("{value:.decimals$}").replace('.', ",");
    if formatted.starts_with('-') {
        formatted.replacen('-', "−", 1)
    } else {
        formatted
    }
}

pub fn format_dbfs(value: f32) -> String {
    format!("{} dBFS", format_decimal(f64::from(value), 1))
}

pub fn format_meter_dbfs(value: f32) -> String {
    if value <= crate::design::widgets::METER_FLOOR_DBFS {
        "−∞".to_owned()
    } else {
        format_decimal(f64::from(value), 0)
    }
}

pub fn format_milliseconds(value: f64) -> String {
    format!("{} ms", format_decimal(value, 2))
}

pub fn format_meters(value: f64) -> String {
    format!("{} m", format_decimal(value, 2))
}

pub fn format_samples(value: f64) -> String {
    format!("{} éch.", format_decimal(value, 0))
}

pub fn format_frequency(frequency_hz: f64) -> String {
    if frequency_hz >= 1000.0 {
        let kilohertz = frequency_hz / 1000.0;
        let decimals = if kilohertz >= 10.0 { 1 } else { 2 };
        format!("{} kHz", format_decimal(kilohertz, decimals))
    } else {
        format!(
            "{} Hz",
            format_decimal(frequency_hz, if frequency_hz < 100.0 { 1 } else { 0 })
        )
    }
}

pub fn format_axis_frequency(frequency_hz: f64) -> String {
    if frequency_hz >= 1000.0 {
        format!("{}k", format_decimal(frequency_hz / 1000.0, 0))
    } else {
        format_decimal(frequency_hz, 0)
    }
}

pub fn format_db(value: f64) -> String {
    let sign = if value > 0.0 { "+" } else { "" };
    format!("{sign}{} dB", format_decimal(value, 1))
}

pub fn format_axis_db(value: f64) -> String {
    let sign = if value > 0.0 { "+" } else { "" };
    format!("{sign}{}", format_decimal(value, 0))
}

pub fn format_fft_length(length: usize) -> String {
    format!("{}k", length / 1024)
}

pub fn format_seconds(value: f64) -> String {
    format!(
        "{} s",
        format_decimal(value, if value.fract() == 0.0 { 0 } else { 1 })
    )
}

pub fn labelled_value(label: &str, value: &str) -> String {
    format!("{label} : {value}")
}

pub fn with_shortcut(label: &str, shortcut: Option<&str>) -> String {
    match shortcut {
        Some(shortcut) => format!("{label} ({shortcut})"),
        None => label.to_owned(),
    }
}

pub fn upper_view_label(view: &str) -> String {
    format!("{view} ({})", VIEW_UPPER_LABEL.to_lowercase())
}

pub fn format_axis_milliseconds(value_ms: f64) -> String {
    let decimals = if value_ms.fract().abs() > f64::EPSILON {
        1
    } else {
        0
    };
    format_decimal(value_ms, decimals)
}

pub fn format_axis_amplitude(value: f64) -> String {
    format_decimal(value, 1)
}

pub fn format_amplitude(value: f64) -> String {
    format_decimal(value, 2)
}

pub fn impulse_peak(time_ms: f64) -> String {
    format!("Pic : {}", format_milliseconds(time_ms))
}

pub fn impulse_readout(time_ms: f64, distance_m: f64) -> String {
    format!(
        "{} · {}",
        format_milliseconds(time_ms),
        format_meters(distance_m)
    )
}

pub fn format_degrees(value_deg: f64) -> String {
    let sign = if value_deg.round() > 0.0 { "+" } else { "" };
    format!("{sign}{}°", format_decimal(value_deg, 0))
}

pub fn format_axis_degrees(value_deg: f64) -> String {
    format!("{}°", format_decimal(value_deg, 0))
}

pub fn crosshair_phase_readout(frequency_hz: f64, phase_deg: f64) -> String {
    format!(
        "{} · {}",
        format_frequency(frequency_hz),
        format_degrees(phase_deg)
    )
}

pub fn format_smoothing(fraction: u32) -> String {
    if fraction == 0 {
        SMOOTHING_NONE.to_owned()
    } else {
        format!("1/{fraction}")
    }
}

pub fn format_input_channel(index: usize) -> String {
    format!("entrée {}", index + 1)
}

pub fn device_line(
    id: &str,
    name: &str,
    input_channels: usize,
    output_channels: usize,
    rates_hz: &[u32],
    marks: &[&str],
) -> String {
    let rates = rates_hz
        .iter()
        .map(|rate| format!("{} kHz", format_decimal(f64::from(*rate) / 1000.0, 1)))
        .collect::<Vec<_>>()
        .join(", ");
    let marks = if marks.is_empty() {
        String::new()
    } else {
        format!(" [{}]", marks.join(", "))
    };
    format!(
        "{name}{marks}\n  id : {id}\n  entrées : {input_channels}, sorties : {output_channels}\n  fréquences : {rates}"
    )
}

pub fn device_channels(input_channels: usize, output_channels: usize) -> String {
    format!("{input_channels} entrées · {output_channels} sorties")
}

pub fn record_started(seconds: f64, path: &str) -> String {
    format!(
        "Enregistrement de {} s vers {path}…",
        format_decimal(seconds, 1)
    )
}

pub fn record_finished(frame_count: usize, sample_rate_hz: u32, path: &str) -> String {
    format!(
        "Terminé : {} s enregistrées dans {path}.",
        format_decimal(frame_count as f64 / f64::from(sample_rate_hz), 2)
    )
}

pub fn record_incomplete(frame_count: usize, expected: usize) -> String {
    format!("Enregistrement incomplet : {frame_count} trames sur {expected} attendues.")
}

pub fn source_label(name: &str, sample_rate_hz: u32) -> String {
    format!(
        "{name} · {} kHz",
        format_decimal(f64::from(sample_rate_hz) / 1000.0, 1)
    )
}

pub fn file_source_label(file_name: &str, sample_rate_hz: u32) -> String {
    format!(
        "Fichier {file_name} · {} kHz",
        format_decimal(f64::from(sample_rate_hz) / 1000.0, 1)
    )
}

pub fn device_name(device: &audio_io::DeviceInfo) -> String {
    if device.is_default_driver {
        device.name.clone()
    } else {
        format!("{} ({})", device.name, device.driver)
    }
}

pub fn auto_output_label(resolved: Option<&audio_io::DeviceInfo>) -> String {
    match resolved {
        Some(device) => format!("{AUDIO_AUTO_OUTPUT} — {}", device_name(device)),
        None => format!("{AUDIO_AUTO_OUTPUT} — sortie du système"),
    }
}

fn format_rate_list(rates_hz: &[u32]) -> String {
    if rates_hz.is_empty() {
        return "aucune".to_owned();
    }
    rates_hz
        .iter()
        .map(|rate| format_sample_rate(*rate))
        .collect::<Vec<_>>()
        .join(", ")
}

fn direction_label(direction: &str) -> &'static str {
    if direction == "input" {
        "l'entrée"
    } else {
        "la sortie"
    }
}

pub fn audio_error(error: &audio_io::AudioIoError) -> String {
    use audio_io::AudioIoError;
    match error {
        AudioIoError::DeviceNotFound(_) => {
            "périphérique audio introuvable : vérifiez qu'il est branché puis ouvrez les réglages audio".to_owned()
        }
        AudioIoError::ChannelOutOfRange {
            direction,
            channel_index,
            available,
        } => format!(
            "{} n'a pas de canal {} (elle en a {available})",
            direction_label(direction),
            channel_index + 1
        ),
        AudioIoError::UnsupportedSampleRate {
            direction,
            sample_rate_hz,
        } => format!(
            "{} ne prend pas en charge {}",
            direction_label(direction),
            format_sample_rate(*sample_rate_hz)
        ),
        AudioIoError::NoCommonSampleRate {
            input_rates_hz,
            output_rates_hz,
        } => format!(
            "aucune fréquence commune entre l'entrée ({}) et la sortie ({}) : choisissez une autre sortie ou la référence rebouclée",
            format_rate_list(input_rates_hz),
            format_rate_list(output_rates_hz)
        ),
        AudioIoError::UnsupportedSampleFormat(format) => {
            format!("format d'échantillons non pris en charge ({format})")
        }
        AudioIoError::NoGeneratorChannel => {
            "choisissez au moins une sortie pour le bruit rose".to_owned()
        }
        AudioIoError::MissingGeneratorOutput => {
            "la référence interne a besoin d'une sortie audio : choisissez-en une dans les réglages audio".to_owned()
        }
        other => other.to_string(),
    }
}

pub fn sample_rate_adjusted(actual_hz: u32, requested_hz: u32) -> String {
    format!(
        "Fréquence adaptée à {} : {} n'est pas disponible sur l'entrée et la sortie choisies",
        format_sample_rate(actual_hz),
        format_sample_rate(requested_hz)
    )
}

pub fn delay_found(milliseconds: f64) -> String {
    format!(
        "{NOTIFY_DELAY_FOUND} : {}",
        format_milliseconds(milliseconds)
    )
}

pub fn delay_details(distance_m: f64, samples: f64, approximate: bool) -> String {
    let prefix = if approximate { "≈ " } else { "" };
    format!(
        "{prefix}{} · {}",
        format_meters(distance_m),
        format_samples(samples)
    )
}

pub fn update_available(latest: impl std::fmt::Display, current: impl std::fmt::Display) -> String {
    format!("SoundTuning {latest} est disponible (version installée : {current}).")
}

pub fn up_to_date(current: impl std::fmt::Display) -> String {
    format!("SoundTuning {current} est à jour.")
}

pub fn start_failed(error: &str) -> String {
    format!("{BANNER_START_FAILED} : {error}")
}

pub fn delay_applied(milliseconds: f64) -> String {
    format!(
        "{NOTIFY_DELAY_APPLIED} : {}",
        format_milliseconds(milliseconds)
    )
}

pub fn latency_calibrated(milliseconds: f64) -> String {
    format!(
        "{NOTIFY_LATENCY_CALIBRATED} : {}",
        format_milliseconds(milliseconds)
    )
}

pub fn wizard_step(index: usize, total: usize) -> String {
    format!("Étape {index} sur {total}")
}

pub fn display_option_label(option: &str) -> String {
    format!("{DISPLAY_TITLE} : {option}")
}

pub fn version_label(version: impl std::fmt::Display) -> String {
    format!("{APP_NAME} {version}")
}

pub fn calibrated_latency(milliseconds: f64) -> String {
    format!("Calibrated: {}", format_milliseconds(milliseconds))
}

pub fn peak_applied(milliseconds: f64) -> String {
    format!("{DELAY_OFFSET} : {}", format_milliseconds(milliseconds))
}

pub fn delay_restored(milliseconds: f64) -> String {
    format!(
        "{NOTIFY_DELAY_RESTORED} : {}",
        format_milliseconds(milliseconds)
    )
}

pub fn overlays_removed(count: usize) -> String {
    format!("{NOTIFY_OVERLAYS_REMOVED} : {count}")
}

pub fn channel_number(index: usize) -> String {
    (index + 1).to_string()
}

pub fn signal_channel_label(index: usize) -> String {
    format!("{SIGNAL_CHANNEL} {}", index + 1)
}

pub fn reference_channel_label(index: usize) -> String {
    format!("{REFERENCE_CHANNEL} {}", index + 1)
}

pub fn play_channel_label(index: usize) -> String {
    format!("{PLAY_CHANNEL} {}", index + 1)
}

pub fn fft_size_choice(length: usize, sample_rate_hz: u32) -> String {
    let rate = f64::from(sample_rate_hz.max(1));
    let duration_s = length as f64 / rate;
    format!(
        "{} · {} s · {} Hz",
        format_fft_length(length),
        format_decimal(duration_s, 2),
        format_decimal(rate / length.max(1) as f64, 2)
    )
}

pub fn averages_choice(time_constant_s: Option<f64>) -> String {
    time_constant_s.map_or_else(
        || format!("{AVERAGING_INFINITE} (Inf)"),
        |seconds| format!("Exp {}", format_seconds(seconds)),
    )
}

pub fn setup_line(
    file_mode: bool,
    source: Option<&str>,
    reference: Option<&str>,
    generator: Option<(f32, &str)>,
) -> String {
    let signal = if file_mode {
        SETUP_FILE_SIGNAL
    } else {
        SETUP_LIVE_SIGNAL
    };
    let mut parts = vec![format!(
        "{SETUP_PREFIX} {signal} {}",
        source.unwrap_or(SETUP_NO_DEVICE)
    )];
    if let Some(reference) = reference {
        parts.push(format!("{SETUP_REFERENCE_LOOPBACK}: {reference}"));
    }
    if let Some((level_dbfs, outputs)) = generator {
        parts.push(format!(
            "{SETUP_PLAY_REFERENCE}: {OUTPUT_PINK_NOISE} {} on {outputs}",
            format_dbfs(level_dbfs)
        ));
    }
    parts.join(" / ")
}

pub fn output_channels(indices: &[usize]) -> String {
    let numbers: Vec<String> = indices.iter().map(|index| channel_number(*index)).collect();
    format!("out {}", numbers.join("+"))
}

pub fn readout_with_trace(readout: &str, trace: &str, value: &str) -> String {
    format!("{readout}    {trace} : {value}")
}

pub fn crosshair_readout(frequency_hz: f64, level_db: f64) -> String {
    format!(
        "{} · {}",
        format_frequency(frequency_hz),
        format_db(level_db)
    )
}

#[cfg(test)]
mod tests {

    #[test]
    fn typed_numbers_accept_french_format_and_units() {
        assert_eq!(parse_number("1,5"), Some(1.5));
        assert_eq!(parse_number("−5,0 dB"), Some(-5.0));
        assert_eq!(parse_number("1,20 kHz"), Some(1200.0));
        assert_eq!(parse_number("250 Hz"), Some(250.0));
        assert_eq!(parse_number("Q 1,40"), Some(1.4));
        assert_eq!(parse_number("+3"), Some(3.0));
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number(&format_frequency(1000.0)), Some(1000.0));
        assert_eq!(parse_number(&format_db(-4.5)), Some(-4.5));
    }
    use super::*;

    #[test]
    fn decimals_use_french_comma_and_true_minus() {
        assert_eq!(format_decimal(12.4321, 2), "12,43");
        assert_eq!(format_dbfs(-3.0), "−3,0 dBFS");
        assert_eq!(format_db(6.0), "+6,0 dB");
    }

    #[test]
    fn source_label_shows_khz() {
        assert_eq!(source_label("UAX 2-2", 48_000), "UAX 2-2 · 48,0 kHz");
    }

    #[test]
    fn frequencies_are_human_readable() {
        assert_eq!(format_frequency(63.0), "63,0 Hz");
        assert_eq!(format_frequency(250.0), "250 Hz");
        assert_eq!(format_frequency(1250.0), "1,25 kHz");
        assert_eq!(format_frequency(12_500.0), "12,5 kHz");
        assert_eq!(format_axis_frequency(2000.0), "2k");
        assert_eq!(format_axis_frequency(50.0), "50");
    }

    #[test]
    fn alignment_instruction_names_the_source_to_delay() {
        assert_eq!(
            alignment_instruction(-1.5, false, "Top", "Sub"),
            "Delay « Top » : 1,50 ms"
        );
        assert_eq!(
            alignment_instruction(2.0, true, "Top", "Sub"),
            "Delay « Sub » : 2,00 ms · Pol − sur « Sub »"
        );
        assert_eq!(
            alignment_instruction(0.0, false, "Top", "Sub"),
            ALIGNMENT_NO_DELAY
        );
        assert_eq!(
            alignment_choice_summary(
                crate::alignment_state::AlignmentChoice::Negative,
                Some((-3.25, true))
            ),
            "Neg : −3,25 ms · Pol −"
        );
    }

    #[test]
    fn setup_line_reads_like_the_systune_status_bar() {
        assert_eq!(
            setup_line(
                false,
                Some("UAX 2-2 · 48,0 kHz"),
                None,
                Some((-20.0, "out 1+2"))
            ),
            "Setup: Live Signal UAX 2-2 · 48,0 kHz / Play+Reference Signal: Pink Noise −20,0 dBFS on out 1+2"
        );
        assert_eq!(
            setup_line(
                true,
                Some("Fichier a.wav · 48,0 kHz"),
                Some("entrée 2"),
                None
            ),
            "Setup: File Fichier a.wav · 48,0 kHz / Reference: entrée 2"
        );
        assert_eq!(fft_size_choice(32_768, 48_000), "32k · 0,68 s · 1,46 Hz");
        assert_eq!(averages_choice(None), "∞ (Inf)");
        assert_eq!(averages_choice(Some(2.0)), "Exp 2 s");
    }

    #[test]
    fn settings_values_are_compact() {
        assert_eq!(format_fft_length(32_768), "32k");
        assert_eq!(format_seconds(0.5), "0,5 s");
        assert_eq!(format_seconds(2.0), "2 s");
        assert_eq!(format_smoothing(0), SMOOTHING_NONE);
        assert_eq!(format_smoothing(48), "1/48");
    }

    #[test]
    fn audio_errors_are_explained_in_french() {
        let error = audio_io::AudioIoError::NoCommonSampleRate {
            input_rates_hz: vec![48_000],
            output_rates_hz: vec![44_100],
        };
        assert_eq!(
            audio_error(&error),
            "aucune fréquence commune entre l'entrée (48,0 kHz) et la sortie (44,1 kHz) : choisissez une autre sortie ou la référence rebouclée"
        );
        let error = audio_io::AudioIoError::UnsupportedSampleRate {
            direction: "output",
            sample_rate_hz: 48_000,
        };
        assert_eq!(
            audio_error(&error),
            "la sortie ne prend pas en charge 48,0 kHz"
        );
    }

    #[test]
    fn devices_from_another_driver_show_it() {
        let mut device = audio_io::DeviceInfo {
            id: "asio:UAX".into(),
            name: "UAX 2-2".into(),
            driver: "ASIO".into(),
            is_default_driver: false,
            input_channel_count: 2,
            output_channel_count: 2,
            input_sample_rates_hz: vec![48_000],
            output_sample_rates_hz: vec![48_000],
            is_default_input: false,
            is_default_output: false,
        };
        assert_eq!(device_name(&device), "UAX 2-2 (ASIO)");
        device.is_default_driver = true;
        assert_eq!(device_name(&device), "UAX 2-2");
    }
}
