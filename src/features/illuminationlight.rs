use std::error::Error;

use crate::{
    context::{Context, WithContext},
    report::{FapBuilder, HidppReport},
};

use super::{root::Root, CommonFeatureImpl, FeatureAccessor, FeatureError};

const FEATURE_ID: u16 = 0x1990;

enum FuncIndex {
    GetIllumination,
    SetIllumination,
    GetBrightnessInfo,
    GetBrightness,
    SetBrightness,
    GetBrightnessLevels,
    SetBrightnessLevels,
    GetColorTemperatureInfo,
    GetColorTemperature,
    SetColorTemperature,
    GetColorTemperatureLevels,
    SetColorTemperatureLevels,
}

pub struct IlluminationState {
    pub state: u8,
}

impl From<&HidppReport> for IlluminationState {
    fn from(report: &HidppReport) -> Self {
        Self {
            state: report.fap.params[0] & 0b1,
        }
    }
}

impl Into<[u8; 1]> for &IlluminationState {
    fn into(self) -> [u8; 1] {
        [self.state & 0b1]
    }
}

pub struct ControlInfoCapabilities {
    pub has_non_linear_levels: bool,
    pub has_linear_levels: bool,
    pub has_events: bool,
}

impl From<u8> for ControlInfoCapabilities {
    fn from(b: u8) -> Self {
        Self {
            has_events: (b & 0b001) == 0b001,
            has_linear_levels: (b & 0b010) == 0b010,
            has_non_linear_levels: (b & 0b100) == 0b100,
        }
    }
}

pub struct ControlInfo {
    pub capabilities: ControlInfoCapabilities,
    pub min: u16,
    pub max: u16,
    pub res: u16,
    pub max_levels: u8,
}

impl From<&HidppReport> for ControlInfo {
    fn from(report: &HidppReport) -> Self {
        Self {
            capabilities: report.fap.params[0].into(),
            min: u16::from_be_bytes(report.fap.params[1..2].try_into().unwrap()),
            max: u16::from_be_bytes(report.fap.params[3..4].try_into().unwrap()),
            res: u16::from_be_bytes(report.fap.params[5..6].try_into().unwrap()),
            max_levels: report.fap.params[7] & 0x0F,
        }
    }
}

pub struct ControlLevelsFlags {
    pub valid_count: u8,
    pub linear: bool,
}

impl AsRef<ControlLevelsFlags> for ControlLevelsFlags {
    fn as_ref(&self) -> &ControlLevelsFlags {
        &self
    }
}

impl From<u8> for ControlLevelsFlags {
    fn from(b: u8) -> Self {
        Self {
            linear: (b & 0b1) == 0b1,
            valid_count: (b & 0xE0) >> 5,
        }
    }
}

impl Into<u8> for &ControlLevelsFlags {
    fn into(self) -> u8 {
        (self.valid_count & 0xE0) << 5 | (self.linear as u8) << 0
    }
}

pub struct ControlTargetLevels {
    pub start_index: u8,
    pub level_count: u8,
}

impl AsRef<ControlTargetLevels> for ControlTargetLevels {
    fn as_ref(&self) -> &ControlTargetLevels {
        &self
    }
}

impl From<u8> for ControlTargetLevels {
    fn from(b: u8) -> Self {
        Self {
            start_index: (b & 0xF0) >> 4,
            level_count: (b & 0x0F) >> 0,
        }
    }
}

impl Into<u8> for &ControlTargetLevels {
    fn into(self) -> u8 {
        (self.start_index & 0x00FF) << 4 | (self.level_count & 0x00FF) << 0
    }
}

pub struct ControlLevelsLinear {
    pub flags: ControlLevelsFlags,
    pub target_levels: ControlTargetLevels,
    pub level_min_value: u16,
    pub level_max_value: u16,
    pub level_step_value: u16,
}

impl From<&[u8; 16]> for ControlLevelsLinear {
    fn from(bs: &[u8; 16]) -> Self {
        Self {
            flags: bs[0].into(),
            target_levels: bs[1].into(),
            level_min_value: u16::from_be_bytes(bs[2..3].try_into().unwrap()),
            level_max_value: u16::from_be_bytes(bs[4..5].try_into().unwrap()),
            level_step_value: u16::from_be_bytes(bs[6..7].try_into().unwrap()),
        }
    }
}

impl Into<[u8; 16]> for &ControlLevelsLinear {
    fn into(self) -> [u8; 16] {
        let mut res: [u8; 16] = [0; 16];
        res[0] = self.flags.as_ref().into();
        res[1] = self.target_levels.as_ref().into();
        res[2..3].clone_from_slice(&u16::to_be_bytes(self.level_min_value));
        res[4..5].clone_from_slice(&u16::to_be_bytes(self.level_max_value));
        res[6..7].clone_from_slice(&u16::to_be_bytes(self.level_step_value));
        res
    }
}

pub struct ControlLevelsNonLinear {
    pub flags: ControlLevelsFlags,
    pub target_levels: ControlTargetLevels,
    pub level_0_value: u16,
    pub level_1_value: u16,
    pub level_2_value: u16,
    pub level_3_value: u16,
    pub level_4_value: u16,
    pub level_5_value: u16,
    pub level_6_value: u16,
}

impl From<&[u8; 16]> for ControlLevelsNonLinear {
    fn from(bs: &[u8; 16]) -> Self {
        Self {
            flags: bs[0].into(),
            target_levels: bs[1].into(),
            level_0_value: u16::from_be_bytes(bs[2..3].try_into().unwrap()),
            level_1_value: u16::from_be_bytes(bs[4..5].try_into().unwrap()),
            level_2_value: u16::from_be_bytes(bs[6..7].try_into().unwrap()),
            level_3_value: u16::from_be_bytes(bs[8..9].try_into().unwrap()),
            level_4_value: u16::from_be_bytes(bs[10..11].try_into().unwrap()),
            level_5_value: u16::from_be_bytes(bs[12..13].try_into().unwrap()),
            level_6_value: u16::from_be_bytes(bs[14..15].try_into().unwrap()),
        }
    }
}

impl Into<[u8; 16]> for &ControlLevelsNonLinear {
    fn into(self) -> [u8; 16] {
        let mut res: [u8; 16] = [0; 16];
        res[0] = self.flags.as_ref().into();
        res[1] = self.target_levels.as_ref().into();
        res[2..3].clone_from_slice(&u16::to_be_bytes(self.level_0_value));
        res[4..5].clone_from_slice(&u16::to_be_bytes(self.level_1_value));
        res[6..7].clone_from_slice(&u16::to_be_bytes(self.level_2_value));
        res[8..9].clone_from_slice(&u16::to_be_bytes(self.level_3_value));
        res[10..11].clone_from_slice(&u16::to_be_bytes(self.level_4_value));
        res[12..13].clone_from_slice(&u16::to_be_bytes(self.level_5_value));
        res[14..15].clone_from_slice(&u16::to_be_bytes(self.level_6_value));
        res
    }
}

pub enum ControlLevels {
    Linear(ControlLevelsLinear),
    NonLinear(ControlLevelsNonLinear),
}

impl From<&HidppReport> for ControlLevels {
    fn from(report: &HidppReport) -> Self {
        let flags: ControlLevelsFlags = report.fap.params[0].into();
        let bs: &[u8; 16] = report.fap.params[0..15].try_into().unwrap();
        if flags.linear {
            let mode: ControlLevelsLinear = bs.into();
            return ControlLevels::Linear(mode);
        } else {
            let mode: ControlLevelsNonLinear = bs.into();
            return ControlLevels::NonLinear(mode);
        }
    }
}

impl Into<[u8; 16]> for &ControlLevels {
    fn into(self) -> [u8; 16] {
        match self {
            ControlLevels::Linear(x) => x.into(),
            ControlLevels::NonLinear(x) => x.into(),
        }
    }
}

pub struct LevelsRequest {
    start_index: u8,
}

impl Into<[u8; 1]> for &LevelsRequest {
    fn into(self) -> [u8; 1] {
        [self.start_index << 4]
    }
}

pub struct ControlValue {
    value: u16,
}

impl From<&HidppReport> for ControlValue {
    fn from(report: &HidppReport) -> Self {
        let value = u16::from_be_bytes(report.fap.params[0..1].try_into().unwrap());
        Self { value }
    }
}

impl Into<[u8; 2]> for &ControlValue {
    fn into(self) -> [u8; 2] {
        self.value.to_be_bytes()
    }
}

/// This feature controls various aspects of illumination light devices such as studio lights or video
/// streaming lights.
pub trait IlluminationLight {
    /// Retrieves the current illumination state.
    fn get_illumination(
        &mut self,
        context: &mut Context,
    ) -> Result<IlluminationState, FeatureError>;
    /// Turns the illumination on or off.
    fn set_illumination(
        &mut self,
        context: &mut Context,
        state: &IlluminationState,
    ) -> Result<(), FeatureError>;

    /// Returns information about the device’s brightness capabilities.
    fn get_brightness_info(&mut self, context: &mut Context) -> Result<ControlInfo, FeatureError>;
    /// Returns the current hardware brightness value.
    fn get_brightness(&mut self, context: &mut Context) -> Result<ControlValue, FeatureError>;
    /// Sets the hardware brightness value.
    fn set_brightness(
        &mut self,
        context: &mut Context,
        value: &ControlValue,
    ) -> Result<(), FeatureError>;

    /// Returns the device’s current brightness level configuration.
    fn get_brightness_levels(
        &mut self,
        context: &mut Context,
        start_index: &LevelsRequest,
    ) -> Result<ControlLevels, FeatureError>;
    /// Sets the device’s brightness level configuration.
    fn set_brightness_levels(
        &mut self,
        context: &mut Context,
        value: &ControlLevels,
    ) -> Result<(), FeatureError>;

    /// Returns information about the device’s color temperature capabilities.
    fn get_color_temperature_info(
        &mut self,
        context: &mut Context,
    ) -> Result<ControlInfo, FeatureError>;
    /// Returns the current hardware color temperature value.
    fn get_color_temperature(
        &mut self,
        context: &mut Context,
    ) -> Result<ControlValue, FeatureError>;
    /// Sets the hardware color temperature value.
    fn set_color_temperature(
        &mut self,
        context: &mut Context,
        value: &ControlValue,
    ) -> Result<(), FeatureError>;
    /// Returns the device’s current color temperature level configuration.
    fn get_color_temperature_levels(
        &mut self,
        context: &mut Context,
        req: &LevelsRequest,
    ) -> Result<ControlLevels, FeatureError>;
    /// Sets the device’s color temperature level configuration.
    fn set_color_temperature_levels(
        &mut self,
        context: &mut Context,
        value: &ControlLevels,
    ) -> Result<(), FeatureError>;
}

struct IlluminationLightImpl {
    common_impl: CommonFeatureImpl,
}

impl IlluminationLightImpl {
    fn new_fap_builder(&self, func_index: FuncIndex) -> FapBuilder {
        FapBuilder::new()
            .feature_index(self.common_impl.feature_index)
            .funcindex(func_index as u8)
    }
}

impl IlluminationLight for IlluminationLightImpl {
    fn get_illumination(
        &mut self,
        context: &mut Context,
    ) -> Result<IlluminationState, FeatureError> {
        let fap = self.new_fap_builder(FuncIndex::GetIllumination);
        let report = &self.common_impl.send_fap_command(context, fap)?;

        Ok(report.into())
    }

    fn set_illumination(
        &mut self,
        context: &mut Context,
        state: &IlluminationState,
    ) -> Result<(), FeatureError> {
        let params: [u8; 1] = state.into();
        let fap = self
            .new_fap_builder(FuncIndex::SetIllumination)
            .params(&params);
        self.common_impl.send_fap_command(context, fap)?;

        Ok(())
    }

    fn get_brightness_info(&mut self, context: &mut Context) -> Result<ControlInfo, FeatureError> {
        let fap = self.new_fap_builder(FuncIndex::GetBrightnessInfo);
        let report = &self.common_impl.send_fap_command(context, fap)?;

        Ok(report.into())
    }

    fn get_brightness(&mut self, context: &mut Context) -> Result<ControlValue, FeatureError> {
        let fap = self.new_fap_builder(FuncIndex::GetBrightness);
        let report = &self.common_impl.send_fap_command(context, fap)?;

        Ok(report.into())
    }

    fn set_brightness(
        &mut self,
        context: &mut Context,
        value: &ControlValue,
    ) -> Result<(), FeatureError> {
        let params: [u8; 2] = value.into();
        let fap = self
            .new_fap_builder(FuncIndex::SetBrightness)
            .params(&params);
        self.common_impl.send_fap_command(context, fap)?;

        Ok(())
    }

    fn get_brightness_levels(
        &mut self,
        context: &mut Context,
        req: &LevelsRequest,
    ) -> Result<ControlLevels, FeatureError> {
        let params: [u8; 1] = req.into();
        let fap = self
            .new_fap_builder(FuncIndex::GetBrightnessLevels)
            .params(&params);
        let report = &self.common_impl.send_fap_command(context, fap)?;

        Ok(report.into())
    }

    fn set_brightness_levels(
        &mut self,
        context: &mut Context,
        value: &ControlLevels,
    ) -> Result<(), FeatureError> {
        let params: [u8; 16] = value.into();
        let fap = self
            .new_fap_builder(FuncIndex::SetBrightnessLevels)
            .params(&params);
        self.common_impl.send_fap_command(context, fap)?;

        Ok(())
    }

    fn get_color_temperature_info(
        &mut self,
        context: &mut Context,
    ) -> Result<ControlInfo, FeatureError> {
        let fap = self.new_fap_builder(FuncIndex::GetColorTemperatureInfo);
        let report = &self.common_impl.send_fap_command(context, fap)?;

        Ok(report.into())
    }

    fn get_color_temperature(
        &mut self,
        context: &mut Context,
    ) -> Result<ControlValue, FeatureError> {
        let fap = self.new_fap_builder(FuncIndex::GetColorTemperature);
        let report = &self.common_impl.send_fap_command(context, fap)?;

        Ok(report.into())
    }

    fn set_color_temperature(
        &mut self,
        context: &mut Context,
        value: &ControlValue,
    ) -> Result<(), FeatureError> {
        let params: [u8; 2] = value.into();
        let fap = self
            .new_fap_builder(FuncIndex::SetColorTemperature)
            .params(&params);
        self.common_impl.send_fap_command(context, fap)?;

        Ok(())
    }

    fn get_color_temperature_levels(
        &mut self,
        context: &mut Context,
        req: &LevelsRequest,
    ) -> Result<ControlLevels, FeatureError> {
        let params: [u8; 1] = req.into();
        let fap = self
            .new_fap_builder(FuncIndex::GetColorTemperatureLevels)
            .params(&params);
        let report = &self.common_impl.send_fap_command(context, fap)?;

        Ok(report.into())
    }

    fn set_color_temperature_levels(
        &mut self,
        context: &mut Context,
        value: &ControlLevels,
    ) -> Result<(), FeatureError> {
        let params: [u8; 16] = value.into();
        let fap = self
            .new_fap_builder(FuncIndex::SetColorTemperatureLevels)
            .params(&params);
        self.common_impl.send_fap_command(context, fap)?;
        Ok(())
    }
}

pub fn init<'a>(
    context: &mut Context,
    root: &mut dyn Root,
) -> Result<Option<Box<dyn IlluminationLight + 'a>>, Box<dyn Error>> {
    let accessor = FeatureAccessor::new(root, FEATURE_ID)
        .with_context(context)
        .access_common_impl()?;
    Ok(accessor.map_or(None, |common_impl| {
        Some(Box::new(IlluminationLightImpl { common_impl }))
    }))
}
