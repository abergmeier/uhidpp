use std::{
    error::Error,
    fs::File,
    os::unix::prelude::AsRawFd,
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Mutex,
    },
    time::Duration,
};

use crate::{
    error::{Hidpp20Error, HIDPP20_ERROR_FEATURE_INDEX},
    hid::{HIDIOCGDEVINFO, HIDIOCSUSAGE, HIDIOCSREPORT, hiddev, self},
    report::{self, FapBuilder, HidppReport, REPORT_ID_HIDPP_LONG, REPORT_ID_HIDPP_VERY_LONG},
    wait::wait_event_timeout, usb,
};

use crate::io::receiver::{AttachedPendingAnswer, FapCommandEventReceiver, PendingAnswer};

/// ReportAsyncSender sends reports and does
/// not wait for answers
pub trait ReportAsyncSender {
    fn send(&mut self, report: &mut HidppReport) -> Result<(), Box<dyn Error>>;
}

struct AttachedReportAsyncSender<'a, EVS>
where
    EVS: Iterator<Item = HidppReport>,
{
    _attached: AttachedPendingAnswer<'a, EVS>,
    wrapping: &'a mut dyn ReportAsyncSender,
}

impl<'a, EVS> AttachedReportAsyncSender<'a, EVS>
where
    EVS: Iterator<Item = HidppReport>,
{
    pub fn attach_pending_answer(
        receiver: &'a FapCommandEventReceiver<EVS>,
        message: HidppReport,
        answer_sender: SyncSender<HidppReport>,
        wrapping: &'a mut dyn ReportAsyncSender,
    ) -> Self {
        Self {
            _attached: AttachedPendingAnswer::new(
                receiver,
                PendingAnswer {
                    question: message,
                    answer: answer_sender,
                },
            ),
            wrapping,
        }
    }
}

impl<EVS> ReportAsyncSender for AttachedReportAsyncSender<'_, EVS>
where
    EVS: Iterator<Item = HidppReport>,
{
    fn send(&mut self, report: &mut HidppReport) -> Result<(), Box<dyn Error>> {
        self.wrapping.send(report)
    }
}

/// `send_and_drop` sends the report via `sender` and then drops `sender`
fn send_and_drop<EVS>(
    mut sender: AttachedReportAsyncSender<'_, EVS>,
    report: &mut HidppReport,
) -> Result<(), Box<dyn Error>>
where
    EVS: Iterator<Item = HidppReport>,
{
    sender.send(report)
}

/// `FapCommandSyncSender` sends _FAP_ commands/reports and
/// waits for the corresponding answer/report.
/// Similar to `hidpp_send_fap_command_sync`.
pub struct FapCommandSyncSender<RAS, EVS>
where
    RAS: ReportAsyncSender,
    EVS: Iterator<Item = HidppReport>,
{
    processor: FapCommandEventReceiver<EVS>,
    sending: Mutex<()>,
    sender: RAS,
}

impl<RAS, EVS> FapCommandSyncSender<RAS, EVS>
where
    RAS: ReportAsyncSender,
    EVS: Iterator<Item = HidppReport>,
{
    pub fn new(sender: RAS, processor: FapCommandEventReceiver<EVS>) -> Self {
        Self {
            processor,
            sender,
            sending: Mutex::new(()),
        }
    }

    pub fn send(&mut self, fap: FapBuilder) -> Result<HidppReport, Box<dyn Error>> {
        let _mutex = self.sending.lock().unwrap();

        let mut message = HidppReport::new(fap);

        let (channel_sender, channel_receiver) = mpsc::sync_channel(0);
        let answer = {
            let attached = AttachedReportAsyncSender::attach_pending_answer(
                &self.processor,
                message.clone(),
                channel_sender,
                &mut self.sender,
            );
            send_message(attached, &mut message, channel_receiver)?
        };
        Ok(answer)
    }
}

pub struct ReportAsyncSenderImpl {
    hid_raw: File,
    app_index: u32,
}

impl ReportAsyncSenderImpl {
    pub fn from_file(hid_dev: File) -> nix::Result<Self> {
        let fd = hid_dev.as_raw_fd();
        let mut dev_info = hiddev::Devinfo::default();
        unsafe { HIDIOCGDEVINFO(fd, &mut dev_info) }?;

        assert_eq!(dev_info.vendor, usb::VendorId::Logitech as i16);
        assert_eq!(dev_info.product, usb::DeviceId::LogitechLitraGlow as i16);
        // TODO: log version

        // TODO: make usage dynamic
        let app_index = hiddev::find_application_collection(fd, &dev_info, 0xFF43)?;

        Ok(Self {
            hid_raw: hid_dev,
            app_index,
        })
    }
}

impl ReportAsyncSender for ReportAsyncSenderImpl {
    fn send(&mut self, hidpp_report: &mut HidppReport) -> Result<(), Box<dyn Error>> {
        // set the device_index as the receiver, it will be overwritten by
        // hid_hw_request if needed
        hidpp_report.device_index = 0xff;

        let fd = self.hid_raw.as_raw_fd();

        for (i, value) in hidpp_report.fap.params[..report::LONG_LENGTH.into()]
            .iter()
            .enumerate()
        {
            let usage = hiddev::UsageRef {
                field_index: i as u32,
                report_id: hidpp_report.report_id.into(),
                report_type: hid::ReportType::Output.into(),
                usage_code: 0x02,
                usage_index: 0,
                value: (*value).into(),
            };
            unsafe { HIDIOCSUSAGE(fd, &usage) }?;
        }

        let data = hiddev::ReportInfo {
            num_fields: report::LONG_LENGTH.into(),
            report_id: hidpp_report.report_id.into(),
            report_type: hid::ReportType::Output.into(),
        };
        unsafe { HIDIOCSREPORT(fd, &data) }?;
        Ok(())
    }
}

/// `send_message` sends a message via `report_sender`.
fn send_message<EVS>(
    report_sender: AttachedReportAsyncSender<'_, EVS>,
    message: &mut HidppReport,
    answers: Receiver<HidppReport>,
) -> Result<HidppReport, Box<dyn Error>>
where
    EVS: Iterator<Item = HidppReport>,
{
    let response = HidppReport::default();
    //
    // So that we can later validate the answer when it arrives
    // in hidpp_raw_event
    //
    send_and_drop(report_sender, message).map_err(|err| {
        log::debug!("Sending report failed: {}", err);
        err
    })?;

    let timeout = Duration::new(5, 0);
    let result = wait_event_timeout(answers, timeout).map_err(|err| {
        log::debug!("Sending report timed out.");
        Box::new(err)
    })?;

    if (response.report_id == REPORT_ID_HIDPP_LONG
        || response.report_id == REPORT_ID_HIDPP_VERY_LONG)
        && response.fap.feature_index == HIDPP20_ERROR_FEATURE_INDEX
    {
        let err: Hidpp20Error = response.fap.params[1]
            .try_into()
            .expect("Could not convert answer to HIDPP20Error");
        log::debug!("Got HID++ 2.0 error: {}", err);
        return Err(Box::new(err));
    }

    Ok(result)
}
