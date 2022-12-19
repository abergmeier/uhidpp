use std::error::Error;

use crate::{
    context::{Context, WithContext},
    features::{Features, FeaturesCollector},
};

/// Implementation of Builder pattern for `HidppDevice`s.
/// Uses `with` to attach a `Context`.
pub struct HidppDeviceBuilder<'a> {
    context: Option<&'a mut Context>,
}

impl<'a> WithContext<'a> for HidppDeviceBuilder<'a> {
    fn with_context(mut self, context: &'a mut Context) -> Self {
        self.context = Some(context);
        self
    }
}

impl<'a> HidppDeviceBuilder<'a> {
    pub fn new() -> Self {
        Self { context: None }
    }

    /// Build a `HidppDevice` out of the Context
    pub fn build(self) -> Result<HidppDevice, Box<dyn Error>> {
        let context = self
            .context
            .expect("Need to assign `Context` calling `with`");
        let features = FeaturesCollector::new().with_context(context).collect()?;
        Ok(HidppDevice { features })
    }
}

/// `HidppDevice` represents aspects of a Device.
pub struct HidppDevice {
    pub features: Features,
}
