use serde::Serialize;

use crate::Architecture;

/// All of the required methods for instructions which have a branching effect in the target
/// architecture.
pub trait InstructionInfo: Serialize + Send {
    /// Returns the size of the instruction, this is **only** required for instructions which
    /// return a [`JumpKind`]. It will be ignored by the serializer otherwise.
    fn size(&self) -> Option<u16>;

    /// If the instruction has any branching effect on the emulator, this should return a
    /// [`JumpKind`] otherwise it should return [`None`].
    fn kind(&self) -> Option<JumpKind>;
}

#[derive(Serialize)]
/// Indicates that an instruction has a branching effect in the target architecture.
pub enum JumpKind {
    /// The instruction calls another function.
    Call,
    /// The instruction returns from the current function.
    Return,
    /// The instruction jumps to another basic block in the function unconditionally.
    Unconditional,
    /// The instruction potentially jumps to another basic block or moves to next instruction.
    Conditional,
}

/// Internally used for serializing instructions with needed size and kind types.
#[derive(serde::Serialize)]
pub(crate) struct Info<A: Architecture> {
    addr: A::AddressWidth,
    insn: A::Instruction,
    size: Option<u16>,
    kind: Option<JumpKind>,
}

impl<A: Architecture> Info<A> {
    pub fn new(addr: A::AddressWidth, insn: A::Instruction) -> Self {
        let size = insn.size();
        let kind = insn.kind();

        Info {
            addr,
            insn,
            size,
            kind,
        }
    }
}
