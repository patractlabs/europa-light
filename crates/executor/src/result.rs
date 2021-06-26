//! Ceres executor result
use crate::trap::Trap;
use ceres_std::{fmt, format, String, Vec};

#[repr(i32)]
#[derive(Debug, PartialEq, Eq)]
pub enum ReturnCode {
    /// API call successful.
    Success = 0,
    /// The called function trapped and has its state changes reverted.
    /// In this case no output buffer is returned.
    CalleeTrapped = 1,
    /// The called function ran to completion but decided to revert its state.
    /// An output buffer is returned when one was supplied.
    CalleeReverted = 2,
    /// The passed key does not exist in storage.
    KeyNotFound = 3,
    /// Transfer failed because it would have brought the sender's total balance below the
    /// subsistence threshold.
    BelowSubsistenceThreshold = 4,
    /// Transfer failed for other reasons. Most probably reserved or locked balance of the
    /// sender prevents the transfer.
    TransferFailed = 5,
    /// The newly created contract is below the subsistence threshold after executing
    /// its constructor.
    NewContractNotFunded = 6,
    /// No code could be found at the supplied code hash.
    CodeNotFound = 7,
    /// The contract that was called is either no contract at all (a plain account)
    /// or is a tombstone.
    NotCallable = 8,
    /// The call to `seal_debug_message` had no effect because debug message
    /// recording was disabled.
    LoggingDisabled = 9,
    /// Unexpected return code
    UnExpectedReturnCode = 10,
}

impl From<i32> for ReturnCode {
    fn from(n: i32) -> ReturnCode {
        match n {
            0 => ReturnCode::Success,
            1 => ReturnCode::CalleeTrapped,
            2 => ReturnCode::CalleeReverted,
            3 => ReturnCode::KeyNotFound,
            4 => ReturnCode::BelowSubsistenceThreshold,
            5 => ReturnCode::TransferFailed,
            6 => ReturnCode::NewContractNotFunded,
            7 => ReturnCode::CodeNotFound,
            8 => ReturnCode::NotCallable,
            9 => ReturnCode::LoggingDisabled,
            _ => ReturnCode::UnExpectedReturnCode,
        }
    }
}

/// Ceres executor errors
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InitMemoryFailed,
    /// Memory outof bounds
    OutOfBounds,
    InitModuleFailed,
    ExecuteFailed(ReturnCode),
    UnkownError,
    Trap(Trap),
    GetFunctionNameFailed,
    CreateWasmtimeConfigFailed,
    GetExternalFailed(String),
    DecodeRuntimeValueFailed,
    OutputBufferTooSmall,
    WrongArugmentLength,
    SetStorageFailed,
    ReturnData {
        flags: u32,
        data: Vec<u8>,
    },
    /// Topics
    TooManyTopics,
    DuplicateTopics,
    TopicValueTooLarge,
    /// Gas
    OutOfGas,
    /// Custom Error
    Custom(&'static str),
    /// Downcast anyhow error failed
    AnyHow,
    /// Unexpected return value
    UnExpectedReturnValue,
    ParseWasmModuleFailed,
    ExecutorNotInited,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::result::Result<(), fmt::Error> {
        f.write_str(&format!("{:?}", &self))?;
        Ok(())
    }
}

impl From<anyhow::Error> for Error {
    fn from(_: anyhow::Error) -> Error {
        Error::AnyHow
    }
}

/// Ceres executor result
pub type Result<T> = core::result::Result<T, Error>;
