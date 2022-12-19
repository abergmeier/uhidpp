use std::{
    fs::File,
    io::Read,
    sync::{mpsc::SyncSender, Mutex},
};

use crate::report::HidppReport;
use crate::{error::HIDPP20_ERROR_FEATURE_INDEX, report};

/// `AttachedPendingAnswer` sets information for sent question
/// and pending answer on `FapCommandEventReceiver`.
/// When it gets droppped, information gets unset again.
pub(crate) struct AttachedPendingAnswer<'a, EVS>
where
    EVS: Iterator<Item = HidppReport>,
{
    receiver: &'a FapCommandEventReceiver<EVS>,
}

impl<'a, EVS> AttachedPendingAnswer<'a, EVS>
where
    EVS: Iterator<Item = HidppReport>,
{
    pub(crate) fn new(receiver: &'a FapCommandEventReceiver<EVS>, pending: PendingAnswer) -> Self {
        receiver.attach_pending_answer(pending);
        Self { receiver }
    }
}

impl<EVS> Drop for AttachedPendingAnswer<'_, EVS>
where
    EVS: Iterator<Item = HidppReport>,
{
    fn drop(&mut self) {
        self.receiver.detach_pending_answer();
    }
}

pub(crate) struct PendingAnswer {
    pub(crate) question: HidppReport,
    pub(crate) answer: SyncSender<HidppReport>,
}

/// `FapCommandEventReceiver` processes all `HidppReports`.
/// For ASEs scenarios it is possible to enable
/// sending answers via `Sender`. See `attach_pending_answer`.
pub struct FapCommandEventReceiver<EVS>
where
    EVS: Iterator<Item = HidppReport>,
{
    events: EVS,
    current: Mutex<Option<PendingAnswer>>,
    broadcasts: SyncSender<HidppReport>,
}

impl<EVS> FapCommandEventReceiver<EVS>
where
    EVS: Iterator<Item = HidppReport>,
{
    pub fn new(events: EVS, broadcasts: SyncSender<HidppReport>) -> Self {
        Self {
            broadcasts,
            current: Mutex::new(None),
            events,
        }
    }

    pub(crate) fn attach_pending_answer(&self, answer: PendingAnswer) {
        let mut current = self.current.lock().unwrap();
        *current = Some(answer);
    }

    pub(crate) fn detach_pending_answer(&self) {
        let mut current = self.current.lock().unwrap();
        *current = None;
    }

    pub fn receive(&mut self) {
        loop {
            let next = self.events.next();
            if next.is_none() {
                break;
            }
            self.process_event(next.unwrap());
        }
    }

    fn process_event(&self, report: HidppReport) {
        let current = self.current.lock().unwrap();

        if current.is_some() {
            let communication = current
                .as_ref()
                .expect("Corrupted internal state - is_some was true before");
            let question = &communication.question;
            // If a sender is present, then we have a pending answer from a
            // previously sent command.

            // Check for a correct hidpp20 answer or the corresponding
            // error
            if match_answer(question, &report) || match_error(question, &report) {
                return self.process_ase(communication, report);
            }
        }

        if report.has_sw_id() {
            // Seems like an answer for another software
            return;
        }

        return self.process_broadcast(report);
    }

    fn process_ase(&self, communication: &PendingAnswer, report: HidppReport) {
        // This was an answer to a command that this driver sent
        communication.answer.send(report).expect("Failure of process_ase not handled gracefully yet");
    }

    fn process_broadcast(&self, report: HidppReport) {
        self.broadcasts.send(report).expect("Failure of process_broadcast not handled gracefully yet");
    }
}

fn match_answer(question: &HidppReport, answer: &HidppReport) -> bool {
    (answer.fap.feature_index == question.fap.feature_index)
        && (answer.fap.funcindex_clientid == question.fap.funcindex_clientid)
}

fn match_error(question: &HidppReport, answer: &HidppReport) -> bool {
    (answer.fap.feature_index == HIDPP20_ERROR_FEATURE_INDEX)
        && (answer.fap.funcindex_clientid == question.fap.feature_index)
        && (answer.fap.params[0] == question.fap.funcindex_clientid)
}

/// `EventIterator` exposes all events of a HIDDEV character device via `Iterator`.
pub struct EventIterator {
    pub(crate) hid_dev: File,
}

impl Iterator for EventIterator {
    type Item = HidppReport;

    /// Tries to read next entry from HID raw.
    fn next(&mut self) -> Option<Self::Item> {
        let mut data: [u8; report::VERY_LONG_MAX_LENGTH as usize] =
            [0; report::VERY_LONG_MAX_LENGTH as usize];
        let size = self.hid_dev.read(&mut data).unwrap();
        if size == 0 {
            return None;
        }

        let borrowed_data = &data;
        return Some(borrowed_data.into());
    }
}

/// `EventReader` reads events from HIDDEV devices.
pub struct EventReader {
    pub hid_dev: File,
}

impl EventReader {
    pub fn into_iter(&self) -> EventIterator {
        EventIterator {
            hid_dev: self.hid_dev.try_clone().expect("Cloning of File reference failed"),
        }
    }
}
