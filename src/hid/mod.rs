pub mod hiddev;
mod hiddev_test;

pub use hiddev::HIDIOCAPPLICATION;
pub use hiddev::HIDIOCGCOLLECTIONINDEX;
pub use hiddev::HIDIOCGCOLLECTIONINFO;
pub use hiddev::HIDIOCGDEVINFO;
pub use hiddev::HIDIOCGFIELDINFO;
pub use hiddev::HIDIOCGFLAG;
pub use hiddev::HIDIOCGNAME;
pub use hiddev::HIDIOCGPHYS;
pub use hiddev::HIDIOCGREPORT;
pub use hiddev::HIDIOCGREPORTINFO;
pub use hiddev::HIDIOCGSTRING;
pub use hiddev::HIDIOCGUCODE;
pub use hiddev::HIDIOCGUSAGE;
pub use hiddev::HIDIOCGUSAGES;
pub use hiddev::HIDIOCGVERSION;
pub use hiddev::HIDIOCINITREPORT;
pub use hiddev::HIDIOCSFLAG;
pub use hiddev::HIDIOCSREPORT;
pub use hiddev::HIDIOCSUSAGE;
pub use hiddev::HIDIOCSUSAGES;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

#[derive(Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum ReportType {
    Input = 1,
    Output,
    Feature,
}
