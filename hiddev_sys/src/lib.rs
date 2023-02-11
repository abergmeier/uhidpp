mod field;

pub use field::FieldInfo;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

use std::fmt::Display;

use nix::ioctl_none;
use nix::ioctl_read;
use nix::ioctl_read_buf;
use nix::ioctl_readwrite;
use nix::ioctl_write_ptr;

const HID_STRING_SIZE: usize = 256;
pub const HID_MAX_MULTI_USAGES: usize = 1024;

#[repr(C)]
pub struct Event {
    hid: u32,
    value: i32,
}

#[derive(Default)]
#[repr(C)]
pub struct Devinfo {
    pub bustype: u32,
    pub busnum: u32,
    pub devnum: u32,
    pub ifnum: u32,
    pub vendor: i16,
    pub product: i16,
    pub version: i16,
    pub num_applications: u32,
}

impl Display for Devinfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ busnum: {:#X}, bustype: {:#X}, devnum: {:#X}, ifnum: {:#X}, num_applications: {:#X}, product: {:#X}, vendor: {:#X}, version: {:#X} }}", self.busnum, self.bustype, self.devnum, self.ifnum, self.num_applications, self.product, self.vendor, self.version)
    }
}

#[repr(C)]
pub struct StringDescriptor {
    pub index: i32,
    pub value: [char; HID_STRING_SIZE],
}

#[derive(Default)]
#[repr(C)]
pub struct ReportInfo {
    pub report_type: u32,
    pub report_id: u32,
    pub num_fields: u32,
}

pub enum ReportIds {
    Short = 0x10,
    Long = 0x11,
    VeryLong = 0x12,
}

impl Display for ReportInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ num_fields: {}, report_id: {:#X}, report_type: {:#X} }}",
            self.num_fields, self.report_id, self.report_type
        )
    }
}

#[derive(Default)]
#[repr(C)]
pub struct UsageRef {
    pub report_type: u32,
    pub report_id: u32,
    pub field_index: u32,
    pub usage_index: u32,
    pub usage_code: u32,
    pub value: i32,
}

impl Display for UsageRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ field_index: {:#X}, report_id: {:#X}, report_type: {:#X}, usage_code: {:#X}, usage_index: {:#X}, value: {:#X} }}",
		self.field_index, self.report_id, self.report_type, self.usage_code, self.usage_index, self.value)
    }
}

/// hiddev_usage_ref_multi is used for sending multiple bytes to a control.
/// It really manifests itself as setting the value of consecutive usages
#[repr(C)]
pub struct UsageRefMulti {
    pub uref: UsageRef,
    pub num_values: u32,
    pub values: [i32; HID_MAX_MULTI_USAGES],
}

impl Default for UsageRefMulti {
    fn default() -> Self {
        Self {
            uref: UsageRef::default(),
            num_values: 0,
            values: [0; HID_MAX_MULTI_USAGES],
        }
    }
}

impl Display for UsageRefMulti {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ num_values: {}, uref: {}, values:",
            self.num_values, self.uref,
        )?;
        let values = &self.values;
        // TODO: This seems inefficient!
        let last_index = values.iter().enumerate().rev().fold(None, |acc, (i, v)| {
            if acc.is_some() {
                return acc;
            }
            if *v == 0x0 {
                None
            } else {
                Some(i)
            }
        });
        last_index.map_or(Ok(()), |index| {
            // TODO: This seems inefficient!
            let s = values[0..=index]
                .as_ref()
                .iter()
                .map(|v| format!("{:#0x}", *v))
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, " {}", s)
        })?;
        f.write_str(" }")
    }
}



#[repr(C)]
pub struct CollectionInfo {
    pub index: u32,
    pub _type: u32,
    pub usage: u32,
    pub level: u32,
}

ioctl_read!(HIDIOCGVERSION, 'H', 0x01, i32);
pub unsafe fn HIDIOCAPPLICATION(fd: nix::libc::c_int, index: u32) -> nix::Result<nix::libc::c_int> {
    nix::convert_ioctl_res!(nix::libc::ioctl(
        fd,
        nix::request_code_none!('H', 0x02) as nix::sys::ioctl::ioctl_num_type,
        index
    ))
}
ioctl_read!(HIDIOCGDEVINFO, 'H', 0x03, Devinfo);
ioctl_read!(HIDIOCGSTRING, 'H', 0x04, StringDescriptor);
ioctl_none!(HIDIOCINITREPORT, 'H', 0x05);
ioctl_read_buf!(HIDIOCGNAME, 'H', 0x06, u8);
ioctl_write_ptr!(HIDIOCGREPORT, 'H', 0x07, ReportInfo);
ioctl_write_ptr!(HIDIOCSREPORT, 'H', 0x08, ReportInfo);
ioctl_readwrite!(HIDIOCGREPORTINFO, 'H', 0x09, ReportInfo);
ioctl_readwrite!(HIDIOCGFIELDINFO, 'H', 0x0A, FieldInfo);
ioctl_readwrite!(HIDIOCGUSAGE, 'H', 0x0B, UsageRef);
ioctl_write_ptr!(HIDIOCSUSAGE, 'H', 0x0C, UsageRef);
ioctl_readwrite!(HIDIOCGUCODE, 'H', 0x0D, UsageRef);

ioctl_read!(HIDIOCGFLAG, 'H', 0x0E, i32);
ioctl_write_ptr!(HIDIOCSFLAG, 'H', 0x0F, HIDIOCSFLAGFlag);
ioctl_write_ptr!(HIDIOCGCOLLECTIONINDEX, 'H', 0x10, UsageRef);
ioctl_readwrite!(HIDIOCGCOLLECTIONINFO, 'H', 0x11, CollectionInfo);
ioctl_read_buf!(HIDIOCGPHYS, 'H', 0x12, u8);

// For writing/reading to multiple/consecutive usages
ioctl_readwrite!(HIDIOCGUSAGES, 'H', 0x13, UsageRefMulti);
ioctl_write_ptr!(HIDIOCSUSAGES, 'H', 0x14, UsageRefMulti);

#[derive(IntoPrimitive)]
#[repr(u8)]
/// Flag to be used in HIDIOCSFLAG
pub enum HIDIOCSFLAGFlag {
    UREF = 0x1,
    REPORT = 0x2,
    ALL = 0x3,
}

/* To traverse the input report descriptor info for a HID device, perform the
 * following:
 *
 * rinfo.report_type = HID_REPORT_TYPE_INPUT;
 * rinfo.report_id = HID_REPORT_ID_FIRST;
 * ret = ioctl(fd, HIDIOCGREPORTINFO, &rinfo);
 *
 * while (ret >= 0) {
 * 	for (i = 0; i < rinfo.num_fields; i++) {
 * 		finfo.report_type = rinfo.report_type;
 * 		finfo.report_id = rinfo.report_id;
 * 		finfo.field_index = i;
 * 		ioctl(fd, HIDIOCGFIELDINFO, &finfo);
 * 		for (j = 0; j < finfo.maxusage; j++) {
 * 			uref.report_type = rinfo.report_type;
 * 			uref.report_id = rinfo.report_id;
 * 			uref.field_index = i;
 * 			uref.usage_index = j;
 * 			ioctl(fd, HIDIOCGUCODE, &uref);
 * 			ioctl(fd, HIDIOCGUSAGE, &uref);
 * 		}
 * 	}
 * 	rinfo.report_id |= HID_REPORT_ID_NEXT;
 * 	ret = ioctl(fd, HIDIOCGREPORTINFO, &rinfo);
 * }
 */

