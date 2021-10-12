# blash-target

This provides RTT print and optionally a panic handler and exception handler to be used with https://github.com/bjoernQ/blash

Also see https://github.com/bjoernQ/bl602-rtt-example

The panic and exception handler are both feature gated:
- panic_backtrace
- exception_backtrace

If you don't get full backtraces you should compile the target application like `cargo build -Z build-std=core --target riscv32imc-unknown-none-elf` to make sure there is unwind info for all the code.
