#![no_std]
use core::fmt;
use embedded_hal::{
    digital::v2::OutputPin,
    serial::{Read, Write},
};

/// Custom Error type
#[derive(Debug)]
pub enum Error {
    PinError,
    SerialError,
}

/// Represents the module itself Uses a normal serial port +  a pin
/// to control wether the module is in read or write mode.
pub struct Max485<RIDO, REDE>
where
    RIDO: Read<u8> + Write<u8>,
    REDE: OutputPin,
{
    serial: RIDO,
    pin: REDE,
}

impl<RIDO, REDE> Max485<RIDO, REDE>
where
    RIDO: Read<u8> + Write<u8>,
    REDE: OutputPin,
{
    pub fn new(serial: RIDO, pin: REDE) -> Self {
        Self { serial, pin }
    }
    pub fn take_peripherals(self) -> (RIDO, REDE) {
        (self.serial, self.pin)
    }
    /// Provide a configuration function to be applied to the underlying serial port.
    pub fn reconfig_port<F>(&mut self, config: F)
    where
        F: Fn(&mut RIDO),
    {
        config(&mut self.serial);
    }
}

impl<RIDO, REDE> fmt::Write for Max485<RIDO, REDE>
where
    RIDO: Read<u8> + Write<u8>,
    REDE: OutputPin,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        self.pin.set_high().map_err(|_| fmt::Error)?;
        for b in bytes {
            nb::block!(self.serial.write(*b)).map_err(|_| fmt::Error)?;
        }
        self.pin.set_low().map_err(|_| fmt::Error)?;
        Ok(())
    }
}

impl<RIDO, REDE> Write<u8> for Max485<RIDO, REDE>
where
    RIDO: Read<u8> + Write<u8>,
    REDE: OutputPin,
{
    type Error = crate::Error;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.pin.set_high().map_err(|_| Error::PinError)?;
        self.serial.write(word).map_err(|err|match err{
            nb::Error::Other(_) => nb::Error::Other(Error::SerialError),
            nb::Error::WouldBlock => nb::Error::WouldBlock,
        })?;
        self.pin.set_low().map_err(|_| Error::PinError)?;
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.pin.set_high().map_err(|_| Error::PinError)?;
        self.serial.flush().map_err(|err|match err{
            nb::Error::Other(_) => nb::Error::Other(Error::SerialError),
            nb::Error::WouldBlock => nb::Error::WouldBlock,
        })?;
        self.pin.set_low().map_err(|_| Error::PinError)?;
        Ok(())
    }
}

impl<RIDO, REDE> Read<u8> for Max485<RIDO, REDE>
where
    RIDO: Read<u8> + Write<u8>,
    REDE: OutputPin,
{
    type Error = crate::Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        //assert pin is low
        self.pin.set_low().map_err(|_| Error::PinError)?;
        self.serial.read(). map_err(|err|match err{
            nb::Error::Other(_) => nb::Error::Other(Error::SerialError),
            nb::Error::WouldBlock => nb::Error::WouldBlock,
        })
    }
}
