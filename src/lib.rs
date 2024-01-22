//! ctrl-flow
#![forbid(rust_2018_idioms, unsafe_code)]
#![deny(missing_docs, unused_results, rustdoc::broken_intra_doc_links)]

use serde::Serialize;
use thiserror::Error;

pub use instruction::InstructionInfo;
pub use register::RegisterInfo;
pub use tracer::Tracer;

pub mod instruction;
mod listener;
pub mod register;
pub mod tracer;

/// The parent trait above all of its associated types which enables the ability to trace the
/// given architecture.
pub trait Architecture
where
    Self: Clone + Sized + 'static,
{
    /// The associated register type of the architecture
    type Register: RegisterInfo;
    /// The associated instruction type of the architecture
    type Instruction: InstructionInfo;
    /// The address width of the architecture
    type AddressWidth: AddressMode;
}

/// All of the possible errors which could result from the tracer.
#[derive(Debug, Error)]
pub enum Error {
    /// A [`Start`] effect was sent, but there wasn't a [`End`] effect to end the previous
    /// [`Start`].
    #[error("There was an attempt to send another Start effect before the previous was ended.")]
    Ordering,

    /// All of the senders for the stored receiver have been dropped.
    #[error("All of the effect senders were dropped.")]
    SendersDropped,

    /// A sender failed when attempting to send an effect to the receiver.
    #[error("There was an issue when sending an effect to the tracer.")]
    SendFailure,

    /// An issue was encountered when serializing the effects or storing the architecture
    /// information.
    #[error("There was an issue serializing an effect.")]
    Serialization(#[from] bincode::Error),

    /// IO Error
    #[error("There was an issue while creating or writing to the log file.")]
    IO(#[from] std::io::Error),
}

/// All of the given effects that can be communicated from the main emulator thread.
#[derive(Serialize, Debug, Clone)]
pub enum Effect<A: Architecture> {
    /// Indicates the starting of a given instruction at a given address.
    ///
    /// The instruction is laid out in the graph exactly as is it is serialized.
    InsnStart(A::AddressWidth, A::Instruction),

    /// Indicates the end of an instruction's effects.
    InsnEnd,

    /// Signals to the tracer thread that emulation has finished and to wrap up work.
    Terminate,

    /// Indicates a read has taken place for the given register.
    RegRead(A::Register),

    /// Indicates a write has taken place at a given register with a bytearray, serializes the
    /// value which was written as a bytearray in the log.
    RegWrite(A::Register, Box<[u8]>),

    /// Indicates that a read has taken place at a given memory address.
    MemRead(A::AddressWidth),

    /// Indicates that a write has taken place at a given memory address with a bytearray,
    /// serializes the value which was written as a bytearray in the log.
    MemWrite(A::AddressWidth, Box<[u8]>),
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
impl_address_mode!(u128, 16);
