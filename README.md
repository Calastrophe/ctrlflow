## What is ctrl-flow?

**ctrl-flow** is an **experimental** architecture agnostic tracing library for emulators.

Rather than requiring the user to implement their architecture's instruction semantics in this library, we take the approach of passing an `Effect` type over a channel.

An `Effect` is broken down as such,

```rs

enum Effect<A :Architecture> {
    InsnStart(A::AddressWidth, A::Instruction),
    InsnEnd,
    Terminate,
    RegRead(A::Register),
    RegWrite(A::Register, Box<[u8]>),
    MemRead(A::AddressWidth),
    MemWrite(A::AddressWidth, Box<[u8]>),
}

```

The read and write effects are sent from the target emulator's assumed wrapper type for registers and memory.

So, a short and primitive example would look like,

```rs

enum Register {
    R0,
    R1
}

struct Registers {
    registers: [u64; 2],
    tx: EffectSender<Arch>,
}

impl Registers {
    fn read(&self, reg: Register) -> u64 {
        self.tx.send(Effect::RegRead(reg));
        self.registers[reg as usize];
    }

    fn write(&mut self, reg: Register, val: u64) -> u64 {
        self.tx.send(Effect::RegWrite(reg, Box::from(val.to_le_bytes())));
        self.registers[reg as usize] = val;
    }
}

```

This example is incredibly simple, but we leave it up to the implementer to determine how they want it structured - it could be either as complicated or simple as you want.



