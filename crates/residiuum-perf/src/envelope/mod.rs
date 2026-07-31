//! PQH-4: L0 calibration and L1 filesystem/device envelope.
//!
//! L0 measures the observer floor (generator/copy/clock/probes) without
//! storage. L1 measures disposable ordinary-file I/O on a dedicated work
//! root — never raw block devices. Tests use a fake I/O adapter.

mod io;
mod l0;
mod l1;
mod report;
mod window;

pub use io::{
    FakeIoAdapter, FakeIoConfig, FileIoAdapter, IoAdapter, IoError, IoMode, IoOp, IoResult,
    SyncMode,
};
pub use l0::{run_l0_calibration, L0Config, L0Report};
pub use l1::{run_l1_envelope, ColdState, L1Config, L1Point};
pub use report::L1Report;
pub use window::{WindowClass, WindowDetector, WindowSample};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvelopeError {
    #[error("envelope: {0}")]
    Msg(String),
    #[error("io: {0}")]
    Io(#[from] IoError),
    #[error("runner: {0}")]
    Runner(#[from] crate::runner::RunnerError),
}
