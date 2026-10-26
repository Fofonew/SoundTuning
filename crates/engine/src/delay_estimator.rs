use std::sync::mpsc::{Receiver, SyncSender, TryRecvError, sync_channel};
use std::thread::JoinHandle;

use dsp_core::{DelayEstimate, DelayFinder, DspError};

use crate::error::EngineError;

pub struct DelayJob {
    pub reference: Vec<f64>,
    pub measurement: Vec<f64>,
}

impl DelayJob {
    fn new(length: usize) -> Self {
        Self {
            reference: vec![0.0; length],
            measurement: vec![0.0; length],
        }
    }
}

type DelayOutcome = Result<Option<DelayEstimate>, DspError>;

pub enum DelayEstimator {
    Inline {
        finder: Box<DelayFinder>,
        job: Box<DelayJob>,
        pending: Option<DelayOutcome>,
    },
    Threaded {
        spare_job: Option<Box<DelayJob>>,
        jobs: Option<SyncSender<Box<DelayJob>>>,
        results: Receiver<(Box<DelayJob>, DelayOutcome)>,
        worker: Option<JoinHandle<()>>,
    },
}

impl DelayEstimator {
    pub fn inline(fft_length: usize) -> Result<Self, EngineError> {
        Ok(Self::Inline {
            finder: Box::new(DelayFinder::new(fft_length)?),
            job: Box::new(DelayJob::new(fft_length)),
            pending: None,
        })
    }

    pub fn threaded(fft_length: usize) -> Result<Self, EngineError> {
        let mut finder = DelayFinder::new(fft_length)?;
        let (job_sender, job_receiver) = sync_channel::<Box<DelayJob>>(1);
        let (result_sender, result_receiver) = sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("delay-search".into())
            .spawn(move || {
                while let Ok(job) = job_receiver.recv() {
                    let outcome = finder.estimate(&job.reference, &job.measurement);
                    if result_sender.send((job, outcome)).is_err() {
                        break;
                    }
                }
            })
            .map_err(EngineError::Thread)?;
        Ok(Self::Threaded {
            spare_job: Some(Box::new(DelayJob::new(fft_length))),
            jobs: Some(job_sender),
            results: result_receiver,
            worker: Some(worker),
        })
    }

    pub fn fft_length(&self) -> usize {
        match self {
            Self::Inline { job, .. } => job.reference.len(),
            Self::Threaded { spare_job, .. } => {
                spare_job.as_ref().map_or(0, |job| job.reference.len())
            }
        }
    }

    pub fn idle_job(&mut self) -> Option<&mut DelayJob> {
        match self {
            Self::Inline { job, pending, .. } => pending.is_none().then_some(job.as_mut()),
            Self::Threaded { spare_job, .. } => spare_job.as_deref_mut(),
        }
    }

    pub fn submit(&mut self) {
        match self {
            Self::Inline {
                finder,
                job,
                pending,
            } => {
                if pending.is_none() {
                    *pending = Some(finder.estimate(&job.reference, &job.measurement));
                }
            }
            Self::Threaded {
                spare_job, jobs, ..
            } => {
                let Some(sender) = jobs.as_ref() else {
                    return;
                };
                let Some(job) = spare_job.take() else {
                    return;
                };
                if let Err(
                    std::sync::mpsc::TrySendError::Full(job)
                    | std::sync::mpsc::TrySendError::Disconnected(job),
                ) = sender.try_send(job)
                {
                    *spare_job = Some(job);
                }
            }
        }
    }

    pub fn poll(&mut self) -> Option<DelayOutcome> {
        match self {
            Self::Inline { pending, .. } => pending.take(),
            Self::Threaded {
                spare_job, results, ..
            } => match results.try_recv() {
                Ok((job, outcome)) => {
                    *spare_job = Some(job);
                    Some(outcome)
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => None,
            },
        }
    }
}

impl Drop for DelayEstimator {
    fn drop(&mut self) {
        if let Self::Threaded { jobs, worker, .. } = self {
            jobs.take();
            if let Some(worker) = worker.take() {
                let _ = worker.join();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    fn fill_delayed_impulse(job: &mut DelayJob, delay: usize) {
        job.reference.fill(0.0);
        job.measurement.fill(0.0);
        job.reference[10] = 1.0;
        job.measurement[10 + delay] = 1.0;
    }

    #[test]
    fn inline_estimator_returns_result_on_next_poll() {
        let mut estimator = DelayEstimator::inline(1024).unwrap();
        fill_delayed_impulse(estimator.idle_job().unwrap(), 12);
        estimator.submit();
        assert!(estimator.idle_job().is_none());
        let estimate = estimator.poll().unwrap().unwrap().unwrap();
        assert!((estimate.delay_samples - 12.0).abs() < 0.01);
        assert!(estimator.idle_job().is_some());
    }

    #[test]
    fn threaded_estimator_recycles_its_job_buffer() {
        let mut estimator = DelayEstimator::threaded(1024).unwrap();
        for delay in [5, 40] {
            fill_delayed_impulse(estimator.idle_job().unwrap(), delay);
            estimator.submit();
            assert!(estimator.idle_job().is_none());
            let started = Instant::now();
            let outcome = loop {
                if let Some(outcome) = estimator.poll() {
                    break outcome;
                }
                assert!(started.elapsed() < Duration::from_secs(5));
                std::thread::sleep(Duration::from_millis(1));
            };
            let estimate = outcome.unwrap().unwrap();
            assert!((estimate.delay_samples - delay as f64).abs() < 0.01);
        }
        assert_eq!(estimator.fft_length(), 1024);
    }
}
