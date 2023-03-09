use thiserror::Error;

pub type Result<T> = std::result::Result<T, WaveCollapseError>;

trait IntoWaveCollapseErrorResult<T> {
    fn err_into(self) -> Result<T>;
}

impl<T, E> IntoWaveCollapseErrorResult<T> for std::result::Result<T, E>
where
    E: Into<WaveCollapseError>,
{
    fn err_into(self) -> Result<T> {
        self.map_err(|e| e.into())
    }
}

#[derive(Error, Debug)]
pub enum WaveCollapseError {
    #[error("failed to collapse wave function")]
    InvalidSuperposition,
    #[error("unknown error")]
    Other,
    #[error("not implemented")]
    NotImplemented,
    #[error("input is empty")]
    EmptyInput,
    #[error("iteration failed, this should never happen")]
    IterationError,
}
