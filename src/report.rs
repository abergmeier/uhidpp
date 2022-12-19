pub const SHORT_LENGTH: u8 = 8;
pub const LONG_LENGTH: u8 = 20;
pub const VERY_LONG_MAX_LENGTH: u8 = 64;

pub const REPORT_ID_HIDPP_SHORT: u8 = 0x10;
pub const REPORT_ID_HIDPP_LONG: u8 = 0x11;
pub const REPORT_ID_HIDPP_VERY_LONG: u8 = 0x12;

bitflags! {
    pub struct SupportedReportLengths: u8 {
        const SHORT = 0x01;
        const LONG = 0x02;
        const VERY_LONG = 0x04;
    }
}

/// Deliberately do not use the software ID of the kernel here
const SW_ID: u8 = 0x02;

pub struct FapBuilder {
    built: Fap,
    copied_len: usize,
}

impl FapBuilder {
    pub fn new() -> Self {
        Self {
            built: Fap::default(),
            copied_len: 0,
        }
    }

    pub fn copied_params_len(&self) -> usize {
        self.copied_len
    }

    pub fn feature_index(mut self, value: u8) -> Self {
        self.built.feature_index = value;
        self
    }

    pub fn funcindex(mut self, value: u8) -> Self {
        self.built.funcindex_clientid = value | SW_ID;
        self
    }

    pub fn params(mut self, value: &[u8]) -> Self {
        self.copied_len = value.len();
        self.built.params.clone_from_slice(value);
        self
    }

    pub fn build(self) -> Fap {
        self.built
    }
}

/// Feature access protocol - as specified for
/// Hid++ 2.0 and up
#[derive(Clone, Debug, PartialEq)]
pub struct Fap {
    pub feature_index: u8,
    pub funcindex_clientid: u8,
    pub params: [u8; VERY_LONG_MAX_LENGTH as usize - 4],
}

impl Fap {
    pub fn copy_into(&self, buf: &mut [u8]) {
        buf[0] = self.feature_index;
        buf[1] = self.funcindex_clientid;
        let buf_len = buf.len();
        buf[2..].copy_from_slice(&self.params[0..(buf_len - 2)]);
    }

    pub fn has_sw_id(&self) -> bool {
        (self.funcindex_clientid & 0x0F) != 0x00
    }
}

impl Default for Fap {
    fn default() -> Self {
        Self {
            feature_index: 0,
            funcindex_clientid: 0,
            params: [0; VERY_LONG_MAX_LENGTH as usize - 4],
        }
    }
}

impl From<&[u8]> for Fap {
    fn from(bs: &[u8]) -> Self {
        let feature_index = bs[0];
        let funcindex_clientid = bs[1];
        let mut params = [0; VERY_LONG_MAX_LENGTH as usize - 4];
        params.copy_from_slice(&bs[2..]);
        Self {
            feature_index,
            funcindex_clientid,
            params,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HidppReport {
    pub report_id: u8,
    pub device_index: u8,
    pub fap: Fap,
}

impl HidppReport {
    pub fn new(fap: FapBuilder) -> Self {
        let report_id = if fap.copied_params_len() > (LONG_LENGTH as usize - 4) {
            REPORT_ID_HIDPP_VERY_LONG
        } else {
            REPORT_ID_HIDPP_LONG
        };

        Self {
            report_id,
            device_index: 0,
            fap: fap.build(),
        }
    }

    pub fn has_sw_id(&self) -> bool {
        self.fap.has_sw_id()
    }

    pub fn copy_into(&self, buf: &mut [u8]) {
        buf[0] = self.report_id;
        buf[1] = self.device_index;
        self.fap.copy_into(&mut buf[2..]);
    }
}

impl From<&[u8]> for HidppReport {
    fn from(bs: &[u8]) -> Self {
        let report_id = bs[0];
        let device_index = bs[1];
        Self {
            report_id,
            device_index,
            fap: bs[2..].into(),
        }
    }
}

impl From<&[u8; VERY_LONG_MAX_LENGTH as usize]> for HidppReport {
    fn from(bs: &[u8; VERY_LONG_MAX_LENGTH as usize]) -> Self {
        let report_id = bs[0];
        let device_index = bs[1];
        Self {
            report_id,
            device_index,
            fap: bs[2..].into(),
        }
    }
}
