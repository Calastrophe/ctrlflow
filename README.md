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

The future goal of this project is to largely build up tooling to have something similar to [QIRA](https://github.com/geohot/qira) and focus on one specific emulator (not strictly QEMU).

However, due to the current limitations of the QEMU plugin system, which restrict the ability to see exact values written to a specific address or register, it would require a custom version of QEMU.

Another option, which is the ideal solution, involves leveraging [SLEIGH](https://grant-h.github.io/docs/ghidra/decompiler/sleigh.html) which allows for individual processor instructions to be translated into [p-code operations](https://spinsel.dev/assets/2020-06-17-ghidra-brainfuck-processor-1/ghidra_docs/language_spec/html/pcoderef.html).

Which in turn, such p-code operations are ran in an interpreter, which emulate the target architecture environment.

There are some emulators which do this already, such as [icicle-emu](https://github.com/icicle-emu/icicle-emu), but they are largely immature in their implementation and allowing of instrumentation.



