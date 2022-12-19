use std::{error::Error, fmt::Display};

use crate::{
    context::{Context, WithContext},
    error::Hidpp20Error,
};

use self::{common::CommonFeatureImpl, root::RootFeature};

pub use self::devicetypename::DeviceTypeName;
pub use self::illuminationlight::IlluminationLight;
pub use self::root::Root;

mod common;
mod devicetypename;
mod illuminationlight;
mod root;
mod root_test;

#[derive(Debug)]
pub enum FeatureError {
    Internal(Box<dyn Error>),
    Hidpp20(Hidpp20Error),
}

impl Display for FeatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal(err) => {
                write!(f, "Internal error: ")?;
                err.fmt(f)?
            }
            Self::Hidpp20(err) => err.fmt(f)?,
        }

        std::fmt::Result::Ok(())
    }
}

impl Error for FeatureError {}

/// `FeatureAccessor` implements lookup of root feature by id and
/// constructs `CommonFeatureImpl`.
pub struct FeatureAccessor<'a> {
    context: Option<&'a mut Context>,
    root: &'a mut dyn Root,
    feature_id: u16,
}

impl<'a> FeatureAccessor<'a> {
    pub fn new(root: &'a mut dyn Root, feature_id: u16) -> Self {
        Self {
            context: None,
            root,
            feature_id,
        }
    }

    pub fn build_root_feature(self) -> Result<Option<RootFeature>, Box<dyn Error>> {
        let context = self
            .context
            .expect("Context needs to be set via `with` before calling `build_root_feature`");
        let root_feature = self
            .root
            .get_feature(context, self.feature_id)
            .map_err(|err| Box::new(err))?;
        Ok(if root_feature.feature_type.hidden {
            // TODO: Add debug
            None
        } else {
            Some(root_feature)
        })
    }

    /// Constructs struct with common code
    pub fn access_common_impl(self) -> Result<Option<CommonFeatureImpl>, Box<dyn Error>> {
        let root_feature = self.build_root_feature()?;
        Ok(root_feature.map_or(None, |v| {
            Some(CommonFeatureImpl {
                feature_index: v.feature_index,
            })
        }))
    }
}

impl<'a> WithContext<'a> for FeatureAccessor<'a> {
    fn with_context(mut self, context: &'a mut Context) -> Self {
        self.context = Some(context);
        self
    }
}

/// FeaturesCollector collects all known features
/// of a HID++ device
pub struct FeaturesCollector<'a> {
    context: Option<&'a mut Context>,
}

impl FeaturesCollector<'_> {
    pub fn new() -> Self {
        Self { context: None }
    }

    pub fn collect(&mut self) -> Result<Features, Box<dyn Error>> {
        let context = self
            .context
            .as_deref_mut()
            .expect("Context must have previously added to FeaturesCollector");
        let root_opt = root::init()?;
        let mut root = root_opt.expect("Expected Root Feature to always be constructible");
        let devicetypename = devicetypename::init(context, root.as_mut())?;
        let illuminationlight = illuminationlight::init(context, root.as_mut())?;
        Ok(Features {
            root,
            devicetypename,
            illuminationlight,
        })
    }
}

impl<'a> WithContext<'a> for FeaturesCollector<'a> {
    fn with_context(mut self, context: &'a mut Context) -> Self {
        self.context = Some(context);
        self
    }
}

pub struct Features {
    pub root: Box<dyn Root>,
    pub devicetypename: Option<Box<dyn DeviceTypeName>>,
    pub illuminationlight: Option<Box<dyn IlluminationLight>>,
}
