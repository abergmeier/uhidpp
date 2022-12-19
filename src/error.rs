use std::{error::Error, fmt::Display};

use num_enum::TryFromPrimitive;

pub const HIDPP20_ERROR_FEATURE_INDEX: u8 = 0xff;

pub enum HidError {
    BadReportSize,
}

pub enum HidDeviceError {
    BadDevice,
}

pub enum HidppError {
    BadReportSize,
    ReportIdInvalid,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Hidpp20Error {
    Unknown = 1,
    InvalidArgument,
    OutOfRange,
    HWError,
}

impl Display for Hidpp20Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hid++ 2.0 error: {}", self)
    }
}

impl Error for Hidpp20Error {}
