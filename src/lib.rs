//! ctrl-flow
#![forbid(rust_2018_idioms, unsafe_code)]
#![deny(missing_docs, unused_results, rustdoc::broken_intra_doc_links)]

use serde::Serialize;
use thiserror::Error;

pub use instruction::InsnInfo;
pub use register::RegInfo;
pub use tracer::Tracer;

/// A module holding the [`InstructionInfo`] trait and associated items.
pub mod instruction;
mod listener;
/// A module holding the [`RegisterInfo`] trait and associated items.
pub mod register;
/// A module holding the [`Tracer`] and associated items.
pub mod tracer;

/// The parent trait above all of its associated types which enables the ability to trace the
/// given architecture.
pub trait Architecture
where
    Self: Serialize + Clone + Sized + 'static,
{
    /// The associated register type of the architecture
    type Register: RegInfo;

    /// The associated instruction type of the architecture
    type Instruction: InsnInfo;

    /// The address width of the architecture
    type AddressWidth: AddressMode;
}

/// All of the possible errors which could result from the tracer.
#[derive(Debug, Error)]
pub enum Error {
    /// [`Event::InsnStart`] was sent, but there wasn't a [`Event::InsnEnd`] to end the preceding
    /// instruction's effect(s).
    #[error("There was an attempt to send another Start effect before the previous was ended.")]
    Ordering,

    /// All of the senders for the stored receiver have been dropped.
    #[error("All of the effect senders were dropped.")]
    SendersDropped,

    /// A sender failed when attempting to send an effect to the tracing thread.
    #[error("There was an issue when sending an effect to the tracer.")]
    SendFailure,

    /// An issue was encountered when serializing the effects or storing the architecture
    /// information.
    #[error("There was an issue serializing an effect.")]
    Serialization(#[from] serde_json::Error),

    /// IO Error
    #[error("There was an issue while creating or writing to the trace file.")]
    IO(#[from] std::io::Error),
}

/// All of the given effects that can be communicated from the main emulator thread.
#[derive(Serialize, Debug, Clone)]
pub enum Event<A: Architecture> {
    /// Indicates the starting of a given instruction at a given address.
    InsnStart(A::AddressWidth, A::Instruction),

    /// Indicates the end of an instruction's effects.
    InsnEnd,

    /// Signals to the tracer thread that emulation has finished and to wrap up work.
    Terminate,

    /// Indicates that a register read has taken place for a given register.
    RegRead(A::Register),

    /// Indicates that a register write has taken place for a given register.
    RegWrite(A::Register, Box<[u8]>),

    /// Indicates that a memory read has taken place at a given memory address.
    MemRead(A::AddressWidth),

    /// Indicates that a memory write has taken place at a given memory address.
    MemWrite(A::AddressWidth, A::AddressWidth),
}

/// Determines the size of memory addresses for the given architecture in bytes.
pub trait AddressMode: Serialize + Send {
    /// Returns the addressing mode size in bytes.
    fn mode() -> u8;
}

macro_rules! impl_address_mode {
    ($type: ty, $size: expr) => {
        impl AddressMode for $type {
            fn mode() -> u8 {
                $size
            }
        }
    };
}

impl_address_mode!(u8, 1);
impl_address_mode!(u16, 2);
impl_address_mode!(u32, 4);
impl_address_mode!(u64, 8);
