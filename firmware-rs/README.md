# Fennec firmware (Rust, experimental)

This is an experimental Rust version of the Fennec firmware, developed alongside the team's C/STM32CubeMX firmware in [`../firmware/`](../firmware/). Nothing here touches the C project.

It currently contains a **blinky** for the **STM32F4-Discovery** dev board (STM32F407VG). The blinky exists to prove that building, flashing, debugging and emulation all work. The Discovery is a stand-in: **it is not the final Fennec MCU**, which the team hasn't chosen yet.

Built with [Embassy](https://embassy.dev) (async HAL and executor), [defmt](https://defmt.ferrous-systems.com) (logging) and [probe-rs](https://probe.rs) (flashing), and emulated with [Renode](https://renode.io).

## Quick start

Run all commands from this folder (`firmware-rs/`).

```bash
# On the real board (plugged in over USB):
cargo run --bin blinky

# In the Renode emulator (no hardware needed):
cargo run-emu --bin blinky

# Automated emulator tests:
cargo build-emu && renode-test renode/tests/blinky.robot
```

First time? Do [Setup](#setup) first.

---

## Setup

These steps are for Ubuntu 24.04 (x86_64). You only need to do them once.

### 1. Rust toolchain

Install Rust from <https://rustup.rs> if you don't have it. Then:

```bash
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools
cargo install probe-rs-tools flip-link cargo-binutils
```

`rust-toolchain.toml` also asks rustup for the target and components automatically, but `cargo install` must be run by hand.

| Tool | What it's for |
|---|---|
| `thumbv7em-none-eabihf` | Compile target for the Cortex-M4F with hardware floating point |
| `probe-rs` | Flashes the board and shows defmt logs (used by `cargo run`) |
| `flip-link` | Linker wrapper that turns stack overflows into a clean crash |
| `cargo-binutils` | `cargo size`, `cargo nm`, `cargo objdump` |

### 2. Board access (hardware only)

Linux needs permission to talk to the board's ST-LINK without `sudo`. Ubuntu's `openocd` package installs udev rules that usually cover this (`sudo apt install openocd`). If `probe-rs list` shows nothing while the board is plugged in, follow the probe-rs [probe setup guide](https://probe.rs/docs/getting-started/probe-setup/).

### 3. Renode (emulation only)

Download `renode_<version>_amd64.deb` from the [Renode releases](https://github.com/renode/renode/releases) and install it:

```bash
sudo apt install ./renode_<version>_amd64.deb
```

The automated tests need some Python packages. Ubuntu 24.04 requires a virtual environment for these:

```bash
python3 -m venv ~/.venvs/renode
~/.venvs/renode/bin/pip install -r /opt/renode/tests/requirements.txt
```

Activate it (`source ~/.venvs/renode/bin/activate`) in any terminal where you run `renode-test`.

### 4. Optional: OpenOCD + GDB

Only needed for the GDB fallback below:

```bash
sudo apt install openocd gdb-multiarch
```

---

## Running on the STM32F4-Discovery

1. Plug the board in through its **mini-USB** port (the ST-LINK side, CN1).
2. Check that the probe is detected:
   ```bash
   probe-rs list
   ```
3. Build, flash and watch the logs:
   ```bash
   cargo run --bin blinky
   ```

The green LED (LD4) should blink once per second, and the terminal should print:

```
0.000000 [INFO ] Fennec blinky starting
0.000061 [INFO ] blink 1
0.500183 [INFO ] blink 2
```

Ctrl+C stops the log viewer, but the board keeps running. The firmware is stored in flash, so it restarts on its own after a reset or power cycle.

To watch logs from a board that's already running, without reflashing:

```bash
probe-rs attach --chip STM32F407VGTx target/thumbv7em-none-eabihf/debug/blinky
```

The ELF must be the same build that's on the board, because log text is decoded from it.

### Release builds

```bash
cargo run --release --bin blinky    # smaller and faster
cargo size --bin blinky             # flash (text + data) and RAM (data + bss) usage
```

---

## Running in Renode

Renode runs the **emulation build**. It's the same firmware, except logs are sent as plain text on USART2 instead of through the debug probe, because Renode can't read defmt's RTT output. Emulation builds go to `target/emulation/` so they're never mixed up with hardware builds.

> **Never flash an emulation build to the real device.**

### Interactive

```bash
cargo run-emu --bin blinky
```

This builds the emulation version and starts Renode with it. A **usart2** window shows the log lines:

```
0.000 INFO Fennec blinky starting
0.000 INFO blink 1
0.500 INFO blink 2
1.000 INFO blink 3
```

Equivalent manual steps:

```bash
cargo build-emu
renode renode/blinky.resc
```

Useful commands in the Renode monitor:

```
(discovery) sysbus.gpioPortD.UserLED State   # green LED: True / False
(discovery) pause
(discovery) start
(discovery) machine Reset
(discovery) quit
```

### Headless (no windows, e.g. over SSH)

```bash
cargo build-emu
renode --console --disable-xwt -e 'mach create "discovery"; machine LoadPlatformDescription @renode/discovery.repl; sysbus LoadELF @target/emulation/thumbv7em-none-eabihf/debug/blinky; sysbus.usart2 CreateFileBackend @/tmp/fennec-uart.txt true; start'
# in another terminal:
tail -f /tmp/fennec-uart.txt
```

### Automated tests

```bash
cargo build-emu
source ~/.venvs/renode/bin/activate
renode-test renode/tests/blinky.robot
```

The tests check that:
- the green LED blinks at 1 Hz,
- the orange LED stays off,
- the startup and `blink N` log lines appear with the right timing.

A pass ends with `Tests finished successfully :)`. Results are written to `log.html` / `report.html` (git-ignored).

---

## Debugging with GDB

### Against the real board (OpenOCD)

Terminal 1:
```bash
openocd -f openocd.cfg
```

Terminal 2:
```bash
gdb-multiarch target/thumbv7em-none-eabihf/debug/blinky
(gdb) target extended-remote :3333
(gdb) load
(gdb) monitor reset halt
(gdb) break blinky.rs:35        # led.toggle(); check the line number if the file changed
(gdb) continue
```

### Against Renode

While `renode renode/blinky.resc` is running, Renode serves GDB on port 3333. Don't run `load`, because Renode already loaded the firmware:

```bash
gdb-multiarch target/emulation/thumbv7em-none-eabihf/debug/blinky -ex "target remote 127.0.0.1:3333"
```

Debug builds are lightly optimised (`opt-level = 1`), so GDB may show some variables as `<optimized out>` or move a breakpoint by a line. That's normal.

---

## Logging

Use `fennec_fw::info!`, `warn!` and `error!` (in `src/log.rs`), **not** `defmt::info!` directly:

| Build | Command | Logs appear |
|---|---|---|
| Hardware | `cargo build` / `cargo run` | in the `cargo run` terminal (defmt over the debug probe) |
| Emulation | `cargo build-emu` / `cargo run-emu` | as text on USART2 (TX = PA2), shown by Renode |

Format strings must work in both modes. Use plain `{}` with integers, `bool`, `char` and `&str`. defmt-only syntax such as `{=u8}` breaks the emulation build.

The log level is set by `DEFMT_LOG` in `.cargo/config.toml` (default `info`).

---

## Project layout

```
firmware-rs/
├── Cargo.toml            package "fennec-fw": dependencies, features, build profiles
├── Cargo.lock            exact dependency versions (commit this)
├── .cargo/config.toml    target, flash runner, linker flags, build-emu / run-emu
├── rust-toolchain.toml   pinned Rust toolchain
├── openocd.cfg           OpenOCD config for the GDB fallback
├── src/
│   ├── lib.rs            code shared by all firmware binaries
│   ├── log.rs            logging macros (defmt on hardware, UART text in emulation)
│   └── bin/
│       └── blinky.rs     each file in bin/ is its own firmware image
└── renode/
    ├── discovery.repl    emulated board (Renode's STM32F4-Discovery + extra LEDs)
    ├── blinky.resc       interactive Renode script
    ├── run.sh            cargo runner used by `cargo run-emu`
    └── tests/
        └── blinky.robot  automated tests
```

### Cargo features

| Feature | Default | Meaning |
|---|---|---|
| `hardware` | yes | Normal build: `panic-probe` prints panics over defmt |
| `emulation` | no | Renode build: text logs and panics on USART2 |

`cargo build-emu` / `cargo run-emu` switch from `hardware` to `emulation` for you. Enabling both at once fails to compile on purpose.

### Adding another firmware binary

Create `src/bin/<name>.rs`, then use `cargo run --bin <name>` / `cargo run-emu --bin <name>`. Copy the top of `blinky.rs` for the required `#![no_std]`, `#![no_main]`, `use` lines and the emulation log setup.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| `probe-rs list` shows no probes | Check the USB cable is on the ST-LINK (mini-USB) port, then see [Board access](#2-board-access-hardware-only). |
| `error: linker 'flip-link' not found` | `cargo install flip-link` |
| `can't find crate for 'core'` | `rustup target add thumbv7em-none-eabihf` |
| `duplicate lang item ... panic_impl` | You enabled `emulation` without disabling `hardware`. Use `cargo build-emu`. |
| Renode test fails to find the ELF | Run `cargo build-emu` first. Tests use `target/emulation/...`. |
| No log lines in Renode | Make sure it's an emulation build (`cargo build-emu`) and the code uses `fennec_fw::info!`, not `defmt::info!`. |
| Emulated timing is wrong | `renode/discovery.repl` sets TIM2 to 16 MHz to match the firmware's clock. If you change the clock config, update it. |

## Known limitations

- Only tested so far in Renode. **Not yet run on a physical Discovery board.**
- USART2 as the emulation log port is a dev-board assumption. Fennec's real hardware will need a UART that isn't used for GNSS or the radio.
- Renode's STM32F4 model has no SDIO/SDMMC or ADC, so SD logging and battery sensing will need hardware testing (or extra Renode models).
