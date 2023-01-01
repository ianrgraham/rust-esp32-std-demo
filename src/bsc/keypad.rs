use esp_idf_hal::gpio::{PinDriver, AnyIOPin, Input, Output};
use esp_idf_hal::gpio::Pull;

pub struct Keypad<'d> {
    rows: [PinDriver<'d, AnyIOPin, Output>; 4],
    cols: [PinDriver<'d, AnyIOPin, Input>; 4]
}

impl<'d> Keypad<'d> {
    pub fn new(rows: [AnyIOPin; 4], cols: [AnyIOPin; 4]) -> anyhow::Result<Self> {

        let mut rows = rows.try_map(|pin| PinDriver::output(pin))?;

        for pin in &mut rows {
            pin.set_low()?;
        }

        let mut cols = cols.try_map(|pin| PinDriver::input(pin))?;
        
        for pin in &mut cols {
            pin.set_pull(Pull::Down)?;
        }


        Ok(Self { rows, cols })
    }

    pub fn scan(&mut self, buffer: &mut Vec<(u8, u8)>) {
        buffer.clear();
        for row in 0..4 {
            self.rows[row].set_high().unwrap();
            
            for col in 0..4 {
                if self.cols[col].is_high() {
                    buffer.push((row as u8, col as u8));

                }
            }
            self.rows[row].set_low().unwrap();
        }
    }
}