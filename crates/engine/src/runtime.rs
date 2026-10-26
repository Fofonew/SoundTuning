use std::sync::Arc;
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError, TrySendError, sync_channel};
use std::thread::JoinHandle;
use std::time::Duration;

use audio_io::{FrameSource, StereoFrame};
use dsp_core::{DEFAULT_DELAY_FFT_LENGTH, DisplayGrid};
use triple_buffer::{Input, Output, triple_buffer};

use crate::core::{EngineCommand, MeasurementCore};
use crate::delay_estimator::DelayEstimator;
use crate::error::EngineError;
use crate::settings::{AnalysisSettings, DelayPolicy};
use crate::snapshot::MeasurementSnapshot;

const COMMAND_QUEUE_CAPACITY: usize = 256;
const IDLE_WAIT: Duration = Duration::from_millis(2);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EngineConfig {
    pub settings: AnalysisSettings,
    pub policy: DelayPolicy,
    pub system_latency_ms: Option<f64>,
}

enum ControlMessage {
    Command(EngineCommand),
    Shutdown,
}

pub struct EngineHandle {
    commands: SyncSender<ControlMessage>,
    snapshots: Output<MeasurementSnapshot>,
    worker: Option<JoinHandle<()>>,
    grid: Arc<DisplayGrid>,
    sample_rate_hz: u32,
}

impl EngineHandle {
    pub fn spawn(
        source: Box<dyn FrameSource>,
        config: EngineConfig,
        grid: Arc<DisplayGrid>,
    ) -> Result<Self, EngineError> {
        let sample_rate_hz = source.sample_rate_hz();
        let mut core = MeasurementCore::new(
            sample_rate_hz,
            config.settings,
            config.policy,
            Arc::clone(&grid),
            DEFAULT_DELAY_FFT_LENGTH,
        )?;
        core.handle_command(EngineCommand::SetSystemLatencyMs(config.system_latency_ms))?;
        let delay_estimator = DelayEstimator::threaded(DEFAULT_DELAY_FFT_LENGTH)?;
        let initial = MeasurementSnapshot::empty(grid.len(), sample_rate_hz, core.settings());
        let (snapshot_input, snapshot_output) = triple_buffer(&initial);
        let (command_sender, command_receiver) = sync_channel(COMMAND_QUEUE_CAPACITY);

        let worker = std::thread::Builder::new()
            .name("measurement-engine".into())
            .spawn(move || {
                run_engine_loop(
                    source,
                    core,
                    delay_estimator,
                    command_receiver,
                    snapshot_input,
                );
            })
            .map_err(EngineError::Thread)?;

        Ok(Self {
            commands: command_sender,
            snapshots: snapshot_output,
            worker: Some(worker),
            grid,
            sample_rate_hz,
        })
    }

    pub fn send(&self, command: EngineCommand) {
        match self.commands.try_send(ControlMessage::Command(command)) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                tracing::warn!(?command, "engine command queue full; command dropped");
            }
            Err(TrySendError::Disconnected(_)) => {
                tracing::warn!(?command, "engine thread is gone; command dropped");
            }
        }
    }

    pub fn has_new_snapshot(&self) -> bool {
        self.snapshots.updated()
    }

    pub fn latest_snapshot(&mut self) -> &MeasurementSnapshot {
        self.snapshots.read()
    }

    pub fn grid(&self) -> &Arc<DisplayGrid> {
        &self.grid
    }

    pub fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    pub fn is_running(&self) -> bool {
        self.worker
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
    }
}

impl Drop for EngineHandle {
    fn drop(&mut self) {
        if self.commands.send(ControlMessage::Shutdown).is_err() {
            tracing::debug!("engine thread already stopped");
        }
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            tracing::error!("engine thread panicked");
        }
    }
}

fn run_engine_loop(
    mut source: Box<dyn FrameSource>,
    mut core: MeasurementCore,
    mut delay_estimator: DelayEstimator,
    commands: Receiver<ControlMessage>,
    mut snapshots: Input<MeasurementSnapshot>,
) {
    let mut frames: Vec<StereoFrame> = Vec::with_capacity(crate::settings::MAX_HOP_FRAMES);
    let mut source_finished = false;
    loop {
        let mut state_changed = false;
        loop {
            match commands.try_recv() {
                Ok(ControlMessage::Command(command)) => {
                    if let Err(error) = core.handle_command(command) {
                        tracing::error!(%error, ?command, "engine command failed");
                    }
                    state_changed = true;
                }
                Ok(ControlMessage::Shutdown) | Err(TryRecvError::Disconnected) => return,
                Err(TryRecvError::Empty) => break,
            }
        }

        frames.clear();
        let report = source.read_frames(&mut frames, core.settings().hop);
        if report.discontinuity {
            core.note_discontinuity();
            state_changed = true;
        }
        if report.finished != source_finished {
            source_finished = report.finished;
            core.set_source_finished(source_finished);
            state_changed = true;
        }
        state_changed |= core.push_frames(&frames, &mut delay_estimator) > 0;
        state_changed |= core.poll_delay(&mut delay_estimator);

        if state_changed {
            match core.fill_snapshot(snapshots.input_buffer_mut()) {
                Ok(()) => {
                    snapshots.publish();
                }
                Err(error) => tracing::error!(%error, "snapshot preparation failed"),
            }
        }
        if frames.is_empty() {
            std::thread::sleep(IDLE_WAIT);
        }
    }
}
