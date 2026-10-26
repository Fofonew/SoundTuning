use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    Dsp(#[from] dsp_core::DspError),
    #[error("could not start engine thread: {0}")]
    Thread(#[source] std::io::Error),
}
