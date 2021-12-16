use fvm_shared::{actor_error, encoding, error::ActorError, error::ExitCode};
use wasmtime::Trap;

use crate::kernel::blocks;

/// Execution result.
pub type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(thiserror::Error, Debug)]
pub enum ExecutionError {
    #[error("{0:?}")]
    Actor(#[from] ActorError),
    #[error("{0:?}")]
    SystemError(#[from] anyhow::Error),
}

impl ExecutionError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            ExecutionError::Actor(e) => e.exit_code(),
            ExecutionError::SystemError(_) => ExitCode::ErrPlaceholder, // same as fatal before
        }
    }
}

impl From<encoding::Error> for ExecutionError {
    fn from(e: encoding::Error) -> Self {
        ExecutionError::SystemError(e.into())
    }
}

impl From<blocks::BlockError> for ExecutionError {
    fn from(e: blocks::BlockError) -> Self {
        use blocks::BlockError::*;
        match e {
            Unreachable(..)
            | InvalidHandle(..)
            | InvalidMultihashSpec { .. }
            | InvalidCodec(..) => {
                ExecutionError::Actor(actor_error!(SysErrIllegalArgument; e.to_string()))
            }
            // TODO: Not quite the correct error but we don't have a better oen for now.
            TooManyBlocks => ExecutionError::Actor(actor_error!(SysErrIllegalActor; e.to_string())),
            MissingState(k) => ExecutionError::SystemError(anyhow::anyhow!("missing block: {}", k)),
        }
    }
}

impl From<ipld_hamt::Error> for ExecutionError {
    fn from(e: ipld_hamt::Error) -> Self {
        // TODO: box dyn error is pervasive..
        ExecutionError::SystemError(anyhow::anyhow!("{:?}", e))
    }
}

impl From<Box<dyn std::error::Error>> for ExecutionError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        // TODO: make better
        ExecutionError::SystemError(anyhow::anyhow!(e.to_string()))
    }
}

impl From<ExecutionError> for Trap {
    fn from(e: ExecutionError) -> Self {
        Trap::from(Box::from(e))
    }
}
