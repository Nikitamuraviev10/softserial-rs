use crate::hal;
use core::arch::asm;
use hal::blocking::spi::{Transfer, Write};
use hal::digital::v2::{InputPin, OutputPin};

pub struct Spi<I, O> {
    sck: Option<O>,
    miso: Option<I>,
    mosi: Option<O>,
    delay: usize,
}

impl<I, O> Spi<I, O>
where
    I: InputPin,
    O: OutputPin,
{
    pub fn new(sck: Option<O>, miso: Option<I>, mosi: Option<O>, delay: usize) -> Self {
        Self {
            sck,
            miso,
            mosi,
            delay,
        }
    }
}

impl<I, O> Write<u8> for Spi<I, O>
where
    I: InputPin,
    O: OutputPin,
{
    type Error = ();
    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        for word in words {
            for bit in (0..8).rev() {
                let mask = 1 << bit;

                if let Some(mosi) = self.mosi.as_mut() {
                    match word & mask {
                        0 => mosi.set_high().ok(),
                        _ => mosi.set_low().ok(),
                    };
                }

                if let Some(sck) = self.sck.as_mut() {
                    sck.set_high().ok();
                    for _ in 0..self.delay {
                        unsafe { asm!("nop") };
                    }
                    sck.set_low().ok();
                    for _ in 0..self.delay {
                        unsafe { asm!("nop") };
                    }
                }
            }
        }
        Ok(())
    }
}

impl<I, O> Transfer<u8> for Spi<I, O>
where
    I: InputPin,
    O: OutputPin,
{
    type Error = I::Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        for word in words.as_mut() {
            for bit in (0..8).rev() {
                let mask = 1 << bit;

                if let Some(mosi) = self.mosi.as_mut() {
                    match *word & mask {
                        0 => mosi.set_high().ok(),
                        _ => mosi.set_low().ok(),
                    };
                }

                if let Some(sck) = self.sck.as_mut() {
                    sck.set_high().ok();
                    for _ in 0..self.delay {
                        unsafe { asm!("nop") };
                    }

                    if let Some(miso) = self.miso.as_mut() {
                        match miso.is_high()? {
                            true => *word |= mask,
                            false => *word &= !mask,
                        }
                    }
                    sck.set_low().ok();
                    for _ in 0..self.delay {
                        unsafe { asm!("nop") };
                    }
                }
            }
        }

        Ok(words)
    }
}
