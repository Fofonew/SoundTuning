use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread::JoinHandle;

use anyhow::Context;
use dsp_core::{Complex64, power_average_db};
use engine::MeasurementSnapshot;
use serde::{Deserialize, Serialize};

use crate::design::tokens::STORED_TRACE_COLOR_COUNT;
use crate::text;

const SESSION_FILE_NAME: &str = "session.json";
const MAX_UNDO_STEPS: usize = 100;
const MAX_TRACES: usize = 64;
const WRITE_QUEUE_DEPTH: usize = 1;
const BACKUP_EXTENSION: &str = "json.bak";
const CORRUPT_FILE_PREFIX: &str = "session.corrupt-";

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureError {
    NoAverages,
    TooManyTraces,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AverageError {
    NeedTwoTraces,
    TooManyTraces,
    IncompatibleTraces,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Session {
    pub traces: Vec<Arc<Trace>>,
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
    ) -> Result<u64, CaptureError> {
        if snapshot.averages == 0 {
            return Err(CaptureError::NoAverages);
        }
        if self.traces.len() >= MAX_TRACES {
            return Err(CaptureError::TooManyTraces);
        }
        self.next_trace_number += 1;
        let id = self.next_trace_number;
        self.traces.push(Arc::new(Trace {
            id,
            name: text::trace_name(id, clock_label),
            color_index: color_for(id),
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
        }));
        Ok(id)
    }

    pub fn trace(&self, id: u64) -> Option<&Trace> {
        self.traces
            .iter()
            .find(|trace| trace.id == id)
            .map(Arc::as_ref)
    }

    pub fn trace_mut(&mut self, id: u64) -> Option<&mut Trace> {
        self.traces
            .iter_mut()
            .find(|trace| trace.id == id)
            .map(Arc::make_mut)
    }

    pub fn remove(&mut self, id: u64) {
        self.traces.retain(|trace| trace.id != id);
        self.alignment.forget_trace(id);
        self.eq.forget_trace(id);
    }

    pub fn average_selected(
        &mut self,
        captured_at_unix_s: i64,
        clock_label: &str,
    ) -> Result<u64, AverageError> {
        let selected: Vec<&Trace> = self
            .traces
            .iter()
            .filter(|trace| trace.selected)
            .map(Arc::as_ref)
            .collect();
        let Some(first) = selected.first().filter(|_| selected.len() >= 2) else {
            return Err(AverageError::NeedTwoTraces);
        };
        if self.traces.len() >= MAX_TRACES {
            return Err(AverageError::TooManyTraces);
        }
        let grid_len = first.magnitude_db.len();
        if selected.iter().any(|trace| {
            trace.magnitude_db.len() != grid_len
                || trace.metadata.sample_rate_hz != first.metadata.sample_rate_hz
        }) {
            return Err(AverageError::IncompatibleTraces);
        }
        let displayed: Vec<Vec<Option<f64>>> =
            selected.iter().map(|trace| trace.displayed_db()).collect();
        let references: Vec<&[Option<f64>]> = displayed.iter().map(Vec::as_slice).collect();
        let mut magnitude_db = vec![None; grid_len];
        power_average_db(&references, &mut magnitude_db)
            .map_err(|_| AverageError::IncompatibleTraces)?;
        let coherence = (0..grid_len)
            .map(|index| {
                selected
                    .iter()
                    .filter_map(|trace| trace.coherence.get(index).copied().flatten())
                    .reduce(f64::min)
            })
            .collect();
        let same_compensation = selected.iter().all(|trace| {
            (trace.metadata.applied_compensation_ms - first.metadata.applied_compensation_ms).abs()
                < f64::EPSILON
        });
        let metadata = TraceMetadata {
            captured_at_unix_s,
            measured_delay_ms: None,
            averages: selected.iter().map(|trace| trace.metadata.averages).sum(),
            applied_compensation_ms: if same_compensation {
                first.metadata.applied_compensation_ms
            } else {
                0.0
            },
            ..first.metadata.clone()
        };
        let member_count = selected.len();
        self.next_trace_number += 1;
        let id = self.next_trace_number;
        self.traces.push(Arc::new(Trace {
            id,
            name: text::average_trace_name(id, clock_label, member_count),
            color_index: color_for(id),
            visible: true,
            selected: false,
            offset_db: 0.0,
            magnitude_db,
            coherence,
            response: vec![None; grid_len],
            metadata,
        }));
        Ok(id)
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
        let displayed: Vec<Vec<Option<f64>>> = self
            .traces
            .iter()
            .map(|trace| trace.displayed_db())
            .collect();
        for (index, frequency_hz) in frequencies_hz.iter().enumerate() {
            csv.push_str(&text::format_csv_number(*frequency_hz, 3));
            for (trace, levels) in self.traces.iter().zip(&displayed) {
                csv.push(';');
                if let Some(level) = levels.get(index).copied().flatten() {
                    csv.push_str(&text::format_csv_number(level, 3));
                }
                csv.push(';');
                if let Some(coherence) = trace.coherence.get(index).copied().flatten() {
                    csv.push_str(&text::format_csv_number(coherence, 4));
                }
            }
            csv.push('\n');
        }
        csv
    }
}

fn color_for(id: u64) -> usize {
    (id.saturating_sub(1) % STORED_TRACE_COLOR_COUNT as u64) as usize
}

fn csv_field(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

#[derive(Clone, Debug, Default)]
pub struct UndoHistory {
    past: VecDeque<Session>,
    future: Vec<Session>,
}

impl UndoHistory {
    pub fn record(&mut self, before: Session) {
        self.past.push_back(before);
        if self.past.len() > MAX_UNDO_STEPS {
            self.past.pop_front();
        }
        self.future.clear();
    }

    pub fn undo(&mut self, current: &mut Session) -> bool {
        match self.past.pop_back() {
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
                self.past.push_back(std::mem::replace(current, next));
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

#[derive(Clone, Debug, PartialEq)]
pub enum LoadIssue {
    Corrupt {
        moved_to: Option<PathBuf>,
        restored_backup: bool,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoadedSession {
    pub session: Session,
    pub issue: Option<LoadIssue>,
}

impl SessionStore {
    pub fn default_location() -> Self {
        match crate::config::default_config_directory() {
            Some(directory) => Self::File(directory.join(SESSION_FILE_NAME)),
            None => Self::Memory,
        }
    }

    pub fn load(&self) -> LoadedSession {
        match self {
            Self::Memory => LoadedSession::default(),
            Self::File(path) => load_session_file(path),
        }
    }

    pub fn save(&self, session: &Session) -> anyhow::Result<()> {
        match self {
            Self::Memory => Ok(()),
            Self::File(path) => {
                let bytes = serde_json::to_vec(session).context("serializing the session")?;
                if path.exists() {
                    std::fs::rename(path, backup_path(path))
                        .with_context(|| format!("backing up {}", path.display()))?;
                }
                crate::config::write_bytes_atomically(path, &bytes)
            }
        }
    }
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension(BACKUP_EXTENSION)
}

fn corrupt_path(path: &Path) -> PathBuf {
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    path.with_file_name(format!("{CORRUPT_FILE_PREFIX}{stamp}.json"))
}

fn load_session_file(path: &Path) -> LoadedSession {
    let backup = backup_path(path);
    if !path.exists() {
        let session = read_session(&backup).ok().flatten().unwrap_or_default();
        return LoadedSession {
            session,
            issue: None,
        };
    }
    match read_session(path) {
        Ok(Some(session)) => LoadedSession {
            session,
            issue: None,
        },
        Ok(None) => LoadedSession::default(),
        Err(error) => {
            tracing::warn!(%error, path = %path.display(), "session file is unreadable");
            let target = corrupt_path(path);
            let moved_to = match std::fs::rename(path, &target) {
                Ok(()) => Some(target),
                Err(error) => {
                    tracing::warn!(%error, "could not set the unreadable session aside");
                    None
                }
            };
            let restored = read_session(&backup).ok().flatten();
            LoadedSession {
                issue: Some(LoadIssue::Corrupt {
                    moved_to,
                    restored_backup: restored.is_some(),
                }),
                session: restored.unwrap_or_default(),
            }
        }
    }
}

fn read_session(path: &Path) -> anyhow::Result<Option<Session>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .with_context(|| format!("parsing {}", path.display()))
}

pub struct SessionWriter {
    store: SessionStore,
    requests: Option<SyncSender<Session>>,
    results: Receiver<Result<(), String>>,
    worker: Option<JoinHandle<()>>,
}

impl SessionWriter {
    pub fn spawn(store: SessionStore) -> Self {
        let (request_sender, request_receiver) = mpsc::sync_channel::<Session>(WRITE_QUEUE_DEPTH);
        let (result_sender, results) = mpsc::channel();
        let worker_store = store.clone();
        let worker = std::thread::Builder::new()
            .name("session-writer".into())
            .spawn(move || {
                while let Ok(mut session) = request_receiver.recv() {
                    while let Ok(newer) = request_receiver.try_recv() {
                        session = newer;
                    }
                    let result = worker_store
                        .save(&session)
                        .map_err(|error| format!("{error:#}"));
                    if result_sender.send(result).is_err() {
                        break;
                    }
                }
            });
        match worker {
            Ok(worker) => Self {
                store,
                requests: Some(request_sender),
                results,
                worker: Some(worker),
            },
            Err(error) => {
                tracing::warn!(%error, "session writer thread unavailable, saving inline");
                Self {
                    store,
                    requests: None,
                    results,
                    worker: None,
                }
            }
        }
    }

    pub fn request(&self, session: Session) -> Option<Result<(), String>> {
        match &self.requests {
            Some(sender) => match sender.try_send(session) {
                Ok(()) => None,
                Err(TrySendError::Full(_)) => Some(Err(String::from("write queue full"))),
                Err(TrySendError::Disconnected(_)) => {
                    Some(Err(String::from("session writer stopped")))
                }
            },
            None => Some(
                self.store
                    .save(&session)
                    .map_err(|error| format!("{error:#}")),
            ),
        }
    }

    pub fn poll(&self) -> Vec<Result<(), String>> {
        self.results.try_iter().collect()
    }

    pub fn finish(&mut self, session: &Session) -> anyhow::Result<()> {
        self.requests = None;
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            tracing::warn!("session writer thread panicked");
        }
        self.store.save(session)
    }
}

impl Drop for SessionWriter {
    fn drop(&mut self) {
        self.requests = None;
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
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
        assert_eq!(
            session.capture(&empty, 48, 0, "21:42"),
            Err(CaptureError::NoAverages)
        );
    }

    #[test]
    fn undo_and_redo_restore_sessions() {
        let mut session = Session::default();
        let mut history = UndoHistory::default();
        history.record(session.clone());
        session.capture(&snapshot(0.0), 48, 0, "21:42").unwrap();
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
        assert!(lines[1].starts_with("100,000;6,000;0,9000"));
        assert!(lines[3].contains(";;"));
    }

    #[test]
    fn session_survives_a_json_round_trip() {
        let mut session = Session::default();
        session
            .capture(&snapshot(1.5), 24, 1_700_000_000, "21:42")
            .unwrap();
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

    fn temporary_directory(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "soundtuning-session-test-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn capture_is_refused_beyond_the_trace_limit() {
        let mut session = Session::default();
        for _ in 0..MAX_TRACES {
            session.capture(&snapshot(0.0), 48, 0, "21:42").unwrap();
        }
        assert_eq!(
            session.capture(&snapshot(0.0), 48, 0, "21:42"),
            Err(CaptureError::TooManyTraces)
        );
    }

    #[test]
    fn average_needs_two_compatible_traces() {
        let mut session = Session::default();
        let first = session.capture(&snapshot(0.0), 48, 0, "21:42").unwrap();
        session.trace_mut(first).unwrap().selected = true;
        assert_eq!(
            session.average_selected(0, "21:43"),
            Err(AverageError::NeedTwoTraces)
        );
        let mut other_rate = snapshot(0.0);
        other_rate.sample_rate_hz = 44_100;
        let second = session.capture(&other_rate, 48, 0, "21:42").unwrap();
        session.trace_mut(second).unwrap().selected = true;
        assert_eq!(
            session.average_selected(0, "21:43"),
            Err(AverageError::IncompatibleTraces)
        );
    }

    #[test]
    fn removing_a_trace_forgets_it_everywhere() {
        let mut session = Session::default();
        let id = session.capture(&snapshot(0.0), 48, 0, "21:42").unwrap();
        session.alignment.trace_a = Some(id);
        session.remove(id);
        assert!(session.trace(id).is_none());
        assert_eq!(session.alignment.trace_a, None);
    }

    #[test]
    fn a_new_edit_clears_the_redo_stack() {
        let mut session = Session::default();
        let mut history = UndoHistory::default();
        history.record(session.clone());
        session.capture(&snapshot(0.0), 48, 0, "21:42").unwrap();
        assert!(history.undo(&mut session));
        assert!(history.can_redo());
        history.record(session.clone());
        session.capture(&snapshot(1.0), 48, 0, "21:43").unwrap();
        assert!(!history.can_redo());
    }

    #[test]
    fn history_shares_unchanged_traces_between_steps() {
        let mut session = Session::default();
        session.capture(&snapshot(0.0), 48, 0, "21:42").unwrap();
        let before = session.clone();
        assert!(Arc::ptr_eq(&before.traces[0], &session.traces[0]));
        session.trace_mut(1).unwrap().name = String::from("Façade");
        assert!(!Arc::ptr_eq(&before.traces[0], &session.traces[0]));
        assert_eq!(before.traces[0].name, "Trace 1 — 21:42");
    }

    #[test]
    fn csv_uses_decimal_commas_and_scales_linearly() {
        let mut session = Session::default();
        let mut long = snapshot(-1.25);
        long.magnitude_db = vec![Some(-1.25); 957];
        long.coherence = vec![Some(0.5); 957];
        for _ in 0..32 {
            session.capture(&long, 48, 0, "21:42").unwrap();
        }
        let frequencies: Vec<f64> = (0..957).map(|index| 20.0 + index as f64).collect();
        let started = std::time::Instant::now();
        let csv = session.to_csv(&frequencies);
        assert!(started.elapsed() < std::time::Duration::from_millis(500));
        let second_line = csv.lines().nth(1).unwrap();
        assert!(second_line.starts_with("20,000;-1,250;0,5000"));
    }

    #[test]
    fn unreadable_session_is_set_aside_and_the_backup_restored() {
        let directory = temporary_directory("corrupt");
        let path = directory.join(SESSION_FILE_NAME);
        let store = SessionStore::File(path.clone());
        let mut session = Session::default();
        session.capture(&snapshot(0.0), 48, 0, "21:42").unwrap();
        store.save(&session).unwrap();
        session.capture(&snapshot(1.0), 48, 0, "21:43").unwrap();
        store.save(&session).unwrap();
        std::fs::write(&path, b"{ not json").unwrap();
        let loaded = store.load();
        assert_eq!(loaded.session.traces.len(), 1);
        match loaded.issue {
            Some(LoadIssue::Corrupt {
                moved_to: Some(moved_to),
                restored_backup: true,
            }) => {
                assert_eq!(std::fs::read(&moved_to).unwrap(), b"{ not json");
                assert!(
                    moved_to
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with(CORRUPT_FILE_PREFIX)
                );
            }
            other => panic!("{other:?}"),
        }
        assert!(!path.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn unreadable_session_without_backup_starts_empty() {
        let directory = temporary_directory("corrupt-empty");
        let path = directory.join(SESSION_FILE_NAME);
        std::fs::write(&path, b"\"truncated").unwrap();
        let loaded = SessionStore::File(path).load();
        assert!(loaded.session.traces.is_empty());
        assert!(matches!(
            loaded.issue,
            Some(LoadIssue::Corrupt {
                restored_backup: false,
                ..
            })
        ));
        std::fs::remove_dir_all(directory).unwrap();
    }

    fn wait_for_result(writer: &SessionWriter) -> Result<(), String> {
        let started = std::time::Instant::now();
        loop {
            if let Some(result) = writer.poll().pop() {
                return result;
            }
            assert!(started.elapsed() < std::time::Duration::from_secs(10));
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    #[test]
    fn writer_saves_off_thread_and_reports_failures() {
        let directory = temporary_directory("writer");
        let blocker = directory.join("not-a-directory");
        std::fs::write(&blocker, b"").unwrap();
        let failing = SessionWriter::spawn(SessionStore::File(blocker.join(SESSION_FILE_NAME)));
        assert_eq!(failing.request(Session::default()), None);
        assert!(wait_for_result(&failing).is_err());

        let path = directory.join(SESSION_FILE_NAME);
        let mut writer = SessionWriter::spawn(SessionStore::File(path.clone()));
        let mut session = Session::default();
        session.capture(&snapshot(0.0), 48, 0, "21:42").unwrap();
        assert_eq!(writer.request(session.clone()), None);
        assert_eq!(wait_for_result(&writer), Ok(()));
        assert_eq!(SessionStore::File(path.clone()).load().session, session);
        session.capture(&snapshot(1.0), 48, 0, "21:43").unwrap();
        writer.finish(&session).unwrap();
        assert_eq!(SessionStore::File(path).load().session.traces.len(), 2);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
