use std::{
    fs::File,
    path::Path,
    sync::mpsc::{self, Receiver},
};

use crate::{
    io::{
        EventIterator, EventReader, FapCommandEventReceiver, FapCommandSyncSender,
        ReportAsyncSenderImpl,
    },
    report::HidppReport,
};

/// `Context` is the current device Context.
pub struct Context {
    pub(crate) sender: FapCommandSyncSender<ReportAsyncSenderImpl, EventIterator>,
    pub(crate) broadcasts_receiver: Receiver<HidppReport>,
}

impl Context {
    pub fn from_path<P>(hiddev_path: P) -> Context
    where
        P: AsRef<Path>,
    {
        let hiddev = File::open(hiddev_path).expect("Could not open provided path");
        Self::from_file(hiddev)
    }

    pub fn from_file(hiddev: File) -> Context {
        let sender_file = hiddev;

        let event_file = sender_file
            .try_clone()
            .expect("Cloning of File reference failed");
        let events = EventReader {
            hid_dev: event_file,
        };
        let sender = ReportAsyncSenderImpl::from_file(sender_file)
            .expect("Creating async Sender for file failed");
        let (broadcasts_sender, broadcasts_receiver) = mpsc::sync_channel(0);
        let processor = FapCommandEventReceiver::new(events.into_iter(), broadcasts_sender);
        let sender = FapCommandSyncSender::new(sender, processor);
        Context {
            sender,
            broadcasts_receiver,
        }
    }
}

/// `WithContext` is the implementation trait that Builders should implement.
/// Unless of course they are contextless.
pub trait WithContext<'a> {
    fn with_context(self, context: &'a mut Context) -> Self;
}
