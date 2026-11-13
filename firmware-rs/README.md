# Fennec firmware (Rust, experimental)

This is an experimental Rust version of the Fennec firmware. It is developed alongside the team's C/STM32CubeMX firmware in `../firmware/`, which it does not touch.

It currently contains a **blinky** for the **STM32F4-Discovery** dev board (STM32F407VG). Its purpose is to prove that the build, flash, debug and emulation tools all work. The STM32F4-Discovery is **not** the final Fennec MCU; the team hasn't chosen that part yet.

Built with [Embassy](https://embassy.dev) (async Rust HAL and executor), [defmt](https://defmt.ferrous-systems.com) logging, and [probe-rs](https://probe.rs) for flashing.

## Layout

```
firmware-rs/
├── Cargo.toml            single package (fennec-fw): deps, features, profiles
├── .cargo/config.toml    target, flash runner, linker flags, build-emu/run-emu aliases
├── rust-toolchain.toml   pinned toolchain (rustup installs it automatically)
├── openocd.cfg           OpenOCD fallback config
├── src/
│   ├── lib.rs            shared code for all binaries
│   ├── log.rs            info!/warn!/error!: defmt on hardware, UART text in emulation
│   └── bin/blinky.rs
└── renode/               emulation
    ├── discovery.repl    emulated board description
    ├── blinky.resc       interactive Renode script
    ├── run.sh            cargo runner that launches Renode
    └── tests/blinky.robot  automated test
```

## One-time setup

```bash
rustup target add thumbv7em-none-eabihf      # rust-toolchain.toml also does this
rustup component add llvm-tools
cargo install probe-rs-tools flip-link cargo-binutils
```

- **Probe permissions (Linux):** the ST-Link must be accessible without root. The udev rules installed by Ubuntu's `openocd` package (`49-stlinkv2*.rules`) usually cover this. If `probe-rs list` shows nothing while the board is plugged in, follow <https://probe.rs/docs/getting-started/probe-setup/>.
- **Renode:** install `renode_<version>_amd64.deb` with `sudo apt install ./renode_<version>_amd64.deb`. Then set up the test tools in a venv:
  ```bash
  python3 -m venv ~/.venvs/renode
  ~/.venvs/renode/bin/pip install -r /opt/renode/tests/requirements.txt
  ```

All commands below are run from `firmware-rs/`.

## Build

```bash
cargo build --bin blinky              # debug build
cargo build --release --bin blinky    # optimised for size
cargo size --bin blinky               # flash (text + data) and RAM (data + bss) usage
```

## Flash and run on the real board

1. Plug the Discovery board into your PC through its **mini-USB** port (the ST-LINK side, labelled CN1).
2. Check that the probe is found:
   ```bash
   probe-rs list
   ```
3. Build, flash and watch the logs:
   ```bash
   cargo run --bin blinky
   ```
   The green LED (LD4) blinks at 1 Hz and the terminal prints `blink 1`, `blink 2`, ... Press Ctrl+C to stop watching. The firmware keeps running.

The firmware is stored in flash, so it starts again on its own after a reset or a power cycle.

## Debug with OpenOCD and GDB (fallback)

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
(gdb) break main                 # program entry point
(gdb) break blinky.rs:58         # the led.toggle() line; update if the file changes
(gdb) continue
```
Notes:
- defmt logs are not shown in this mode. Use `cargo run` (probe-rs) when you need logs.
- Debug builds use `opt-level = 1`, so GDB may report some local variables as `<optimized out>` and move a breakpoint by a line. That's normal.

## Logging: `fennec_fw::info!`

Log with `fennec_fw::info!`, `warn!` and `error!` (from `src/log.rs`), not `defmt::info!` directly:

| Build | Command | Where logs go |
|---|---|---|
| Hardware | `cargo build` / `cargo run` | defmt over RTT through the probe, shown by `cargo run` |
| Emulation | `cargo build-emu` / `cargo run-emu` | plain text on **USART2** (TX = PA2), e.g. `1.000 INFO blink 3` |

Emulation builds also print panics to USART2 (`PANIC panicked at ...`). They go to `target/emulation/` so they can't be confused with hardware builds. **Never flash an emulation build to the real device.**

Format strings must work in both modes: use plain `{}` with integers, `bool`, `char` and `&str`. defmt-only syntax like `{=u8}` breaks the emulation build.

## Emulate with Renode

Renode uses the **emulation** build, so build it first:
```bash
cargo build-emu --bin blinky
```

**Automated tests** (LED blinks at 1 Hz, other LEDs stay off, expected log lines appear):
```bash
source ~/.venvs/renode/bin/activate
renode-test renode/tests/blinky.robot
```

**Interactive:**
```bash
renode renode/blinky.resc
# A "usart2" window shows the log lines. In the Renode monitor:
(discovery) sysbus.gpioPortD.UserLED State
```

**Through cargo** (builds the emulation image, then launches Renode):
```bash
cargo run-emu --bin blinky
```

**Headless (no windows), e.g. over SSH:** send the UART to a file and follow it:
```bash
renode --console --disable-xwt -e 'include @renode/blinky.resc; sysbus.usart2 CreateFileBackend @/tmp/fennec-uart.txt true'
# in another terminal:
tail -f /tmp/fennec-uart.txt
```

While Renode runs, a GDB server listens on port 3333, so you can debug the emulator too. Skip `openocd` and `load` (Renode has already loaded the ELF), and connect with:
```bash
gdb-multiarch target/emulation/thumbv7em-none-eabihf/debug/blinky -ex "target remote 127.0.0.1:3333"
```

### Renode limitations to know about

- **defmt logs are not visible in Renode** (defmt over RTT needs a debug probe). That's why emulation builds switch `fennec_fw::info!` to UART text. Calling `defmt::info!` directly still won't show up in Renode.
- **Timers don't follow the clock config.** `renode/discovery.repl` sets TIM2 to 16 MHz to match the firmware's clocks. If you change the clock tree in firmware, update that value too, or emulated time will be wrong.
- Renode's STM32F4 model has **no SDIO/SDMMC and no ADC**. SD logging and battery sensing must be tested on hardware.
