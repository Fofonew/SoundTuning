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
pub const CLI_OUTPUT_DEVICE_HELP: &str =
    "Identifiant du périphérique de sortie du générateur (par défaut : celui d'entrée).";
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

pub const ACTION_TOGGLE_MEASUREMENT: &str = "Démarrer / arrêter la mesure";
pub const ACTION_FIND_DELAY: &str = "Trouver le délai";
pub const ACTION_APPLY_DELAY: &str = "Appliquer le délai";
pub const ACTION_CAPTURE_TRACE: &str = "Capturer une trace";
pub const ACTION_TOGGLE_FREEZE: &str = "Figer / libérer l'affichage";
pub const ACTION_RANGE_FULL: &str = "Plage : plein spectre";
pub const ACTION_RANGE_SUB: &str = "Plage : sub (20–200 Hz)";
pub const ACTION_RANGE_MID: &str = "Plage : médium (150 Hz–4 kHz)";
pub const ACTION_RANGE_HIGH: &str = "Plage : aigu (2–20 kHz)";
pub const ACTION_RESET_VIEW: &str = "Réinitialiser le zoom";
pub const ACTION_TOGGLE_THEME: &str = "Thème Nuit / Plein jour";
pub const ACTION_OPEN_PALETTE: &str = "Palette de commandes";
pub const ACTION_UNDO: &str = "Annuler";
pub const ACTION_REDO: &str = "Rétablir";
pub const ACTION_TOGGLE_SIDE_PANEL: &str = "Afficher / masquer le panneau latéral";
pub const ACTION_TOGGLE_GENERATOR: &str = "Générateur marche / arrêt";
pub const ACTION_CALIBRATE_LATENCY: &str = "Calibrer la latence de la carte";
pub const ACTION_ADD_EQ_FILTER: &str = "Ajouter un filtre d'EQ";
pub const ACTION_EXPORT_SESSION: &str = "Exporter (CSV, filtres)";
pub const ACTION_RESTART_MEASUREMENT: &str = "Relancer la mesure";

pub const BUTTON_START: &str = "Démarrer";
pub const BUTTON_STOP: &str = "Arrêter";
pub const BUTTON_CAPTURE: &str = "Capturer";
pub const BUTTON_FREEZE: &str = "Figer";
pub const BUTTON_UNFREEZE: &str = "Libérer";
pub const BUTTON_FIND_DELAY: &str = "Trouver";
pub const BUTTON_APPLY_DELAY: &str = "Appliquer";
pub const BUTTON_PALETTE: &str = "Commandes";
pub const BUTTON_AUDIO_SETTINGS: &str = "Audio";
pub const ACTION_OPEN_AUDIO_SETTINGS: &str = "Réglages audio (entrée, sortie, canaux)";
pub const AUDIO_SETTINGS_TITLE: &str = "Réglages audio";
pub const AUDIO_SETTINGS_CLOSE: &str = "Fermer les réglages audio";
pub const AUDIO_INPUT_DEVICE: &str = "Entrée (micro et référence)";
pub const AUDIO_OUTPUT_DEVICE: &str = "Sortie (bruit rose)";
pub const AUDIO_SAMPLE_RATE: &str = "Fréquence d'échantillonnage";
pub const AUDIO_BUFFER: &str = "Taille de tampon";
pub const AUDIO_BUFFER_AUTO: &str = "Automatique";
pub const AUDIO_REFERENCE_MODE: &str = "Référence";
pub const AUDIO_REFERENCE_INTERNAL: &str = "Interne";
pub const AUDIO_REFERENCE_LOOPBACK: &str = "Rebouclée";
pub const AUDIO_NONE: &str = "Aucune";
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
pub const BUTTON_FINISH_WIZARD: &str = "Démarrer la mesure";
pub const BUTTON_SKIP_WIZARD: &str = "Passer";
pub const BUTTON_TOGGLE_PANEL: &str = "Panneau";

pub const STATUS_STOPPED: &str = "Arrêté";
pub const STATUS_WARMING_UP: &str = "Préchauffage";
pub const STATUS_RUNNING: &str = "Mesure";
pub const STATUS_FROZEN: &str = "Figé";
pub const STATUS_ERROR: &str = "Erreur";
pub const STATUS_FILE_FINISHED: &str = "Fichier terminé";

pub const METER_REFERENCE: &str = "Réf.";
pub const METER_MEASUREMENT: &str = "Mic";
pub const METER_CLIP: &str = "ÉCRÊTAGE";

pub const DELAY_TITLE: &str = "Délai";
pub const DELAY_NONE: &str = "— ms";
pub const DELAY_CONFIDENCE: &str = "Confiance du délai";

pub const CHIP_FFT: &str = "FFT";
pub const CHIP_AVERAGING: &str = "Moyenne";
pub const CHIP_SMOOTHING: &str = "Lissage";
pub const CHIP_COHERENCE: &str = "Seuil γ²";
pub const CHIP_REFERENCE: &str = "Réf.";
pub const CHIP_GENERATOR: &str = "Bruit rose";
pub const AVERAGING_INFINITE: &str = "∞";
pub const SMOOTHING_NONE: &str = "aucun";
pub const REFERENCE_INTERNAL: &str = "interne";
pub const GENERATOR_OFF: &str = "arrêt";

pub const TAB_TRACES: &str = "Traces";
pub const TAB_ALIGNMENT: &str = "Alignement";
pub const TAB_EQ: &str = "EQ";
pub const EMPTY_TRACES_TITLE: &str = "Aucune trace";
pub const EMPTY_TRACES_BODY: &str = "Appuyez sur S pour figer la mesure en cours dans une trace.";
pub const EMPTY_ALIGNMENT_TITLE: &str = "Aligner deux sources";
pub const EMPTY_ALIGNMENT_BODY: &str =
    "Capturez au moins deux traces (par exemple tête et sub), puis choisissez-les ici.";
pub const EMPTY_EQ_TITLE: &str = "EQ virtuel";
pub const EMPTY_EQ_BODY: &str = "Double-cliquez sur le tracé ou appuyez sur E pour ajouter un filtre. Aucun traitement n'est appliqué au son.";

pub const PLOT_EMPTY_TITLE: &str = "Prêt à mesurer";
pub const PLOT_EMPTY_BODY: &str = "Appuyez sur Espace pour démarrer la mesure continue.";
pub const PLOT_WARMING_UP: &str = "Remplissage de la fenêtre d'analyse…";
pub const COHERENCE_LABEL: &str = "Cohérence";
pub const AXIS_DB: &str = "dB";
pub const COHERENCE_AXIS_MAX: &str = "1";
pub const COHERENCE_AXIS_MIN: &str = "0";
pub const LIVE_TRACE_LABEL: &str = "Mesure";
pub const PLOT_ACCESSIBLE_LABEL: &str = "Réponse en fréquence (magnitude) et cohérence";

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
pub const NOTIFY_TRACKING_ON: &str = "Suivi du délai activé";
pub const NOTIFY_TRACKING_OFF: &str = "Suivi du délai désactivé";
pub const ACTION_TOGGLE_DELAY_TRACKING: &str = "Suivi du délai";
pub const ACTION_NUDGE_DELAY_DOWN: &str = "Délai −0,01 ms";
pub const ACTION_NUDGE_DELAY_UP: &str = "Délai +0,01 ms";
pub const BUTTON_TRACKING: &str = "Suivi";
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
pub const TRACES_AVERAGE: &str = "Moyenne des sélectionnées";
pub const TRACES_EXPORT_CSV: &str = "Exporter CSV";
pub const TRACE_DELETE: &str = "Supprimer la trace";
pub const TRACE_DELETE_GLYPH: &str = "×";
pub const NOTIFY_TRACE_DELETED: &str = "Trace supprimée";
pub const TRACE_TOGGLE_VISIBLE: &str = "Afficher / masquer";
pub const TRACE_VISIBLE_SHORT: &str = "Visible";
pub const TRACE_SELECT: &str = "Sélectionner pour la moyenne";
pub const TRACE_SELECT_SHORT: &str = "Sél.";
pub const TRACE_COLOR: &str = "Changer la couleur";
pub const TRACE_OFFSET: &str = "Décalage";
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
pub const ALIGNMENT_TRACE_A: &str = "Trace A";
pub const ALIGNMENT_TRACE_B: &str = "Trace B";
pub const ALIGNMENT_CHOOSE_TRACE: &str = "Choisir…";
pub const ALIGNMENT_CRITERION_LEVEL: &str = "Niveau max";
pub const ALIGNMENT_CRITERION_FLAT: &str = "Plus plat";
pub const ALIGNMENT_BAND: &str = "Bande";
pub const ALIGNMENT_BAND_HINT: &str = "Maj + glisser sur le tracé pour choisir la bande.";
pub const ALIGNMENT_PROPOSAL: &str = "Proposition";
pub const ALIGNMENT_USE_PROPOSAL: &str = "Utiliser la proposition";
pub const ALIGNMENT_APPLIED: &str = "Réglage à recopier dans le processeur";
pub const ALIGNMENT_INVERT_B: &str = "Inverser la polarité de B";
pub const ALIGNMENT_SIGN_HINT: &str = "Δ > 0 : retarder B\nΔ < 0 : retarder A";
pub const ALIGNMENT_NO_DELAY: &str = "Aucun délai";
pub const ALIGNMENT_DELTA: &str = "Δ";
pub const ALIGNMENT_DELTA_LABEL: &str = "Délai Δ entre A et B";
pub const ALIGNMENT_BAND_LOW: &str = "Bande : fréquence basse";
pub const ALIGNMENT_BAND_HIGH: &str = "Bande : fréquence haute";
pub const EQ_FREQUENCY: &str = "Fréquence";
pub const EQ_GAIN: &str = "Gain";
pub const EQ_Q: &str = "Q";
pub const EQ_KIND: &str = "Type";
pub const EQ_SLOPE: &str = "Pente";
pub const EQ_TARGET_LABEL: &str = "Trace égalisée";
pub const ALIGNMENT_NO_PROPOSAL: &str = "Pas de proposition : il faut la phase des deux traces dans la bande (une moyenne de traces n'en a pas).";
pub const PREDICTED_SUM_LABEL: &str = "Somme prédite";
pub const ALIGNMENT_POLARITY_SUFFIX: &str = " · B inversée";

pub fn format_signed_milliseconds(value: f64) -> String {
    let sign = if value > 0.0 { "+" } else { "" };
    format!("{sign}{}", format_milliseconds(value))
}

pub fn alignment_instruction(delay_ms: f64, inverted: bool) -> String {
    let delay = if delay_ms.abs() < 0.005 {
        ALIGNMENT_NO_DELAY.to_owned()
    } else if delay_ms > 0.0 {
        format!("Retarder B de {}", format_milliseconds(delay_ms))
    } else {
        format!("Retarder A de {}", format_milliseconds(-delay_ms))
    };
    if inverted {
        format!("{delay}{ALIGNMENT_POLARITY_SUFFIX}")
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

pub fn audio_error(error: &str) -> String {
    format!("Erreur audio : {error}")
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

pub fn crosshair_readout(frequency_hz: f64, level_db: f64) -> String {
    format!(
        "{} · {}",
        format_frequency(frequency_hz),
        format_db(level_db)
    )
}

#[cfg(test)]
mod tests {
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
        assert_eq!(alignment_instruction(-1.5, false), "Retarder A de 1,50 ms");
        assert_eq!(
            alignment_instruction(2.0, true),
            "Retarder B de 2,00 ms · B inversée"
        );
        assert_eq!(alignment_instruction(0.0, false), ALIGNMENT_NO_DELAY);
    }

    #[test]
    fn settings_values_are_compact() {
        assert_eq!(format_fft_length(32_768), "32k");
        assert_eq!(format_seconds(0.5), "0,5 s");
        assert_eq!(format_seconds(2.0), "2 s");
        assert_eq!(format_smoothing(0), SMOOTHING_NONE);
        assert_eq!(format_smoothing(48), "1/48");
    }
}
