use crate::instruction::Info;
use crate::{Architecture, Effect, Error};
use bincode::serialize_into;
use flume::{Receiver, TryRecvError};
use std::fs::File;

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
    /// Creates a new listener with the provided receiver and creates a handle to the trace file.
    pub fn new(receiver: Receiver<Effect<A>>, file: File) -> Self {
        Self {
            receiver,
            file,
            effects: Vec::new(),
        }
    }

    /// Holds the main listening loop along with the loop which awaits for a [`Effect::InsnStart`]
    /// or [`Effect::Terminate`] to determine what to do.
    pub fn listen(&mut self) -> Result<(), Error> {
        loop {
            loop {
                if let Some(effect) = self.read_effect()? {
                    match effect {
                        Effect::InsnStart(addr, insn) => {
                            let info = Info::<A>::new(addr, insn);
                            serialize_into(&mut self.file, &info)?;

                            break;
                        }
                        Effect::Terminate => return Ok(()),
                        _ => {}
                    }
                }
            }

            self.handle_effects()?;
        }
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
                        serialize_into(&mut self.file, &self.effects)?;

                        self.effects.clear();

                        return Ok(());
                    }
                    Effect::InsnStart(..) => return Err(Error::Ordering),
                    _ => self.effects.push(effect),
                }
            }
        }
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
