use std::{error::Error, fmt::Display};

use crate::{
    context::Context,
    report::{FapBuilder, HidppReport},
};

use super::FeatureError;

const FEATURE_ID: u8 = 0x0000;

enum FuncIndex {
    GetFeature,
    GetProtocolVersion,
}

pub struct RootFeatureType {
    pub compl_deact: bool,
    pub manuf_deact: bool,
    pub eng: bool,
    pub hidden: bool,
    pub obsl: bool,
}

impl From<u8> for RootFeatureType {
    fn from(b: u8) -> Self {
        Self {
            compl_deact: (b >> 3) & 0b1 == 0b1,
            manuf_deact: (b >> 4) & 0b1 == 0b1,
            eng: (b >> 5) & 0b1 == 0b1,
            hidden: (b >> 6) & 0b1 == 0b1,
            obsl: (b >> 7) & 0b1 == 0b1,
        }
    }
}

pub struct RootFeature {
    pub feature_index: u8,
    pub feature_type: RootFeatureType,
    pub feature_version: u8,
}

impl From<&HidppReport> for RootFeature {
    fn from(report: &HidppReport) -> Self {
        let bs = &report.fap.params;
        Self {
            feature_index: bs[0],
            feature_type: bs[1].into(),
            feature_version: bs[2],
        }
    }
}

pub struct RootProtocolVersion {
    pub protocol_num: u8,
    pub target_sw: u8,
    pub ping_data: u8,
}

impl From<&HidppReport> for RootProtocolVersion {
    fn from(report: &HidppReport) -> Self {
        let bs = &report.fap.params;
        Self {
            protocol_num: bs[0],
            target_sw: bs[1],
            ping_data: bs[2],
        }
    }
}

#[derive(Debug)]
pub enum RootError {
    Internal(Box<dyn Error>),
}

impl Display for RootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal(err) => {
                write!(f, "Internal Root Error: ")?;
                err.fmt(f)
            }
        }
    }
}

impl Error for RootError {}

pub trait Root {
    fn get_feature(
        &mut self,
        context: &mut Context,
        feature_id: u16,
    ) -> Result<RootFeature, FeatureError>;
    fn get_protocol_version(
        &mut self,
        context: &mut Context,
        ping_data: u8,
    ) -> Result<RootProtocolVersion, FeatureError>;
}

pub struct RootImpl {}

fn send_fap_command(context: &mut Context, fap: FapBuilder) -> Result<HidppReport, FeatureError> {
    context
        .sender
        .send(fap)
        .map_err(|err| FeatureError::Internal(err))
}

impl RootImpl {
    fn new_fap_builder(&self, func_index: FuncIndex) -> FapBuilder {
        FapBuilder::new()
            .feature_index(FEATURE_ID)
            .funcindex(func_index as u8)
    }
}

impl Root for RootImpl {
    fn get_feature(
        &mut self,
        context: &mut Context,
        feature_id: u16,
    ) -> Result<RootFeature, FeatureError> {
        let fap = self
            .new_fap_builder(FuncIndex::GetFeature)
            .params(&feature_id.to_be_bytes());
        let report = &send_fap_command(context, fap)?;

        Ok(report.into())
    }
    fn get_protocol_version(
        &mut self,
        context: &mut Context,
        ping_data: u8,
    ) -> Result<RootProtocolVersion, FeatureError> {
        let fap = self
            .new_fap_builder(FuncIndex::GetProtocolVersion)
            .params(&[0, 0, ping_data]);
        let report = &send_fap_command(context, fap)?;

        Ok(report.into())
    }
}

pub fn init<'a>() -> Result<Option<Box<dyn Root + 'a>>, Box<dyn Error>> {
    Ok(Some(Box::new(RootImpl {})))
}
