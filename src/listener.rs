use crate::instruction::Info;
use crate::{Architecture, Error, Event};
use flume::{Receiver, TryRecvError};
use serde_json::to_writer_pretty;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};

/// The main workhorse for the [`crate::Tracer`], lives on a separate thread awaiting events being sent
/// over a channel.
///
/// This is responsible for handling all of the event logic along with serializing and logging said
/// events when the instruction has finished.
pub struct EventListener<A: Architecture> {
    /// The internal receiver which listens for events.
    receiver: Receiver<Event<A>>,
    /// The handle to the trace file.
    file: File,
    /// The current instruction being executed by the target architecture.
    insn: Option<(A::AddressWidth, A::Instruction)>,
    /// Holds all the events for the current instruction.
    events: Vec<Event<A>>,
}

impl<A: Architecture> EventListener<A> {
    /// Creates a new listener with the provided receiver and creates a handle to the trace file.
    pub fn new(receiver: Receiver<Event<A>>, file: File) -> Self {
        Self {
            receiver,
            file,
            insn: None,
            events: Vec::new(),
        }
    }

    /// Holds the main listening loop along with the loop which awaits for a [`Event::InsnStart`]
    /// or [`Event::Terminate`] to determine what to do.
    pub fn listen(&mut self) -> Result<(), Error> {
        loop {
            loop {
                if let Some(event) = self.read_event()? {
                    match event {
                        Event::InsnStart(addr, insn) => {
                            self.insn = Some((addr, insn));

                            break;
                        }
                        Event::Terminate => {
                            // We need to overwrite the trailing comma...
                            let _ = self.file.seek(SeekFrom::End(-2));

                            let _ = self.file.write(b"]\n}")?;
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }

            self.handle_events()?;
        }
    }

    /// Constantly reads every memory and register event and appends them to a vector until
    /// [`Event::InsnEnd`] is sent. At that point, the events are serialized, logged, and the vector
    /// is cleared.
    ///
    /// If another [`Event::InsnStart`] event were to be sent while in this loop, this will return [`Error::Ordering`].
    fn handle_events(&mut self) -> Result<(), Error> {
        loop {
            if let Some(event) = self.read_event()? {
                match event {
                    Event::InsnEnd => {
                        // NOTE: It is guaranteed that this should be Some to get to this point.
                        self.insn
                            .take()
                            .map(|(addr, insn)| {
                                let _ = to_writer_pretty(
                                    &mut self.file,
                                    &Info::new(addr, insn, &self.events),
                                );
                            })
                            .unwrap_or_else(|| unreachable!());

                        let _ = self.file.write(b",\n")?;

                        self.events.clear();

                        return Ok(());
                    }
                    Event::InsnStart(..) => return Err(Error::Ordering),
                    _ => self.events.push(event),
                }
            }
        }
    }

    /// Attempts to read an [`Event`] from the channel, only erroring in the event that all senders are dropped
    /// to the receiver.
    fn read_event(&mut self) -> Result<Option<Event<A>>, Error> {
        match self.receiver.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(Error::SendersDropped),
        }
    }
}
