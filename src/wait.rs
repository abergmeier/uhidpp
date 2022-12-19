use std::{
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use crate::report::HidppReport;

pub(crate) fn wait_event_timeout(
    answers: Receiver<HidppReport>,
    timeout: Duration,
) -> Result<HidppReport, RecvTimeoutError> {
    answers.recv_timeout(timeout)
}
