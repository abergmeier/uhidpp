use crate::{
    context::Context,
    report::{FapBuilder, HidppReport},
};

use super::FeatureError;

pub struct CommonFeatureImpl {
    pub feature_index: u8,
}

impl CommonFeatureImpl {
    pub fn send_fap_command(
        &mut self,
        context: &mut Context,
        fap: FapBuilder,
    ) -> Result<HidppReport, FeatureError> {
        context
            .sender
            .send(fap)
            .map_err(|err| FeatureError::Internal(err))
    }
}
