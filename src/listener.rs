use crate::instruction::Info;
use crate::{Architecture, Event, Error};
use bincode::serialize_into;
use flume::{Receiver, TryRecvError};
use std::fs::File;

/// The main workhorse for the [`crate::Tracer`], lives on a separate thread awaiting events being sent
/// over a channel.
///
/// This is responsible for handling all of the event logic along with serializing and logging said
/// events when the instruction has finished.
pub struct EventListener<A: Architecture> {
    receiver: Receiver<Event<A>>,
    file: File,
    events: Vec<Event<A>>,
}

impl<A: Architecture> EventListener<A> {
    /// Creates a new listener with the provided receiver and creates a handle to the trace file.
    pub fn new(receiver: Receiver<Event<A>>, file: File) -> Self {
        Self {
            receiver,
            file,
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
                            let info = Info::<A>::new(addr, insn);
                            serialize_into(&mut self.file, &info)?;

                            break;
                        }
                        Event::Terminate => return Ok(()),
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
                        serialize_into(&mut self.file, &self.events)?;

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
