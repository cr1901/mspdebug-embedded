# `mspdebug-embedded`

`mspdebug-embedded` is a wrapper library crate for the [`mspdebug`](https://github.com/dlbeer/mspdebug)
binary. The crate spawns an `mspdebug` process in [embedded mode](https://github.com/dlbeer/mspdebug/blob/master/EmbeddedMode.txt)
and uses [pipes](https://doc.rust-lang.org/std/process/struct.Stdio.html#method.piped)
to the child `mspdebug` for inter-process communication (IPC).

IPC allows `mspdebug-embedded` to take advantage of all the chips that `mspdebug`
supports without running afoul of the GPL. If `mspdebug` was compiled as a
library, this application would have to be GPL 2.0. While _I_ certainly don't
mind this, the Rust embedded ecosystem at large uses MIT, Apache or similar, and so
`mspdebug-embedded` follows precedent.

Right now, I am not providing a [crates.io](https://crates.io/) release of this
crate; the functionality is rather minimal and tailored to the

## `msprun`

`mspdebug` is extremely powerful, but it's more of a lower-level tool for
interacting with MSP430 microcontrollers. `msprun` is a driver _application_ 
that leverages the `mspdebug-embedded` crate to provide a higher-level interface
to lower-level `mspdebug` commands. As a side-effect, it is also a turnkey solution for
running msp430 applications using `cargo run`!  It occupies the same niche as
[`ravedude`](https://github.com/Rahix/avr-hal/tree/main/ravedude) for AVR.

### Installation

I'll provide binaries in the future, but for now:

```
cargo install --git https://github.com/cr1901/mspdebug-embedded --features=msprun
```

### Invocation

`msprun` is an application-in-progress, but two main commands are working:
* `prog`: Program an attached microcontroller via `mspdebug` given a filename.
* `gdb`: Start a `gdb` server via `mspdebug` for an attached microcontroller.
  Then, spawn an interactive `msp430-elf-gdb` session. `mspdebug` exits when
  `msp430-elf-gdb` exits.

The typical invocation is: `msprun mspdebug-driver [options] command [command-options] /path/to/elf`.
Help on options and commands are available via `msprun --help` or `msprun command --help`.

### Rationale on Commands

#### `prog`

If you attempt to use `mspdebug` directly as a [runner](https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner)
for the `msp430-none-elf` target, like the below [`config.toml`](https://doc.rust-lang.org/cargo/reference/config.html)
snippet, you quickly run into problems:

```toml
[target.'cfg(target_arch = "msp430")']
runner = "mspdebug rf2500 -q prog"
```

```sh
$ cargo +nightly run ...
Device: MSP430G2xx3
prog: you need to specify a filename
expand_tilde: getenv: The operation completed successfully.
error: process didn't exit successfully: `mspdebug rf2500 -q prog /path/to/elf` (exit code: 0xffffffff)
```

The _correct_ invocation is `mspdebug rf2500 -q 'prog /path/to/elf`. AFACT
`cargo` does not know how to handle anything more complicated than appending a
filename. Since all commands to `mspdebug` must be single-quoted, this
precludes using `mspdebug` directly as a `cargo` runner.

#### `gdb`

It is [possible](https://github.com/rust-embedded/cortex-m-quickstart/blob/master/.cargo/config.toml#L1)
to use an architecture-appropriate `gdb` as your `cargo` runner. `mspdebug`
has a built-in `gdb` server, and `msp430-elf-gdb` works just fine with it.
However, this often requires the `gdb` server and `gdb` to be invoked
separately, and in different terminals unless you want CTRL+C to be sent to
both the `gdb` server and `gdb` at the same time (_you probably don't_).
`msprun` contains logic to spawn `mspdebug` and `gdb` in the same terminal
_using a single binary_ so that only `gdb` receives CTRL+C events; `mspdebug`
exits when `msp430-none-elf` and/or `msprun` exits.

Both the [server](https://github.com/rust-embedded/cortex-m-quickstart/blob/master/openocd.cfg)
and [debugger](https://github.com/rust-embedded/cortex-m-quickstart/blob/master/openocd.gdb)
require scripts to set them up to talk to each other. A decent chunk of the gdb server and debugger setup, like e.g. `gdb`'s `target remote`, can be automated and specified on the command-line. `msprun` takes
care of setting up `mspdebug` in server mode and initial `msp430-elf-gdb` setup
for you without any _required_ scripts.
