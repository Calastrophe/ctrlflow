use std::fs::File;
use std::path::Path;
use std::thread;
use std::thread::JoinHandle;

use crate::{
    listener::EffectListener,
    register::{Info, RegisterInfo},
    AddressMode, Architecture, Effect, Error,
};
use bincode::serialize_into;
use flume::Sender;

/// TODO: DOCUMENTATION
pub struct Tracer<A: Architecture> {
    tx: EffectSender<A>,
    handle: JoinHandle<Result<(), Error>>,
}

impl<A: Architecture> Tracer<A> {
    /// Creates a new tracer which writes the log file to the given path.
    pub fn new<'a, P: AsRef<Path>>(
        path: P,
        init_mem: impl Iterator<Item = (&'a A::AddressWidth, &'a A::AddressWidth)>,
    ) -> Result<Self, Error> {
        let (tx, rx) = flume::unbounded();

        let mut file = File::create(path)?;

        populate_arch_info::<A>(&mut file)?;

        // Write the initial memory to the log file.
        for pair in init_mem {
            serialize_into(&mut file, &pair)?
        }

        let mut effect_listener: EffectListener<A> = EffectListener::new(rx, file);

        let handle = thread::spawn(move || effect_listener.listen());

        Ok(Self {
            tx: EffectSender(tx),
            handle,
        })
    }

    /// Clones the inner sender, increasing the sender count but allows for more effects to be sent
    /// separately from the tracer.
    pub fn sender(&self) -> EffectSender<A> {
        self.tx.clone()
    }

    /// Sends the passed effect to the receiver only returning [`Error::SendFailure`] if the tracer
    /// has unexpectedly returned an error.
    pub fn send(&self, effect: Effect<A>) -> Result<(), Error> {
        self.tx.send(effect).map_err(|_| Error::SendFailure)
    }

    /// Sends a [`Effect::Terminate`] to the tracer thread causing the thread to suspend and return
    /// after finishing its work.
    pub fn terminate(self) -> Result<(), Error> {
        self.send(Effect::Terminate)?;
        self.handle.join().unwrap()
    }
}

#[derive(Debug, Clone)]
/// A simple wrapper type around the internal sender.
pub struct EffectSender<A: Architecture>(Sender<Effect<A>>);

impl<A: Architecture> EffectSender<A> {
    /// Sends the passed effect to the receiver only returning [`Error::SendFailure`] if the tracer
    /// has unexpectedly returned an error.
    pub fn send(&self, effect: Effect<A>) -> Result<(), Error> {
        self.0.send(effect).map_err(|_| Error::SendFailure)
    }

    /// Allows access to the internal sender.
    pub fn inner_sender(&self) -> &Sender<Effect<A>> {
        &self.0
    }
}

/// Populate the architecture information into the log file.
fn populate_arch_info<A: Architecture>(file: &mut File) -> Result<(), Error> {
    #[derive(serde::Serialize)]
    struct ArchInfo<R: RegisterInfo> {
        mode: u8,
        registers: Vec<&'static Info<R>>,
    }

    let mode = A::AddressWidth::mode();

    let registers: Vec<_> = A::Register::iter().map(|r| r.info()).collect();

    let info = ArchInfo { mode, registers };

    let _ = serialize_into(file, &info)?;

    Ok(())
}
