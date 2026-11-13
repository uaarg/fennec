*** Comments ***
Run from firmware-rs/: cargo build-emu && renode-test renode/tests/blinky.robot

*** Variables ***
${PLATFORM}         @${CURDIR}/../discovery.repl
${ELF}              @${CURDIR}/../../target/emulation/thumbv7em-none-eabihf/debug/blinky
${LOG_UART}         sysbus.usart2

*** Keywords ***
Create Discovery Machine
    Execute Command                 mach create "discovery"
    Execute Command                 machine LoadPlatformDescription ${PLATFORM}
    Execute Command                 sysbus LoadELF ${ELF}

*** Test Cases ***
Green LED Should Blink At 1 Hz
    Create Discovery Machine
    Create LED Tester               sysbus.gpioPortD.UserLED
    Assert LED Is Blinking          testDuration=4  onDuration=0.5  offDuration=0.5  tolerance=0.2  pauseEmulation=true

Other LEDs Should Stay Off
    Create Discovery Machine
    Create LED Tester               sysbus.gpioPortD.led_orange
    Start Emulation
    Assert LED State                false  timeout=2

Should Print Startup And Blink Logs
    Create Discovery Machine
    Create Terminal Tester          ${LOG_UART}  defaultPauseEmulation=true
    Wait For Line On Uart           ^\\d+\\.\\d{3} INFO Fennec blinky starting$  treatAsRegex=true
    Wait For Line On Uart           INFO blink 1
    Wait For Line On Uart           INFO blink 2
    ${line}=                        Wait For Line On Uart  ^(\\d+)\\.(\\d{3}) INFO blink 3$  treatAsRegex=true
    Should Be Equal As Integers     ${line.Groups[0]}  1
