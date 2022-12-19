use crate::report::HidppReport;

use super::sender::{ReportAsyncSender};

/// `ReportCollector` simply saves all `HidppReport`s into a `Vec`.
#[derive(Default)]
pub struct ReportCollector {
    /// All received reports
    pub reports: Vec<HidppReport>
}

impl ReportAsyncSender for ReportCollector {
    fn send(&mut self, report: &mut HidppReport) -> Result<(), Box<dyn std::error::Error>> {
        self.reports.push(report.clone());
        Ok(())
    }
}
