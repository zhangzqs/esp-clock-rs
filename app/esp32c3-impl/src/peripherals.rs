use esp_idf_hal::ledc::{LedcChannel, LedcTimer};
use esp_idf_hal::rmt::RmtChannel;
use esp_idf_svc::hal::gpio::{AnyInputPin, AnyOutputPin, PinDriver};
use esp_idf_svc::hal::modem::Modem;

#[cfg(feature = "pyclock")]
mod pyclock;
#[cfg(feature = "pyclock")]
use pyclock as _;

pub struct SystemPeripherals<
    SPI,
    BuzzerRmtChannel,
    BacklightLedcChannel,
    BacklightLedcTimer,
    BoardLedLedcChannel,
    BoardLedLedcTimer,
> {
    pub buzzer: Option<BuzzerPeripherals<BuzzerRmtChannel>>,
    pub board_led: Option<LedcPeripherals<BoardLedLedcChannel, BoardLedLedcTimer>>,
    pub button: Option<AnyInputPin>,
    pub display: DisplaySpiPeripherals<SPI, BacklightLedcChannel, BacklightLedcTimer>,
    pub modem: Modem,
}

pub struct BuzzerPeripherals<C> {
    pub pin: AnyOutputPin,
    pub rmt_channel: C,
}

pub struct LedcPeripherals<C, T> {
    pub pin: AnyOutputPin,
    pub ledc_channel: C,
    pub ledc_timer: T,
}

pub struct DisplayControlPeripherals<BacklightLedcChannel, BacklightLedcTimer> {
    pub backlight: Option<LedcPeripherals<BacklightLedcChannel, BacklightLedcTimer>>,
    pub dc: AnyOutputPin,
    pub rst: AnyOutputPin,
}

pub struct DisplaySpiPeripherals<SPI, BacklightLedcChannel, BacklightLedcTimer> {
    pub control: DisplayControlPeripherals<BacklightLedcChannel, BacklightLedcTimer>,
    pub spi: SPI,
    pub sclk: AnyOutputPin,
    pub sdo: AnyOutputPin,
    pub cs: AnyOutputPin,
}
