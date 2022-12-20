use esp_idf_hal::gpio::PinDriver;
use esp_idf_sys::EspError;
use esp_idf_hal::gpio::Pull;
use embedded_hal::digital::v2::{OutputPin, InputPin};

pub struct Keypad {
    rows: [Box<dyn OutputPin<Error = EspError>>; 4],
    cols: [Box<dyn InputPin<Error = EspError>>; 4]
}

impl Keypad {
    pub fn new(pins: esp_idf_hal::gpio::Pins ) -> anyhow::Result<Self> {

        let mut rows: [Box<dyn OutputPin<Error = EspError>>; 4] = [
            Box::new(PinDriver::output(pins.gpio0)?),
            Box::new(PinDriver::output(pins.gpio1)?),
            Box::new(PinDriver::output(pins.gpio2)?),
            Box::new(PinDriver::output(pins.gpio3)?)
        ];

        for input in &mut rows {
            input.set_low()?;
        }

        let mut in1 = PinDriver::input(pins.gpio4)?;
        let mut in2 = PinDriver::input(pins.gpio5)?;
        let mut in3 = PinDriver::input(pins.gpio6)?;
        let mut in4 = PinDriver::input(pins.gpio7)?;
        
        in1.set_pull(Pull::Down)?;
        in2.set_pull(Pull::Down)?;
        in3.set_pull(Pull::Down)?;
        in4.set_pull(Pull::Down)?;

        let cols: [Box<dyn InputPin<Error = EspError>>; 4] = [
            Box::new(in1),
            Box::new(in2),
            Box::new(in3),
            Box::new(in4)
        ];



        Ok(Self { rows, cols })
    }

    pub fn scan(&mut self, buffer: &mut Vec<(u8, u8)>) {
        buffer.clear();
        for row in 0..4 {
            self.rows[row].set_high().unwrap();
            for col in 0..4 {
                if self.cols[col].is_high().unwrap() {
                    buffer.push((row as u8, col as u8));
                }
            }
            self.rows[row].set_low().unwrap();
        }
    }
}