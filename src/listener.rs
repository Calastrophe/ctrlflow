use crate::{Architecture, Effect, Error};
use flume::{Receiver, TryRecvError};
use std::fs::File;
use std::io::Write;

/// The main workhorse for the [`crate::Tracer`], lives on a separate thread awaiting effects being sent
/// over a channel.
///
/// This is responsible for handling all of the effect logic along with serializing and logging
/// after an instruction's effects have ended.
pub struct EffectListener<A: Architecture> {
    receiver: Receiver<Effect<A>>,
    file: File,
    effects: Vec<Effect<A>>,
}

impl<A: Architecture> EffectListener<A> {
    /// Creates a new listener with the provided receiver and creates a handle to the log file.
    pub fn new(receiver: Receiver<Effect<A>>, file: File) -> Self {
        Self {
            receiver,
            file,
            effects: Vec::new(),
        }
    }

    /// The main listening loop which internally calls [await_start()] and [handle_effects()]
    pub fn listen(&mut self) -> Result<(), Error> {
        loop {
            self.await_start()?;

            self.handle_effects()?;
        }
    }

    /// Constantly reads incoming effects
    fn await_start(&mut self) -> Result<(), Error> {
        loop {
            if let Some(effect) = self.read_effect()? {
                match effect {
                    Effect::InsnStart(..) => {
                        self.file.write(&bincode::serialize(&effect)?)?;

                        break;
                    }
                    Effect::Terminate => break,
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Constantly reads every memory and register effect and appends them to a vector until the
    /// [`Effect::InsnEnd`] effect is sent. At that point, the effects are serialized, logged, and the vector
    /// is cleared.
    ///
    /// If another [`Effect::InsnStart`] effect were to be sent while in this loop, this will return [`Error::Ordering`].
    fn handle_effects(&mut self) -> Result<(), Error> {
        loop {
            if let Some(effect) = self.read_effect()? {
                match effect {
                    Effect::InsnEnd => {
                        self.file.write(&bincode::serialize(&self.effects)?)?;

                        self.effects.clear();

                        break;
                    }
                    Effect::InsnStart(..) => return Err(Error::Ordering),
                    _ => self.effects.push(effect),
                }
            }
        }

        Ok(())
    }

    /// Attempts to read an [`Effect`] from the channel, only erroring in the event that all senders are dropped
    /// to the receiver.
    fn read_effect(&mut self) -> Result<Option<Effect<A>>, Error> {
        match self.receiver.try_recv() {
            Ok(effect) => Ok(Some(effect)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(Error::SendersDropped),
        }
    }
}
