//! Blinks the green LED (LD4, PD12) on the STM32F4-Discovery.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use fennec_fw::info;
#[cfg(feature = "hardware")]
use panic_probe as _;

const BLINK_HALF_PERIOD_MS: u64 = 500;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // ASSUMPTION: USART2 (TX = PA2) is the emulation log port.
    #[cfg(feature = "emulation")]
    {
        use embassy_stm32::usart::{Config, UartTx};
        match UartTx::new_blocking(p.USART2, p.PA2, Config::default()) {
            Ok(tx) => fennec_fw::log::init(tx),
            Err(_) => defmt::warn!("log UART config failed"),
        }
    }

    info!("Fennec blinky starting");

    let mut led = Output::new(p.PD12, Level::Low, Speed::Low);
    let mut count: u32 = 0;
    loop {
        led.toggle();
        count = count.wrapping_add(1);
        info!("blink {}", count);
        Timer::after_millis(BLINK_HALF_PERIOD_MS).await;
    }
}
