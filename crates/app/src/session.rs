use std::path::{Path, PathBuf};

use anyhow::Context;
use dsp_core::{Complex64, power_average_db};
use engine::MeasurementSnapshot;
use serde::{Deserialize, Serialize};

use crate::design::tokens::STORED_TRACE_COLOR_COUNT;
use crate::text;

const SESSION_FILE_NAME: &str = "session.json";
const MAX_UNDO_STEPS: usize = 100;
const MAX_TRACES: usize = 64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceMetadata {
    pub captured_at_unix_s: i64,
    pub sample_rate_hz: u32,
    pub fft_length: usize,
    pub smoothing_fraction: u32,
    pub averages: u64,
    pub measured_delay_ms: Option<f64>,
    pub applied_compensation_ms: f64,
    pub system_latency_ms: Option<f64>,
}

impl TraceMetadata {
    pub fn arrival_time_ms(&self) -> f64 {
        self.applied_compensation_ms - self.system_latency_ms.unwrap_or(0.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub id: u64,
    pub name: String,
    pub color_index: usize,
    pub visible: bool,
    pub selected: bool,
    pub offset_db: f64,
    pub magnitude_db: Vec<Option<f64>>,
    pub coherence: Vec<Option<f64>>,
    pub response: Vec<Option<[f64; 2]>>,
    pub metadata: TraceMetadata,
}

impl Trace {
    pub fn displayed_db(&self) -> Vec<Option<f64>> {
        self.magnitude_db
            .iter()
            .map(|value| value.map(|level| level + self.offset_db))
            .collect()
    }

    pub fn complex_response(&self) -> Vec<Option<Complex64>> {
        let gain = 10_f64.powf(self.offset_db / 20.0);
        self.response
            .iter()
            .map(|value| value.map(|[real, imaginary]| Complex64::new(real, imaginary) * gain))
            .collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Session {
    pub traces: Vec<Trace>,
    pub next_trace_number: u64,
    pub alignment: crate::alignment_state::AlignmentState,
    pub eq: crate::eq_state::EqState,
}

impl Session {
    pub fn capture(
        &mut self,
        snapshot: &MeasurementSnapshot,
        smoothing_fraction: u32,
        captured_at_unix_s: i64,
        clock_label: &str,
    ) -> Option<u64> {
        if snapshot.averages == 0 || self.traces.len() >= MAX_TRACES {
            return None;
        }
        self.next_trace_number += 1;
        let id = self.next_trace_number;
        let color_index = (id as usize - 1) % STORED_TRACE_COLOR_COUNT;
        self.traces.push(Trace {
            id,
            name: text::trace_name(id, clock_label),
            color_index,
            visible: true,
            selected: false,
            offset_db: 0.0,
            magnitude_db: snapshot.magnitude_db.clone(),
            coherence: snapshot.coherence.clone(),
            response: snapshot
                .response
                .iter()
                .map(|value| value.map(|value| [value.re, value.im]))
                .collect(),
            metadata: TraceMetadata {
                captured_at_unix_s,
                sample_rate_hz: snapshot.sample_rate_hz,
                fft_length: snapshot.settings.fft_length,
                smoothing_fraction,
                averages: snapshot.averages,
                measured_delay_ms: snapshot.delay.latest_confident.map(|delay| delay.total_ms),
                applied_compensation_ms: snapshot.delay.applied_ms,
                system_latency_ms: snapshot.delay.system_latency_ms,
            },
        });
        Some(id)
    }

    pub fn trace(&self, id: u64) -> Option<&Trace> {
        self.traces.iter().find(|trace| trace.id == id)
    }

    pub fn trace_mut(&mut self, id: u64) -> Option<&mut Trace> {
        self.traces.iter_mut().find(|trace| trace.id == id)
    }

    pub fn remove(&mut self, id: u64) {
        self.traces.retain(|trace| trace.id != id);
        self.alignment.forget_trace(id);
        self.eq.forget_trace(id);
    }

    pub fn average_selected(&mut self, captured_at_unix_s: i64, clock_label: &str) -> Option<u64> {
        let selected: Vec<&Trace> = self.traces.iter().filter(|trace| trace.selected).collect();
        if selected.len() < 2 || self.traces.len() >= MAX_TRACES {
            return None;
        }
        let first = selected.first()?;
        let grid_len = first.magnitude_db.len();
        if selected
            .iter()
            .any(|trace| trace.magnitude_db.len() != grid_len)
        {
            return None;
        }
        let displayed: Vec<Vec<Option<f64>>> =
            selected.iter().map(|trace| trace.displayed_db()).collect();
        let references: Vec<&[Option<f64>]> = displayed.iter().map(Vec::as_slice).collect();
        let mut magnitude_db = vec![None; grid_len];
        power_average_db(&references, &mut magnitude_db).ok()?;
        let coherence = (0..grid_len)
            .map(|index| {
                selected
                    .iter()
                    .filter_map(|trace| trace.coherence.get(index).copied().flatten())
                    .reduce(f64::min)
            })
            .collect();
        let metadata = TraceMetadata {
            captured_at_unix_s,
            measured_delay_ms: None,
            ..first.metadata.clone()
        };
        let member_names: Vec<String> = selected.iter().map(|trace| trace.name.clone()).collect();
        self.next_trace_number += 1;
        let id = self.next_trace_number;
        self.traces.push(Trace {
            id,
            name: text::average_trace_name(id, clock_label, member_names.len()),
            color_index: (id as usize - 1) % STORED_TRACE_COLOR_COUNT,
            visible: true,
            selected: false,
            offset_db: 0.0,
            magnitude_db,
            coherence,
            response: vec![None; grid_len],
            metadata,
        });
        Some(id)
    }

    pub fn to_csv(&self, frequencies_hz: &[f64]) -> String {
        let mut csv = String::from(text::CSV_FREQUENCY_HEADER);
        for trace in &self.traces {
            csv.push(';');
            csv.push_str(&csv_field(&trace.name));
            csv.push(';');
            csv.push_str(&csv_field(&text::csv_coherence_header(&trace.name)));
        }
        csv.push('\n');
        for (index, frequency_hz) in frequencies_hz.iter().enumerate() {
            csv.push_str(&format!("{frequency_hz:.3}"));
            for trace in &self.traces {
                let level = trace
                    .displayed_db()
                    .get(index)
                    .copied()
                    .flatten()
                    .map(|value| format!("{value:.3}"))
                    .unwrap_or_default();
                let coherence = trace
                    .coherence
                    .get(index)
                    .copied()
                    .flatten()
                    .map(|value| format!("{value:.4}"))
                    .unwrap_or_default();
                csv.push(';');
                csv.push_str(&level);
                csv.push(';');
                csv.push_str(&coherence);
            }
            csv.push('\n');
        }
        csv
    }
}

fn csv_field(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

#[derive(Clone, Debug, Default)]
pub struct UndoHistory {
    past: Vec<Session>,
    future: Vec<Session>,
}

impl UndoHistory {
    pub fn record(&mut self, before: Session) {
        self.past.push(before);
        if self.past.len() > MAX_UNDO_STEPS {
            self.past.remove(0);
        }
        self.future.clear();
    }

    pub fn undo(&mut self, current: &mut Session) -> bool {
        match self.past.pop() {
            Some(previous) => {
                self.future.push(std::mem::replace(current, previous));
                true
            }
            None => false,
        }
    }

    pub fn redo(&mut self, current: &mut Session) -> bool {
        match self.future.pop() {
            Some(next) => {
                self.past.push(std::mem::replace(current, next));
                true
            }
            None => false,
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SessionStore {
    Memory,
    File(PathBuf),
}

impl SessionStore {
    pub fn default_location() -> Self {
        match crate::config::default_config_directory() {
            Some(directory) => Self::File(directory.join(SESSION_FILE_NAME)),
            None => Self::Memory,
        }
    }

    pub fn load(&self) -> Session {
        match self {
            Self::Memory => Session::default(),
            Self::File(path) => read_session(path).unwrap_or_else(|error| {
                tracing::warn!(%error, path = %path.display(), "starting with an empty session");
                Session::default()
            }),
        }
    }

    pub fn save(&self, session: &Session) -> anyhow::Result<()> {
        match self {
            Self::Memory => Ok(()),
            Self::File(path) => crate::config::write_json_atomically(path, session),
        }
    }
}

fn read_session(path: &Path) -> anyhow::Result<Session> {
    if !path.exists() {
        return Ok(Session::default());
    }
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use engine::AnalysisSettings;

    use super::*;

    fn snapshot(level_db: f64) -> MeasurementSnapshot {
        let mut snapshot = MeasurementSnapshot::empty(4, 48_000, AnalysisSettings::default());
        snapshot.averages = 10;
        snapshot.magnitude_db = vec![Some(level_db), Some(level_db), None, Some(level_db)];
        snapshot.coherence = vec![Some(0.9); 4];
        snapshot.response = vec![Some(Complex64::new(1.0, 0.0)); 4];
        snapshot
    }

    #[test]
    fn capture_names_and_colors_traces_in_order() {
        let mut session = Session::default();
        let first = session.capture(&snapshot(0.0), 48, 0, "21:42").unwrap();
        let second = session.capture(&snapshot(1.0), 48, 0, "21:43").unwrap();
        assert_eq!(session.trace(first).unwrap().name, "Trace 1 — 21:42");
        assert_eq!(session.trace(second).unwrap().color_index, 1);
    }

    #[test]
    fn nothing_is_captured_before_the_first_average() {
        let mut session = Session::default();
        let mut empty = snapshot(0.0);
        empty.averages = 0;
        assert_eq!(session.capture(&empty, 48, 0, "21:42"), None);
    }

    #[test]
    fn undo_and_redo_restore_sessions() {
        let mut session = Session::default();
        let mut history = UndoHistory::default();
        history.record(session.clone());
        session.capture(&snapshot(0.0), 48, 0, "21:42");
        assert_eq!(session.traces.len(), 1);
        assert!(history.undo(&mut session));
        assert!(session.traces.is_empty());
        assert!(history.redo(&mut session));
        assert_eq!(session.traces.len(), 1);
        assert!(!history.redo(&mut session));
    }

    #[test]
    fn average_of_identical_selected_traces_is_identical() {
        let mut session = Session::default();
        for _ in 0..2 {
            let id = session.capture(&snapshot(-3.0), 48, 0, "21:42").unwrap();
            session.trace_mut(id).unwrap().selected = true;
        }
        let average = session.average_selected(0, "21:44").unwrap();
        let trace = session.trace(average).unwrap();
        for (value, expected) in
            trace
                .magnitude_db
                .iter()
                .zip([Some(-3.0), Some(-3.0), None, Some(-3.0)])
        {
            match (value, expected) {
                (Some(value), Some(expected)) => assert!((value - expected).abs() < 0.01),
                (None, None) => {}
                mismatch => panic!("{mismatch:?}"),
            }
        }
    }

    #[test]
    fn offsets_shift_displayed_levels_and_csv_export_has_all_traces() {
        let mut session = Session::default();
        let id = session.capture(&snapshot(0.0), 48, 0, "21:42").unwrap();
        session.trace_mut(id).unwrap().offset_db = 6.0;
        assert_eq!(session.trace(id).unwrap().displayed_db()[0], Some(6.0));
        let csv = session.to_csv(&[100.0, 200.0, 300.0, 400.0]);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 5);
        assert!(lines[0].contains("Trace 1 — 21:42"));
        assert!(lines[1].starts_with("100.000;6.000;0.9000"));
        assert!(lines[3].contains(";;"));
    }

    #[test]
    fn session_survives_a_json_round_trip() {
        let mut session = Session::default();
        session.capture(&snapshot(1.5), 24, 1_700_000_000, "21:42");
        let json = serde_json::to_string(&session).unwrap();
        let restored: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, session);
    }

    #[test]
    fn arrival_time_combines_compensation_and_latency() {
        let metadata = TraceMetadata {
            captured_at_unix_s: 0,
            sample_rate_hz: 48_000,
            fft_length: 32_768,
            smoothing_fraction: 48,
            averages: 1,
            measured_delay_ms: Some(10.0),
            applied_compensation_ms: 10.0,
            system_latency_ms: Some(7.3),
        };
        assert!((metadata.arrival_time_ms() - 2.7).abs() < 1.0e-9);
    }
}
