use nix::ioctl_read;
use nix::ioctl_read_buf;
use nix::ioctl_write_ptr;
use nix::ioctl_readwrite;
use nix::ioctl_none;

const HID_STRING_SIZE: usize = 256;
const HID_MAX_MULTI_USAGES: usize = 1024;


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

#[repr(C)]
pub struct UsageRef {
	pub report_type: u32,
    pub report_id: u32,
	pub field_index: u32,
	pub usage_index: u32,
	pub usage_code: u32,
	pub value: i32,
}

/// hiddev_usage_ref_multi is used for sending multiple bytes to a control.
/// It really manifests itself as setting the value of consecutive usages
#[repr(C)]
pub struct UsageRefMulti {
	pub uref: UsageRef,
	pub num_values: u32,
	pub values: [i32; HID_MAX_MULTI_USAGES]
}

#[repr(C)]
pub struct FieldInfo {
	pub report_type: u32,
	pub report_id: u32,
	pub field_index: u32,
	pub maxusage: u32,
    pub flags: u32,
	pub physical: u32, /// physical usage for this field
	pub logical: u32, /// logical usage for this field
	pub application: u32, /// application usage for this field
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
pub unsafe fn HIDIOCAPPLICATION(fd: nix::libc::c_int,
                    index: u32)
                    -> nix::Result<nix::libc::c_int> {
    nix::convert_ioctl_res!(nix::libc::ioctl(fd, nix::request_code_none!('H', 0x02) as nix::sys::ioctl::ioctl_num_type, index))
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
ioctl_write_ptr!(HIDIOCSFLAG, 'H', 0x0F, i32);
ioctl_write_ptr!(HIDIOCGCOLLECTIONINDEX, 'H', 0x10, UsageRef);
ioctl_readwrite!(HIDIOCGCOLLECTIONINFO, 'H', 0x11, CollectionInfo);
ioctl_read_buf!(HIDIOCGPHYS, 'H', 0x12, u8);

// For writing/reading to multiple/consecutive usages
ioctl_readwrite!(HIDIOCGUSAGES, 'H', 0x13, UsageRefMulti);
ioctl_write_ptr!(HIDIOCSUSAGES, 'H', 0x14, UsageRefMulti);

/* 
 * Flags to be used in HIDIOCSFLAG
 */
const FLAG_UREF: u8 = 0x1;
const FLAG_REPORT: u8 = 0x2;
const FLAGS: u8 = 0x3;

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

pub fn find_application_collection(fd: i32, dev_info: &Devinfo, application_collection_usage_page: u16) -> nix::Result<u32> {

	log::debug!("Found {} application collections", dev_info.num_applications);

    for app_index in 0..dev_info.num_applications {
		let usage = unsafe { HIDIOCAPPLICATION(fd, app_index) }?;

		let usage_page = (usage as u32 >> 16) as u16;
		let _usage_code = ((usage as u32) & 0xFFFF) as u16;
        if usage_page == application_collection_usage_page {
            return Ok(app_index);
        }
    }

    Err(nix::Error::ENOENT)
}
