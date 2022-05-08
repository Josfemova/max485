#![no_std]
use embedded_hal::{
    digital::v2::OutputPin,
    serial::{Read, Write},
};
use core::fmt;
use nb;

/// Custom Error type
pub enum Error<E> {
    Max485Error(E),
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

impl<RIDO, REDE, E> Max485<RIDO, REDE>
where
    RIDO: Read<u8> + Write<u8>,
    REDE: OutputPin<Error = E>,
{
    pub fn new(serial: RIDO, pin: REDE)-> Self{
        Self{serial, pin}
    }
    pub fn take_peripherals(self)->(RIDO, REDE){
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

impl<RIDO, REDE, E> fmt::Write for Max485<RIDO, REDE>
where
    RIDO: Read<u8> + Write<u8, Error = Error<E>>,
    REDE: OutputPin<Error = E>,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        self.pin.set_high().map_err(|_| fmt::Error)?;
        for b in bytes{
            nb::block!(self.serial.write(*b)).map_err(|_| fmt::Error)?;
        }
        self.pin.set_low().map_err(|_| fmt::Error)?;
        Ok(())
    }
}

impl<RIDO, REDE, E> Write<u8> for Max485<RIDO, REDE>
where
    RIDO: Read<u8> + Write<u8, Error = Error<E>>,
    REDE: OutputPin<Error = E>,
{
    type Error = crate::Error<E>;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.pin.set_high().map_err( Error::Max485Error)?;
        let res = self.serial.write(word);
        self.pin.set_low().map_err( Error::Max485Error)?;
        res
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.pin.set_high().map_err(Error::Max485Error)?;
        let res = self.serial.flush();
        self.pin.set_low().map_err( Error::Max485Error)?;
        res
    }
}

impl<RIDO, REDE, E> Read<u8> for Max485<RIDO, REDE>
where
    RIDO: Read<u8, Error= Error<E>> + Write<u8>,
    REDE: OutputPin<Error = E>,
{
    type Error = Error<E>;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        //assert pin is low
        self.pin.set_low().map_err(Error::Max485Error)?;
        self.serial.read()
    }
}
