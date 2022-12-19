pub mod context;
pub mod device;
pub mod error;
pub mod features;
pub mod hid;
pub mod io;
pub mod report;
pub mod usb;
pub mod wait;
#[macro_use]
extern crate bitflags;

// TODO: Implement a way of preventing cdev from being opened multiple times
