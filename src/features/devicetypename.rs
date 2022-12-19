use num_enum::TryFromPrimitive;
use std::error::Error;

use crate::{
    context::{Context, WithContext},
    report::{FapBuilder, HidppReport},
};

use super::{root::Root, CommonFeatureImpl, FeatureAccessor, FeatureError};

const FEATURE_ID: u16 = 0x0005;

pub struct DeviceNameCount {
    pub device_name_count: u8,
}

impl From<&HidppReport> for DeviceNameCount {
    fn from(report: &HidppReport) -> Self {
        let b = report.fap.params[0];
        Self {
            device_name_count: b,
        }
    }
}

pub struct DeviceName {
    pub char_index: u8,
}

impl From<&HidppReport> for DeviceName {
    fn from(report: &HidppReport) -> Self {
        let b = report.fap.params[0];
        Self { char_index: b }
    }
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum DeviceTypes {
    Keyboard,
    RemoteControl,
    Numpad,
    Mouse,
    Trackpad,
    Trackball,
    Presenter,
    Receiver,
    Headset,
    Webcam,
    SteeringWheel,
    Joystick,
    Gamepad,
    Dock,
    Speaker,
    Microphone,
    IlluminationLight,
    ProgrammableController,
    CarSimPedals,
    Adapter,
}

pub struct DeviceType {
    pub device_type: Option<DeviceTypes>,
}

impl From<&HidppReport> for DeviceType {
    fn from(report: &HidppReport) -> Self {
        let b = report.fap.params[0];
        let res = b.try_into();
        Self {
            device_type: res.map_or(None, |v| Some(v)),
        }
    }
}

enum FuncIndex {
    GetDeviceNameCount,
    GetDeviceName,
    GetDeviceType,
}

pub trait DeviceTypeName {
    fn get_device_name_count(
        &mut self,
        context: &mut Context,
    ) -> Result<DeviceNameCount, FeatureError>;
    fn get_device_name(
        &mut self,
        context: &mut Context,
        char_index: u8,
    ) -> Result<DeviceName, FeatureError>;
    fn get_device_type(&mut self, context: &mut Context) -> Result<DeviceType, FeatureError>;
}

pub struct DeviceTypeNameImpl {
    common_impl: CommonFeatureImpl,
}

impl DeviceTypeNameImpl {
    fn new_fap_builder(&self, func_index: FuncIndex) -> FapBuilder {
        FapBuilder::new()
            .feature_index(self.common_impl.feature_index)
            .funcindex(func_index as u8)
    }
}

impl DeviceTypeName for DeviceTypeNameImpl {
    fn get_device_name(
        &mut self,
        context: &mut Context,
        char_index: u8,
    ) -> Result<DeviceName, FeatureError> {
        let fap = self
            .new_fap_builder(FuncIndex::GetDeviceName)
            .params(&[char_index]);
        let report = &self.common_impl.send_fap_command(context, fap)?;

        Ok(report.into())
    }
    fn get_device_name_count(
        &mut self,
        context: &mut Context,
    ) -> Result<DeviceNameCount, FeatureError> {
        let fap = self.new_fap_builder(FuncIndex::GetDeviceNameCount);
        let report = &self.common_impl.send_fap_command(context, fap)?;

        Ok(report.into())
    }
    fn get_device_type(&mut self, context: &mut Context) -> Result<DeviceType, FeatureError> {
        let fap = self.new_fap_builder(FuncIndex::GetDeviceType);
        let report = &self.common_impl.send_fap_command(context, fap)?;

        Ok(report.into())
    }
}

pub fn init<'a>(
    context: &mut Context,
    root: &mut dyn Root,
) -> Result<Option<Box<dyn DeviceTypeName + 'a>>, Box<dyn Error>> {
    let accessor = FeatureAccessor::new(root, FEATURE_ID)
        .with_context(context)
        .access_common_impl()?;
    Ok(accessor.map_or(None, |common_impl| {
        Some(Box::new(DeviceTypeNameImpl { common_impl }))
    }))
}
