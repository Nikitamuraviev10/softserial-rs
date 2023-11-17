use crate::hal;
use core::arch::asm;
use hal::blocking::spi::{Transfer, Write};
use hal::digital::v2::{InputPin, OutputPin};

pub use hal::spi::{Mode, Phase, Polarity};

pub struct Spi<SCK, MISO, MOSI> {
    sck: SCK,
    miso: Option<MISO>,
    mosi: Option<MOSI>,
    mode: Mode,
    delay: usize,
}

impl<SCK, MISO, MOSI> Spi<SCK, MISO, MOSI>
where
    SCK: OutputPin,
    MOSI: OutputPin,
    MISO: InputPin,
{
    pub fn new(sck: SCK, miso: Option<MISO>, mosi: Option<MOSI>, mode: Mode, delay: usize) -> Self {
        let mut sck = sck;
        match mode.polarity {
            Polarity::IdleLow => sck.set_low().ok(),
            Polarity::IdleHigh => sck.set_high().ok(),
        };

        Self {
            sck,
            miso,
            mosi,
            mode,
            delay,
        }
    }
}

impl<SCK, MISO, MOSI> Write<u8> for Spi<SCK, MISO, MOSI>
where
    SCK: OutputPin,
    MOSI: OutputPin,
    MISO: InputPin,
{
    type Error = ();
    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let sck = &mut self.sck;
        for word in words {
            for bit in (0..8).rev() {
                let mask = 1 << bit;

                if let Some(mosi) = self.mosi.as_mut() {
                    match word & mask {
                        0 => mosi.set_low().ok(),
                        _ => mosi.set_high().ok(),
                    };
                }

                match self.mode.polarity {
                    Polarity::IdleLow => sck.set_high().ok(),
                    Polarity::IdleHigh => sck.set_low().ok(),
                };

                for _ in 0..self.delay {
                    unsafe { asm!("nop") };
                }

                match self.mode.polarity {
                    Polarity::IdleLow => sck.set_low().ok(),
                    Polarity::IdleHigh => sck.set_high().ok(),
                };

                for _ in 0..self.delay {
                    unsafe { asm!("nop") };
                }
            }
        }
        Ok(())
    }
}

impl<SCK, MISO, MOSI> Transfer<u8> for Spi<SCK, MISO, MOSI>
where
    SCK: OutputPin,
    MOSI: OutputPin,
    MISO: InputPin,
{
    type Error = MISO::Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let sck = &mut self.sck;
        for word in words.as_mut() {
            for bit in (0..8).rev() {
                let mask = 1 << bit;

                if let Some(mosi) = self.mosi.as_mut() {
                    match *word & mask {
                        0 => mosi.set_low().ok(),
                        _ => mosi.set_high().ok(),
                    };
                }

                match self.mode.polarity {
                    Polarity::IdleLow => sck.set_high().ok(),
                    Polarity::IdleHigh => sck.set_low().ok(),
                };

                for _ in 0..self.delay {
                    unsafe { asm!("nop") };
                }

                if let Some(miso) = self.miso.as_mut() {
                    match miso.is_high()? {
                        true => *word |= mask,
                        false => *word &= !mask,
                    }
                }

                match self.mode.polarity {
                    Polarity::IdleLow => sck.set_low().ok(),
                    Polarity::IdleHigh => sck.set_high().ok(),
                };

                for _ in 0..self.delay {
                    unsafe { asm!("nop") };
                }
            }
        }

        Ok(words)
    }
}
