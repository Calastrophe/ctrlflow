## What is ctrlflow?

**ctrlflow** is an **experimental "proof-of-concept"** architecture agnostic tracing library for emulators.

This library aims to provide an intuitive framework to easily create trace files for the user's target architecture.

These trace files in turn can be dropped into a separate tool which parses it and provides *timeless* debugging functionality.

## How does it work?

There is another thread, separate from the emulator, which is constantly listening on a channel which use events as messages.

These events are broken down as such,

```rs
enum Event<A :Architecture> {
    InsnStart(A::AddressWidth, A::Instruction),
    InsnEnd,
    Terminate,
    RegRead(A::Register),
    RegWrite(A::Register, Box<[u8]>),
    MemRead(A::AddressWidth),
    MemWrite(A::AddressWidth, A::AddressWidth),
}
```

Essentially, the tracer thread is awaiting an `InsnStart` then starts recording effects.

These effects are restricted to register and memory read/writes which are sent from the emulator's assumed wrapper type for registers and memory.

Once the instruction has finished executing, the emulator signals to the tracer that by sending `InsnEnd` over the channel.

Then, all the information gathered will be serialized and logged into the trace file for later analysis by an external tool.


## What are the future goals?

The future goal of this project is to largely build up tooling to have something similar to [QIRA](https://github.com/geohot/qira) and focus on one specific emulator.

However, with the current limitations of the QEMU plugin system, it would require a manual patch of the source in QEMU.

Another key feature would be to detect loops, these can quickly fill useless effects into trace files. ( This is possible, just takes a bit of analysis. )

