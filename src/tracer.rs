use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::thread::JoinHandle;

use crate::{
    listener::EventListener,
    register::{Info, RegInfo},
    AddressMode, Architecture, Error, Event,
};
use flume::Sender;
use serde_json::to_writer_pretty;

/// The actual tracer which can is generic over any type which implements [`Architecture`],
/// allowing it to set up a trace file and be ready for events specific to the target
/// architecture.
pub struct Tracer<A: Architecture> {
    tx: EventSender<A>,
    handle: JoinHandle<Result<(), Error>>,
}

impl<A: Architecture> Tracer<A> {
    /// Creates a new tracer which writes a trace file to the given path.
    pub fn new<'a, P: AsRef<Path>>(
        path: P,
        init_mem: impl Iterator<Item = (&'a A::AddressWidth, &'a A::AddressWidth)>,
    ) -> Result<Self, Error> {
        let (tx, rx) = flume::unbounded();

        let mut file = File::create(path)?;

        #[derive(serde::Serialize)]
        struct ArchInfo<'a, A: AddressMode, R: RegInfo> {
            registers: Vec<&'static Info<R>>,
            memory: Vec<(&'a A, &'a A)>,
        }

        let registers: Vec<_> = A::Register::iter().map(|r| r.info()).collect();

        let _ = file.write(b"{\n\"info\": ")?;

        let info = ArchInfo {
            registers,
            memory: Vec::from_iter(init_mem),
        };

        let _ = to_writer_pretty(&file, &info)?;

        let _ = file.write(b",\n\"instructions\": [\n")?;

        let mut event_listener: EventListener<A> = EventListener::new(rx, file);

        let handle = thread::spawn(move || event_listener.listen());

        Ok(Self {
            tx: EventSender(tx),
            handle,
        })
    }

    /// Clones the inner sender, increasing the sender count but allows for more events to be sent
    /// separately from the tracer.
    pub fn sender(&self) -> EventSender<A> {
        self.tx.clone()
    }

    /// Sends the passed event to the receiver only returning [`Error::SendFailure`] if the tracer
    /// has unexpectedly returned an error.
    pub fn send(&self, event: Event<A>) -> Result<(), Error> {
        self.tx.send(event).map_err(|_| Error::SendFailure)
    }

    /// Sends a [`Event::Terminate`] to the tracer thread causing the thread to suspend and return
    /// after finishing its work.
    pub fn terminate(self) -> Result<(), Error> {
        self.send(Event::Terminate)?;
        self.handle.join().unwrap()
    }
}

#[derive(Debug, Clone)]
/// A simple wrapper type around the internal sender.
pub struct EventSender<A: Architecture>(Sender<Event<A>>);

impl<A: Architecture> EventSender<A> {
    /// Sends the passed event to the receiver only returning [`Error::SendFailure`] if the tracer
    /// has unexpectedly returned an error.
    pub fn send(&self, event: Event<A>) -> Result<(), Error> {
        self.0.send(event).map_err(|_| Error::SendFailure)
    }

    /// Allows access to the internal sender.
    pub fn inner_sender(&self) -> &Sender<Event<A>> {
        &self.0
    }
}
