#![no_std]
#![no_main]

// https://dev.to/theembeddedrustacean/embedded-rust-embassy-gpio-button-controlled-blinking-3ee6

use core::sync::atomic::{AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_stm32::{
    Peripherals, bind_interrupts,
    exti::ExtiInput,
    gpio::{AnyPin, Level, Output, Pin, Pull, Speed},
    mode::Async,
    peripherals::USART1,
    usart::{Config, InterruptHandler, Uart},
};
use embassy_time::{Duration, Timer};
use panic_halt as _;

static BLINK_MS: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::task]
async fn led_task(led: AnyPin) {
    let mut led: Output<'_> = Output::new(led, Level::Low, Speed::Low);

    loop {
        let delay: u32 = BLINK_MS.load(Ordering::Relaxed);
        Timer::after(Duration::from_millis(delay.into())).await;
        led.toggle();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals: Peripherals = embassy_stm32::init(Default::default());

    let mut button: ExtiInput<'_> = ExtiInput::new(peripherals.PE11, peripherals.EXTI11, Pull::Up);

    let mut delay_value_ms: u32 = 2_000;
    BLINK_MS.store(delay_value_ms, Ordering::Relaxed);
    spawner.spawn(led_task(peripherals.PE12.degrade())).unwrap();

    bind_interrupts!(struct Irqs {
        USART1 => InterruptHandler<USART1>;
    });
    let mut usart: Uart<'_, Async> = Uart::new(
        peripherals.USART1,
        peripherals.PA10,
        peripherals.PA9,
        Irqs,
        peripherals.DMA1_CH4,
        peripherals.DMA1_CH5,
        Config::default(), // 115200 baud
    )
    .unwrap();

    loop {
        button.wait_for_low().await;
        delay_value_ms = delay_value_ms / 2;
        if delay_value_ms < 50 {
            delay_value_ms = 2_000;
        }
        BLINK_MS.store(delay_value_ms, Ordering::Relaxed);

        usart.write(b"changing speed...\r\n").await.unwrap();

        // debounce....
        Timer::after_millis(200).await;
        button.wait_for_high().await;
    }
}
