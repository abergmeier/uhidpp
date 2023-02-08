mod lib_test;

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
            "{{ num_values: {}, uref: {}, values: {:X?} }}",
            self.num_values, self.uref, self.values
        )
    }
}

#[derive(Default)]
#[repr(C)]
pub struct FieldInfo {
    pub report_type: u32,
    pub report_id: u32,
    pub field_index: u32,
    pub maxusage: u32,
    pub flags: u32,
    pub physical: u32,
    /// physical usage for this field
    pub logical: u32,
    /// logical usage for this field
    pub application: u32,
    /// application usage for this field
    pub logical_minimum: i32,
    pub logical_maximum: i32,
    pub physical_minimum: i32,
    pub physical_maximum: i32,
    pub unit_exponent: u32,
    pub unit: u32,
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

#[derive(Debug, PartialEq)]
pub struct ApplicationCollection {
    pub app_index: u32,
    pub usage_code: u16,
}

pub fn find_application_collection(
    fd: i32,
    dev_info: &Devinfo,
    application_collection_usage_page: u16,
) -> nix::Result<ApplicationCollection> {
    log::debug!(
        "Found {} application collections",
        dev_info.num_applications
    );

    for app_index in 0..dev_info.num_applications {
        let usage = unsafe { HIDIOCAPPLICATION(fd, app_index) }?;

        let usage_page = (usage as u32 >> 16) as u16;
        let usage_code = ((usage as u32) & 0xFFFF) as u16;
        if usage_page == application_collection_usage_page {
            return Ok(ApplicationCollection {
                app_index,
                usage_code,
            });
        }
    }

    Err(nix::Error::ENOENT)
}


#[derive(Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum ReportType {
    Input = 1,
    Output,
    Feature,
}
