#[cfg(test)]
mod test {
    use std::fs::File;
    use std::os::unix::prelude::AsRawFd;

    use crate::{
        find_application_collection, ApplicationCollection, Devinfo, HIDIOCGDEVINFO,
    };

    #[cfg_attr(device_test = "litra_glow", litra_glow)]
    #[test]
    fn find_application_collection_works_for_litra_glow() {
        let hid_dev = File::open("/dev/usb/hiddev0").unwrap();
        let fd = hid_dev.as_raw_fd();
        let mut dev_info = Devinfo::default();
        unsafe { HIDIOCGDEVINFO(fd, &mut dev_info) }.unwrap();

        let app_collection = find_application_collection(fd, &dev_info, 0xFF43).unwrap();
        assert_eq!(app_collection, ApplicationCollection { app_index: 1, usage_code: 0x202 });
    }
}
