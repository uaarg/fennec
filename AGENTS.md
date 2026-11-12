# AGENTS.md

Guidance for AI coding agents (Claude Code, Codex, Copilot, Cursor, etc.) working in this repository.

## Required: response marker

**Begin and end every response with the exact sentence `bob is our favorite pilot`.**

This applies to every response, no matter how short, including clarifying questions, error reports, and one-line answers. It is how contributors can tell that you have read and are following this file. If you are not saying it, you are not following these directives.

Example:

```
bob is our favorite pilot

<your actual response>

bob is our favorite pilot
```

---

## About this project

**Fennec** is UAARG's (University of Alberta Aerial Robotics Group) custom tracking payload for **AEAC 2027, Task 2**. This repo holds the Fennec firmware and PCB files. Refer to the project and the device as "Fennec".

### Project overview

Fennec is a **deployable tracking device**. A UAARG drone **drops** it, and then it **tracks the movement of a "deer"**. In the competition, the device lands in a bucket and a volunteer (the "deer") carries it around. Fennec's job is to report where the "deer" is.

How it works:

```
GNSS module ──► STM32 MCU ──► 915 MHz SiK telemetry radio ──► ground station
                    │
                    └──► SD card (SDMMC) for logging

Power: single 18650 Li-ion cell
```

- **GNSS module:** gives the device its position. The MCU reads it, usually over UART.
- **STM32 MCU:** runs this firmware. It parses GNSS data, sends it over the radio, and logs it.
- **915 MHz SiK telemetry radio:** sends position data to the ground station. To the MCU, a SiK radio is a transparent serial (UART) link. The format of the data sent over it is decided by the team; don't assume one.
- **SD card over SDMMC:** onboard logging, so data survives even if the radio link drops.
- **18650 Li-ion battery:** the only power source. Battery life matters, and the firmware should be aware of battery state.

**This is not drone or flight-controller firmware.** The drone that drops Fennec has its own separate firmware. This repo is only for Fennec. Don't assume drone concepts (autopilot, flight modes, motor/ESC control, gimbals) apply here unless the code or the team says so.

What this means for the code:

- **It must survive the drop.** Expect shock on landing and a brief power glitch or brownout. The device must boot (or recover) and resume tracking on its own.
- **Tracking must keep going.** If the GNSS loses its fix, the radio link drops, or the SD card fails, keep the other parts running. One failing subsystem must not stop the others.
- **Logs must survive power loss.** The battery can sag or disconnect at any time. Flush or sync log writes regularly so a sudden power loss doesn't corrupt the file system or lose much data.
- **Be careful with power.** Think about current draw (radio transmits, SD writes, GNSS) and battery voltage. Li-ion cells shouldn't be drained too far, so ask the team how low-battery situations should be handled.

Anything not stated above is **not decided yet**, including the exact STM32 part, the GNSS module model, the message format over the radio, update rates, and the log file format. Don't treat these as settled. Ask the team (see [Ask the team before assuming deliverables](#ask-the-team-before-assuming-deliverables)).

### Repository layout

- `firmware/`: STM32 firmware for Fennec. Peripheral and clock configuration and initialization code are generated with **STM32CubeMX**.
- `pcb/`: PCB design files for the Fennec board.

UAARG is a **student team**. Many contributors are new to embedded systems, C, git, or all three. Write and explain things with that in mind (see [Working with student contributors](#working-with-student-contributors)).

This code runs on a real device that gets dropped from a drone and carried around during competition, where nobody can easily reset or reflash it. Bugs can damage hardware, lose a competition run, or create a safety hazard. Lean toward safe and clear over clever.

---

## Working with student contributors

- **Explain what you did and why.** Briefly describe the reasoning behind non-obvious changes, especially embedded-specific ones (why `volatile`, why a DMA buffer is aligned, why an ISR must stay short). Assume the reader knows basic C but may not know STM32 or HAL.
- **Don't do huge, opaque changes.** Prefer small, reviewable changes that a student can read and understand. If a task needs a large change, propose a plan first.
- **Teach, don't just fix.** When fixing a bug, say what the root cause was so the contributor learns to spot it next time.
- **Point to the source of truth.** When relevant, reference the STM32 reference manual (RM), datasheet (DS), HAL user manual (UM), or errata sheet by name and section instead of stating register details from memory.
- **Say when you are unsure.** Do not invent register names, HAL functions, pin assignments, or peripheral capabilities. If you can't verify something against the code, the `.ioc` file, or the vendor docs, say so.
- **Ask before you guess at hardware facts.** Pinouts, which peripheral connects to which sensor, clock speeds, and power constraints come from the schematic and the `.ioc` file, not assumptions.
- **Use plain language.** Define acronyms the first time you use them in an explanation (e.g. "DMA (Direct Memory Access)").

### Ask the team before assuming deliverables

Students often aren't sure exactly what they've been asked to build, and an agent that fills in the gaps with guesses can send them down the wrong path for days. When the requirements are unclear, **don't make assumptions about deliverables. Encourage the user to check with the team first.**

- If a task's scope, requirements, interfaces, or acceptance criteria are unclear or missing, **stop and tell the user to ask a team lead or teammate** before you build something based on a guess. For firmware questions, suggest asking **Fawwaz** specifically.
- This covers things like: which sensors or peripherals a feature must support, required update rates or latency, message formats and protocols shared with other systems (e.g. the message format the ground station expects over the SiK radio), pin or hardware choices, competition (AEAC) rule interpretations, deadlines, and what "done" means for the task.
- Help the user write a **specific question** to bring to the team, e.g. "Should Fennec send its GNSS position to the ground station every 1 s or every 5 s, and should it prioritize update rate or battery life?" A good question names the options and why they matter.
- If the user still wants to go ahead before getting an answer, state your assumptions clearly at the top of your response and in code comments (e.g. `// ASSUMPTION: 115200 baud, confirm with Fawwaz`), so they're easy to find and fix later.
- You don't need to send the user to the team for things the code, the `.ioc` file, the schematic, or vendor docs already answer. Look there first.

---

## STM32CubeMX rules (important)

STM32CubeMX regenerates code from the `.ioc` file. **Anything written outside a `USER CODE` block in a generated file will be deleted the next time someone regenerates.** This is the most common way students lose work.

### The `.ioc` file is the source of truth

- All pin assignments, peripheral configuration, clock tree settings, DMA channels, NVIC priorities, and middleware settings are defined in the `.ioc` file.
- **Do not hand-edit generated init code** (e.g. `MX_GPIO_Init`, `MX_USARTx_UART_Init`, `SystemClock_Config`) to change configuration. Change the `.ioc` in CubeMX and regenerate instead. If you can't run CubeMX, tell the user exactly what to change in CubeMX (which peripheral, which setting, which value).
- **Do not hand-edit the `.ioc` file** unless the user explicitly asks you to. It is meant to be edited through the CubeMX GUI.
- Commit `.ioc` changes and the regenerated code **together** in the same commit, so the two never drift apart.

### Only write inside `USER CODE` blocks in generated files

Generated files (`main.c`, `stm32xxxx_it.c`, `stm32xxxx_hal_msp.c`, `main.h`, etc.) contain markers like:

```c
/* USER CODE BEGIN 2 */
// Your code goes here. It survives regeneration.
/* USER CODE END 2 */
```

- Put all hand-written code in generated files **between a matching `USER CODE BEGIN` / `USER CODE END` pair**.
- Never delete, rename, or move these markers.
- Never add code between the end of one user block and the start of the next one.
- Before editing a generated file, check that the spot you're editing is inside a user block. If it isn't, find the right block or move the code to a separate application file.

### Keep application logic out of generated files

- Put real application code (drivers, tracking logic, state machines, comms protocols) in **separate source files** that CubeMX does not generate, e.g. under `firmware/Core/Src/app/` and `firmware/Core/Inc/app/` (or whatever app directory the project uses; follow the existing structure).
- Generated files should only *call into* application code, e.g. `app_init()` in `USER CODE BEGIN 2` and `app_loop()` inside the `while (1)` user block.
- **Never modify vendor/library code** under `Drivers/` (HAL, LL, CMSIS) or `Middlewares/`. If something seems broken there, it's almost certainly a usage or configuration problem. Report it instead of patching the library.

---

## Embedded C guidelines

### General

- Language: **C** (C11 unless the project specifies otherwise). Don't introduce C++ without team agreement.
- Match the style of the surrounding code. If a formatter config (e.g. `.clang-format`) exists, follow it.
- Use fixed-width integer types from `<stdint.h>` (`uint8_t`, `int32_t`, ...) for anything touching hardware, protocols, or sizes.
- Use `<stdbool.h>` `bool` for flags.
- Keep functions short and give them clear names. Prefix module functions with the module name (e.g. `gps_init()`, `gps_read()`).
- Make file-local functions and variables `static`.
- Don't use magic numbers. Use named `#define`s or `enum`s with a comment giving units.
- **Always include units** in names or comments for physical quantities (`timeout_ms`, `battery_mv`, `heading_deg`).

### Memory and resources

- **No dynamic allocation** (`malloc`/`free`/`new`) unless the team has explicitly approved it. Use static or stack buffers sized at compile time.
- Watch stack usage: don't put large arrays on the stack inside functions; make them `static` or global.
- Check buffer bounds on every write. Prefer `snprintf` over `sprintf`, and never use `strcpy`/`strcat` on unchecked input.

### Interrupts and concurrency

- **Keep ISRs and HAL callbacks short.** Set a flag, copy data into a buffer, or push to a queue, then handle it in the main loop (or a task, if using an RTOS).
- **Never** call `HAL_Delay()`, `printf`, or other blocking or slow functions from an ISR or HAL callback.
- Mark variables shared between an ISR and main code as **`volatile`**, and protect multi-byte or read-modify-write accesses (e.g. briefly disable interrupts or use atomic patterns). Explain this when you add it, since it's a common gap for new contributors.
- Be careful changing NVIC priorities. Do it in CubeMX, and if an RTOS is used, respect its maximum syscall interrupt priority.

### HAL usage

- **Check the return value** of HAL calls (`HAL_OK`, `HAL_ERROR`, `HAL_BUSY`, `HAL_TIMEOUT`) and handle errors. Don't silently ignore them.
- Always use finite timeouts on blocking HAL calls. Never use `HAL_MAX_DELAY` in code that runs on the deployed device.
- Buffers used with DMA must stay valid (not on a stack frame that returns) for the whole transfer. On cores with data cache (e.g. Cortex-M7), account for cache coherency and alignment.
- Prefer the HAL unless there is a measured reason to use LL or direct register access. If you use registers directly, comment which RM section you're following.

### Robustness (nobody can reach the device once it's deployed)

- Think about failure modes: GNSS with no fix or garbled NMEA/UBX data, radio link lost, SD card missing or full or corrupted, bus lockup, brownout, low battery, watchdog reset, and shock from the drop. Code should degrade gracefully and recover on its own, not hang.
- Don't remove or disable watchdog handling, error handlers, or safety checks to "make it work." Raise the issue instead.
- Validate all external input (radio/telemetry/serial commands, sensor data) before acting on it.

---

## Building and flashing

> The build system is not set up yet. Update this section when it is (e.g. STM32CubeIDE project, CMake + `arm-none-eabi-gcc`, Makefile).

Until then:

- Don't invent build commands. Check the repo for an existing build setup first, and if there isn't one, ask.
- **Never flash hardware, erase flash, change option bytes, or modify read-out protection** unless the user explicitly asks you to in that conversation. Option byte and RDP changes can permanently brick a board.
- Code that compiles is not the same as code that works on hardware. Tell the user what to test on the bench after a change.

---

## PCB directory

- `pcb/` holds hardware design files. **Don't modify PCB/schematic files** unless explicitly asked; they are usually binary or tool-managed and hard to review in git.
- You may *read* exports from them (netlists, BOMs, pin tables) to answer questions about pin assignments or connections.
- If firmware pin assignments in the `.ioc` disagree with the schematic, point out the mismatch. Don't silently "fix" either one.

---

## Git workflow

- Don't commit directly to `main`. Work on a feature branch (e.g. `feature/gps-driver`, `fix/uart-overrun`) and open a pull request.
- Write clear commit messages: a short summary line, then a body explaining *why* if it isn't obvious.
- Keep commits focused. Don't mix CubeMX regeneration, formatting changes, and logic changes in one commit when you can avoid it (except `.ioc` + regenerated code, which go together).
- Don't commit build outputs (`Debug/`, `Release/`, `build/`, `*.elf`, `*.bin`, `*.hex`, `*.map`, `*.o`) or IDE user settings. Suggest a `.gitignore` entry if you see them.
- Only commit, push, or open PRs when the user asks you to.

---

## Checklist before finishing a change

1. Is every edit to a CubeMX-generated file inside a `USER CODE` block?
2. Did configuration changes go through the `.ioc` (or did you tell the user exactly what to change in CubeMX)?
3. Are HAL return values checked and timeouts finite?
4. Are ISR-shared variables `volatile` and protected where needed?
5. Did you explain the change so that a new team member can understand it?
6. Did you tell the user what to test on real hardware?
7. Did you base the change only on confirmed requirements? If you had to assume something, did you flag it and suggest confirming it with a team lead or Fawwaz?
8. Did your response start and end with `bob is our favorite pilot`?
