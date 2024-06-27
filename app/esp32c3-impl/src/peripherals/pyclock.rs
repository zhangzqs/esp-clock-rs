use esp_idf_hal::{gpio, ledc, rmt, spi};
use esp_idf_svc::hal::peripherals::Peripherals;

use super::{
    BuzzerPeripherals, DisplayControlPeripherals, DisplaySpiPeripherals, LedcPeripherals,
    SystemPeripherals,
};

impl
    SystemPeripherals<
        spi::SPI2,
        rmt::CHANNEL0,
        ledc::CHANNEL0,
        ledc::TIMER0,
        ledc::CHANNEL1,
        ledc::TIMER1,
    >
{
    pub fn take() -> Self {
        let peripherals = Peripherals::take().unwrap();

        SystemPeripherals {
            button: Some(peripherals.pins.gpio9.into()),
            board_led: Some(LedcPeripherals {
                pin: peripherals.pins.gpio2.into(),
                ledc_channel: peripherals.ledc.channel1,
                ledc_timer: peripherals.ledc.timer1,
            }),
            display: DisplaySpiPeripherals {
                control: DisplayControlPeripherals {
                    backlight: Some(LedcPeripherals {
                        pin: peripherals.pins.gpio10.into(),
                        ledc_channel: peripherals.ledc.channel0,
                        ledc_timer: peripherals.ledc.timer0,
                    }),
                    dc: peripherals.pins.gpio4.into(),
                    rst: peripherals.pins.gpio8.into(),
                },
                spi: peripherals.spi2,
                sclk: peripherals.pins.gpio6.into(),
                sdo: peripherals.pins.gpio7.into(),
                cs: peripherals.pins.gpio5.into(),
            },
            modem: peripherals.modem,
            buzzer: Some(BuzzerPeripherals {
                pin: peripherals.pins.gpio0.into(),
                rmt_channel: peripherals.rmt.channel0,
            }),
        }
    }
}
