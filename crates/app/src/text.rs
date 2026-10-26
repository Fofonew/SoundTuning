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

pub const METER_REFERENCE: &str = "Référence";
pub const METER_MEASUREMENT: &str = "Mesure";
pub const METER_CLIP: &str = "ÉCRÊTAGE";
pub const BUTTON_START: &str = "Démarrer";
pub const BUTTON_STOP: &str = "Arrêter";
pub const STATUS_STOPPED: &str = "Arrêté";
pub const STATUS_RUNNING_LIVE: &str = "Mesure en cours";
pub const STATUS_RUNNING_FILE: &str = "Lecture du fichier";
pub const STATUS_DEVICE_DISCONNECTED: &str = "Carte son déconnectée";
pub const STATUS_PERMISSION_DENIED: &str = "Accès au micro refusé : autorisez SoundTuning dans Réglages Système › Confidentialité › Micro.";
pub const STATUS_SILENT_INPUT: &str =
    "Entrée entièrement muette : vérifiez l'autorisation micro de macOS et le câblage.";

pub fn format_decimal(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}").replace('.', ",")
}

pub fn format_dbfs(value: f32) -> String {
    format!("{} dBFS", format_decimal(f64::from(value), 1))
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
        "{name} — {} kHz",
        format_decimal(f64::from(sample_rate_hz) / 1000.0, 1)
    )
}

pub fn audio_error(error: &str) -> String {
    format!("Erreur audio : {error}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimals_use_french_comma() {
        assert_eq!(format_decimal(12.4321, 2), "12,43");
        assert_eq!(format_dbfs(-3.0), "-3,0 dBFS");
    }

    #[test]
    fn source_label_shows_khz() {
        assert_eq!(source_label("UAX 2-2", 48_000), "UAX 2-2 — 48,0 kHz");
    }
}
