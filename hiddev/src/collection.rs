use std::fmt::Display;

use hiddev_sys::{HIDIOCAPPLICATION, Devinfo};

#[derive(Debug, PartialEq)]
pub struct ApplicationCollection {
    pub app_index: u32,
    pub usage_code: u16,
}

impl Display for ApplicationCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ app_index: {}, usage_code: {:#x} }}", self.app_index, self.usage_code)   
    }
}

impl ApplicationCollection {
    pub fn find(
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
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::os::unix::prelude::AsRawFd;
    use hiddev_sys::{Devinfo, HIDIOCGDEVINFO};
    use test_log::test;

    use super::ApplicationCollection;

    #[test]
    fn application_collection_display() {
        let app = ApplicationCollection{
            app_index: 13,
            usage_code: 0x0678,
        };
        let s = format!("A {} C", app);
        assert_eq!(s, "A { app_index: 13, usage_code: 0x678 } C");
    }

    #[cfg(device_test = "litra_glow")]
    #[test]
    fn finding_application_collection_works_for_litra_glow() {
        let hid_dev = File::open("/dev/usb/hiddev0").unwrap();
        let fd = hid_dev.as_raw_fd();
        let mut dev_info = Devinfo::default();
        unsafe { HIDIOCGDEVINFO(fd, &mut dev_info) }.unwrap();

        let app_collection = ApplicationCollection::find(fd, &dev_info, 0xFF43).unwrap();
        assert_eq!(app_collection, ApplicationCollection { app_index: 1, usage_code: 0x202 });
    }

}
