use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Context;
use audio_io::{AudioRoute, ReferenceMode, StreamSettings};
use dsp_core::{Averaging, DEFAULT_AIR_TEMPERATURE_CELSIUS, Smoothing};
use engine::{AnalysisSettings, DelayPolicy, EngineConfig};
use serde::{Deserialize, Serialize};

use crate::design::Theme;
use crate::plot::{DEFAULT_UPPER_PANE_FRACTION, PlotLayout, sanitized_upper_fraction};

pub const SMOOTHING_CHOICES: [u32; 6] = [0, 48, 24, 12, 6, 3];
const CONFIG_FILE_NAME: &str = "config.json";
const APP_DIRECTORY_NAME: &str = "SoundTuning";
const CONFIG_DIRECTORY_OVERRIDE_VARIABLE: &str = "SOUNDTUNING_CONFIG_DIR";
const DEFAULT_SMOOTHING_FRACTION: u32 = 48;
const MIN_AIR_TEMPERATURE_CELSIUS: f64 = -30.0;
const MAX_AIR_TEMPERATURE_CELSIUS: f64 = 50.0;
const MIN_GENERATOR_LEVEL_DBFS: f32 = -60.0;
const MAX_GENERATOR_LEVEL_DBFS: f32 = 0.0;
const CURRENT_CONFIG_VERSION: u32 = 1;
const DEFAULT_DEVICE_KEY: &str = "default";
const LEGACY_DEFAULT_GENERATOR_LEVEL_DBFS: f32 = -20.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StoredAnalysis {
    pub fft_length: usize,
    pub hop: usize,
    pub time_constant_s: Option<f64>,
    pub smoothing_fraction: u32,
    pub coherence_threshold: f64,
}

impl Default for StoredAnalysis {
    fn default() -> Self {
        Self::from_settings(AnalysisSettings::default())
    }
}

impl StoredAnalysis {
    pub fn from_settings(settings: AnalysisSettings) -> Self {
        Self {
            fft_length: settings.fft_length,
            hop: settings.hop,
            time_constant_s: match settings.averaging {
                Averaging::Exponential { time_constant_s } => Some(time_constant_s),
                Averaging::Infinite => None,
            },
            smoothing_fraction: match settings.smoothing {
                Smoothing::None => 0,
                Smoothing::OctaveFraction(fraction) => fraction,
            },
            coherence_threshold: settings.coherence_threshold,
        }
    }

    pub fn to_settings(&self) -> AnalysisSettings {
        AnalysisSettings {
            fft_length: self.fft_length,
            hop: self.hop,
            averaging: match self.time_constant_s {
                Some(time_constant_s) => Averaging::Exponential { time_constant_s },
                None => Averaging::Infinite,
            },
            smoothing: match self.smoothing_fraction {
                0 => Smoothing::None,
                fraction => Smoothing::OctaveFraction(fraction),
            },
            coherence_threshold: self.coherence_threshold,
        }
        .normalized()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameRateCap {
    #[default]
    Sixty,
    Thirty,
}

impl FrameRateCap {
    pub const ALL: [Self; 2] = [Self::Sixty, Self::Thirty];

    pub fn frames_per_second(self) -> u32 {
        match self {
            Self::Sixty => 60,
            Self::Thirty => 30,
        }
    }

    pub fn frame_interval(self) -> std::time::Duration {
        std::time::Duration::from_secs(1) / self.frames_per_second()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub theme: Theme,
    pub wizard_completed: bool,
    pub stream: StreamSettings,
    pub analysis: StoredAnalysis,
    pub delay_tracking: bool,
    pub hidden_clock_warning_routes: BTreeSet<String>,
    pub air_temperature_celsius: f64,
    pub side_panel_open: bool,
    pub plot_layout: PlotLayout,
    pub upper_pane_fraction: f32,
    #[serde(default = "legacy_config_version")]
    pub config_version: u32,
    pub frame_rate_cap: FrameRateCap,
}

fn clock_warning_key(route: &AudioRoute) -> String {
    format!(
        "{}|{}",
        route
            .input_device_id
            .as_deref()
            .unwrap_or(DEFAULT_DEVICE_KEY),
        route
            .output_device_id
            .as_deref()
            .unwrap_or(DEFAULT_DEVICE_KEY)
    )
}

fn legacy_config_version() -> u32 {
    0
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Night,
            wizard_completed: false,
            stream: StreamSettings::default(),
            analysis: StoredAnalysis {
                smoothing_fraction: DEFAULT_SMOOTHING_FRACTION,
                ..StoredAnalysis::default()
            },
            delay_tracking: false,
            hidden_clock_warning_routes: BTreeSet::new(),
            air_temperature_celsius: DEFAULT_AIR_TEMPERATURE_CELSIUS,
            side_panel_open: true,
            plot_layout: PlotLayout::default(),
            upper_pane_fraction: DEFAULT_UPPER_PANE_FRACTION,
            config_version: CURRENT_CONFIG_VERSION,
            frame_rate_cap: FrameRateCap::default(),
        }
    }
}

impl AppConfig {
    pub fn sanitized(mut self) -> Self {
        let defaults = Self::default();
        if self.config_version < CURRENT_CONFIG_VERSION {
            if self.stream.generator.level_dbfs == LEGACY_DEFAULT_GENERATOR_LEVEL_DBFS {
                self.stream.generator.level_dbfs = defaults.stream.generator.level_dbfs;
            }
            self.config_version = CURRENT_CONFIG_VERSION;
        }
        self.analysis = StoredAnalysis::from_settings(self.analysis.to_settings());
        if !audio_io::STANDARD_SAMPLE_RATES_HZ.contains(&self.stream.sample_rate_hz) {
            self.stream.sample_rate_hz = defaults.stream.sample_rate_hz;
        }
        if !self.air_temperature_celsius.is_finite() {
            self.air_temperature_celsius = defaults.air_temperature_celsius;
        }
        self.air_temperature_celsius = self
            .air_temperature_celsius
            .clamp(MIN_AIR_TEMPERATURE_CELSIUS, MAX_AIR_TEMPERATURE_CELSIUS);
        let level = self.stream.generator.level_dbfs;
        self.stream.generator.level_dbfs = if level.is_finite() {
            level.clamp(MIN_GENERATOR_LEVEL_DBFS, MAX_GENERATOR_LEVEL_DBFS)
        } else {
            defaults.stream.generator.level_dbfs
        };
        if self.stream.generator.output_channel_indices.is_empty() {
            self.stream.generator.output_channel_indices = vec![0];
        }
        self.upper_pane_fraction = sanitized_upper_fraction(self.upper_pane_fraction);
        self
    }

    pub fn configured_route(&self) -> AudioRoute {
        AudioRoute {
            input_device_id: self.stream.input_device_id.clone(),
            output_device_id: self
                .stream
                .output_device_id
                .clone()
                .or_else(|| self.stream.input_device_id.clone()),
            sample_rate_hz: self.stream.sample_rate_hz,
        }
    }

    pub fn is_clock_warning_hidden(&self, route: &AudioRoute) -> bool {
        self.hidden_clock_warning_routes
            .contains(&clock_warning_key(route))
    }

    pub fn hide_clock_warning(&mut self, route: &AudioRoute) {
        self.hidden_clock_warning_routes
            .insert(clock_warning_key(route));
    }

    pub fn uses_internal_reference(&self) -> bool {
        matches!(self.stream.reference, ReferenceMode::Internal)
    }

    pub fn delay_policy(&self) -> DelayPolicy {
        DelayPolicy {
            tracking: self.delay_tracking,
            ..DelayPolicy::default()
        }
    }

    pub fn engine_config(&self) -> EngineConfig {
        EngineConfig {
            settings: self.analysis.to_settings(),
            policy: self.delay_policy(),
            system_latency_ms: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConfigStore {
    Memory,
    File(PathBuf),
}

impl ConfigStore {
    pub fn default_location() -> Self {
        match default_config_directory() {
            Some(directory) => Self::File(directory.join(CONFIG_FILE_NAME)),
            None => Self::Memory,
        }
    }

    pub fn load(&self) -> AppConfig {
        match self {
            Self::Memory => AppConfig::default(),
            Self::File(path) => match read_config(path) {
                Ok(config) => config.sanitized(),
                Err(error) => {
                    tracing::warn!(%error, path = %path.display(), "using default configuration");
                    let backup = path.with_extension("json.bak");
                    if let Err(rename_error) = std::fs::rename(path, &backup) {
                        tracing::warn!(%rename_error, "could not keep a backup of the invalid configuration");
                    }
                    AppConfig::default()
                }
            },
        }
    }

    pub fn save(&self, config: &AppConfig) -> anyhow::Result<()> {
        match self {
            Self::Memory => Ok(()),
            Self::File(path) => write_json_atomically(path, config),
        }
    }
}

fn read_config(path: &Path) -> anyhow::Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

pub fn write_json_atomically<T: serde::Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let text = serde_json::to_vec_pretty(value).context("serializing")?;
    write_bytes_atomically(path, &text)
}

pub fn write_bytes_atomically(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let temporary = path.with_extension("json.tmp");
    let mut file = std::fs::File::create(&temporary)
        .with_context(|| format!("creating {}", temporary.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("writing {}", temporary.display()))?;
    file.sync_all()
        .with_context(|| format!("syncing {}", temporary.display()))?;
    drop(file);
    std::fs::rename(&temporary, path).with_context(|| format!("replacing {}", path.display()))
}

pub fn default_config_directory() -> Option<PathBuf> {
    if let Some(directory) = std::env::var_os(CONFIG_DIRECTORY_OVERRIDE_VARIABLE) {
        return Some(PathBuf::from(directory));
    }
    if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join(APP_DIRECTORY_NAME)
        })
    } else if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join(APP_DIRECTORY_NAME))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .map(|base| base.join(APP_DIRECTORY_NAME))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_analysis_round_trips_engine_settings() {
        let settings = AnalysisSettings {
            fft_length: 16_384,
            hop: 1024,
            averaging: Averaging::Infinite,
            smoothing: Smoothing::OctaveFraction(6),
            coherence_threshold: 0.7,
        };
        assert_eq!(
            StoredAnalysis::from_settings(settings).to_settings(),
            settings
        );
    }

    #[test]
    fn config_survives_a_file_round_trip() {
        let path = std::env::temp_dir()
            .join(format!("soundtuning-config-{}", std::process::id()))
            .join(CONFIG_FILE_NAME);
        let store = ConfigStore::File(path.clone());
        let mut config = AppConfig {
            theme: Theme::Daylight,
            wizard_completed: true,
            ..AppConfig::default()
        };
        config.hide_clock_warning(&config.configured_route());
        store.save(&config).unwrap();
        assert_eq!(store.load(), config);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn corrupted_file_falls_back_to_defaults() {
        let directory =
            std::env::temp_dir().join(format!("soundtuning-bad-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join(CONFIG_FILE_NAME);
        std::fs::write(&path, "{ not json").unwrap();
        assert_eq!(ConfigStore::File(path).load(), AppConfig::default());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn the_engine_never_receives_a_system_latency() {
        let mut config = AppConfig::default();
        assert_eq!(config.engine_config().system_latency_ms, None);
        config.stream.reference = ReferenceMode::Loopback {
            reference_channel_index: 1,
        };
        assert_eq!(config.engine_config().system_latency_ms, None);
    }

    fn legacy_config_json(level_dbfs: f32) -> serde_json::Value {
        let mut json = serde_json::to_value(AppConfig::default()).unwrap();
        let object = json.as_object_mut().unwrap();
        object.remove("config_version");
        object.insert(
            "system_latency_ms".into(),
            serde_json::json!({"in|out|48000|auto": 7.3}),
        );
        json["stream"]["generator"]["level_dbfs"] = serde_json::json!(level_dbfs);
        json
    }

    #[test]
    fn an_old_config_moves_to_the_new_generator_default_and_drops_latencies() {
        let old: AppConfig = serde_json::from_value(legacy_config_json(-20.0)).unwrap();
        assert_eq!(old.config_version, 0);
        let migrated = old.sanitized();
        assert_eq!(
            migrated.stream.generator.level_dbfs,
            AppConfig::default().stream.generator.level_dbfs
        );
        assert_eq!(migrated.config_version, CURRENT_CONFIG_VERSION);
    }

    #[test]
    fn a_level_chosen_by_the_user_survives_the_migration() {
        let old: AppConfig = serde_json::from_value(legacy_config_json(-12.0)).unwrap();
        assert_eq!(old.sanitized().stream.generator.level_dbfs, -12.0);
    }

    #[test]
    fn a_hidden_clock_warning_only_applies_to_its_pair_of_devices() {
        let mut config = AppConfig::default();
        let mac = AudioRoute {
            input_device_id: Some("coreaudio:mic".into()),
            output_device_id: Some("coreaudio:speakers".into()),
            sample_rate_hz: 48_000,
        };
        let other = AudioRoute {
            output_device_id: Some("coreaudio:headphones".into()),
            ..mac.clone()
        };
        config.hide_clock_warning(&mac);
        assert!(config.is_clock_warning_hidden(&mac));
        assert!(!config.is_clock_warning_hidden(&other));
        assert!(config.is_clock_warning_hidden(&AudioRoute {
            sample_rate_hz: 44_100,
            ..mac
        }));
    }

    #[test]
    fn the_generator_can_reach_full_scale() {
        let mut config = AppConfig::default();
        config.stream.generator.level_dbfs = 3.0;
        assert_eq!(config.sanitized().stream.generator.level_dbfs, 0.0);
    }

    #[test]
    fn missing_fields_take_defaults() {
        let config: AppConfig = serde_json::from_str(r#"{"theme":"Daylight"}"#).unwrap();
        assert_eq!(config.theme, Theme::Daylight);
        assert!(!config.wizard_completed);
        assert_eq!(config.stream.sample_rate_hz, 48_000);
        assert_eq!(config.frame_rate_cap, FrameRateCap::Sixty);
    }

    #[test]
    fn frame_rate_caps_never_exceed_sixty_frames_per_second() {
        let fastest = std::time::Duration::from_secs(1) / 60;
        for cap in FrameRateCap::ALL {
            assert!(cap.frames_per_second() <= 60);
            assert!(cap.frame_interval() >= fastest);
        }
        assert_eq!(FrameRateCap::default(), FrameRateCap::Sixty);
    }

    #[test]
    fn frame_rate_cap_survives_a_json_round_trip() {
        let config = AppConfig {
            frame_rate_cap: FrameRateCap::Thirty,
            ..AppConfig::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.frame_rate_cap, FrameRateCap::Thirty);
    }
}
