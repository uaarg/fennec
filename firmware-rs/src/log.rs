//! `info!`/`warn!`/`error!`: defmt on hardware, plain text on a UART with
//! `--features emulation` (for Renode). Format strings must work in both,
//! so stick to plain `{}`.

#[cfg(not(feature = "emulation"))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => { ::defmt::info!($($arg)*) };
}

#[cfg(not(feature = "emulation"))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => { ::defmt::warn!($($arg)*) };
}

#[cfg(not(feature = "emulation"))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => { ::defmt::error!($($arg)*) };
}

#[cfg(feature = "emulation")]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => { $crate::log::write_line("INFO", format_args!($($arg)*)) };
}

#[cfg(feature = "emulation")]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => { $crate::log::write_line("WARN", format_args!($($arg)*)) };
}

#[cfg(feature = "emulation")]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => { $crate::log::write_line("ERROR", format_args!($($arg)*)) };
}

#[cfg(feature = "emulation")]
pub use emulation::{init, write_line};

#[cfg(feature = "emulation")]
mod emulation {
    use core::cell::RefCell;
    use core::fmt::{self, Write};

    use critical_section::Mutex;
    use embassy_stm32::mode::Blocking;
    use embassy_stm32::usart::UartTx;

    static LOG_UART: Mutex<RefCell<Option<UartTx<'static, Blocking>>>> =
        Mutex::new(RefCell::new(None));

    pub fn init(tx: UartTx<'static, Blocking>) {
        critical_section::with(|cs| {
            LOG_UART.borrow(cs).replace(Some(tx));
        });
    }

    struct UartWriter<'a>(&'a mut UartTx<'static, Blocking>);

    impl Write for UartWriter<'_> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.0.blocking_write(s.as_bytes()).map_err(|_| fmt::Error)
        }
    }

    /// Writes `<seconds>.<ms> <LEVEL> <message>\r\n`. Blocking.
    pub fn write_line(level: &str, args: fmt::Arguments) {
        let uptime_ms = embassy_time::Instant::now().as_millis();

        critical_section::with(|cs| {
            // try_borrow_mut: a panic mid-line must not panic again here.
            let Ok(mut slot) = LOG_UART.borrow(cs).try_borrow_mut() else {
                return;
            };
            let Some(tx) = slot.as_mut() else {
                return;
            };

            let mut out = UartWriter(tx);
            let _ = write!(out, "{}.{:03} {} ", uptime_ms / 1000, uptime_ms % 1000, level);
            let _ = out.write_fmt(args);
            let _ = out.write_str("\r\n");
        });
    }

    #[panic_handler]
    fn panic(info: &core::panic::PanicInfo) -> ! {
        write_line("PANIC", format_args!("{}", info));
        loop {
            cortex_m::asm::nop();
        }
    }
}
