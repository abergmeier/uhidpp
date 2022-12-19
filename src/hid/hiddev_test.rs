#[cfg(test)]
mod test {
    use std::fs::File;
    use std::os::unix::prelude::AsRawFd;

    use crate::hid::hiddev::{find_application_collection, Devinfo, HIDIOCGDEVINFO};

    #[test]
    fn find_application_collection_works() {
        let hid_dev = File::open("/dev/usb/hiddev0").unwrap();
        let fd = hid_dev.as_raw_fd();
        let mut dev_info = Devinfo::default();
        unsafe { HIDIOCGDEVINFO(fd, &mut dev_info) }.unwrap();

        let app_index = find_application_collection(fd, &dev_info, 0xFF43).unwrap();
        assert_eq!(app_index, 1);
    }
}
