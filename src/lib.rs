//! Sx128x Radio Driver
//! Copyright 2018 Ryan Kurte

#![no_std]
extern crate libc;

extern crate futures;
extern crate nb;

extern crate embedded_spi;
use embedded_spi::compat::Cursed;
use embedded_spi::{Transactional, wrapper::Wrapper as SpiWrapper};

extern crate embedded_hal as hal;
use hal::blocking::{spi, delay};
use hal::digital::v2::{InputPin, OutputPin};
use hal::spi::{Mode, Phase, Polarity};


pub mod bindings;
use bindings::{self as sx1280, SX1280_s};

pub mod base;

pub mod compat;

/// Sx128x Spi operating mode
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

/// Sx128x device object
#[repr(C)]
pub struct Sx128x<Spi, SpiError, Output, Input, PinError, Delay> {
    spi: Spi,
    cs: Output,

    sdn: Output,
    busy: Input,
    delay: Delay,

    c: Option<SX1280_s>,
    err: Option<Sx128xError<SpiError, PinError>>,
}

// Mark Sx128x object as cursed to forever wander the lands of ffi
impl <Spi, SpiError, Output, Input, PinError, Delay> Cursed for Sx128x <Spi, SpiError, Output, Input, PinError, Delay> {}

pub struct Settings {

}

impl Default for Settings {
    fn default() -> Self {
        Self{}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sx128xError<SpiError, PinError> {
    Spi(SpiError),
    Pin(PinError),
}

impl<Spi, SpiError, Output, Input, PinError, Delay> Sx128x<Spi, SpiError, Output, Input, PinError, Delay>
where
    Spi: spi::Transfer<u8, Error = SpiError> + spi::Write<u8, Error = SpiError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
    Delay: delay::DelayMs<u32>,
{
    pub fn new(spi: Spi, sdn: Output, cs: Output, busy: Input, delay: Delay, _settings: Settings) -> Result<Self, Sx128xError<SpiError, PinError>> {

        let mut sx128x = Sx128x { spi, sdn, cs, busy, delay, c: None, err: None };

        // Reset IC
        sx128x.reset()?;

        // Calibrate RX chain
        //sx1280::RxChainCalibration(&sx128x.c);

        // Init IRQs (..?)

        // Confiure modem(s)

        // Set state to idle


        Ok(sx128x)
    }

    pub fn status(&mut self) -> Result<u8, Sx128xError<SpiError, PinError>> {
        let mut d = [0u8; 1];
        self.cmd_read(sx1280::RadioCommands_u_RADIO_GET_STATUS as u8, &mut d)?;
        Ok(d[0])
    }

}


#[cfg(test)]
mod tests {
    use super::Sx128x;
    use bindings as sx1280;

    extern crate std;
    use tests::std::boxed::Box;
    use tests::std::vec::*;

    extern crate embedded_spi;
    use self::embedded_spi::mock::{Mock, MockTransaction as Mt};

    extern crate embedded_hal;
    use tests::embedded_hal::blocking::spi::{Transfer, Write};

    #[test]
    fn test_mod() {
        let mut m = Mock::new();

        let spi = m.spi();
        let cs = m.pin();
        let sdn = m.pin();
        
        let busy = m.pin();
        let delay = m.delay();

        //let s: Box<Test<_, _>> = Box::new(spi.clone());

        let mut radio = Sx128x{spi: spi.clone(), sdn: sdn.clone(), cs: cs.clone(), busy: busy.clone(), delay: delay.clone(), c: None, err: None};

        Sx128x::bind(&mut radio);

        std::println!("Test reset command");

        m.expect(&[
            Mt::set_low(&sdn),
            Mt::delay_ms(&delay, 1),
            Mt::set_high(&sdn),
            Mt::delay_ms(&delay, 10),
        ]);

        radio.reset().unwrap();

        m.finalise();

        std::println!("Test status command");

        m.expect(&[
            Mt::set_low(&cs),
            Mt::write(&spi, &[sx1280::RadioCommands_u_RADIO_GET_STATUS as u8, 0]),
            Mt::transfer(&spi, &[0x00], &[0x00]),
            Mt::set_high(&cs),
        ]);

        radio.status().unwrap();

        m.finalise();
    }
}
