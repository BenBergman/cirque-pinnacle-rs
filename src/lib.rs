#![no_std]

//use bitfield::bitfield; // TODO: use this?
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;

#[derive(Clone, Copy, Debug)]
pub enum Error<SPI: Transfer<u8>, CS: OutputPin, DR: InputPin> {
    Spi(SPI::Error),
    Cs(CS::Error),
    Dr(DR::Error),
    Other,
}

pub struct Driver<SPI: Transfer<u8>, CS: OutputPin, DR: InputPin> {
    spi: SPI, // TODO: Add I2C compatibility
    cs: CS,   // TODO: make this an option?
    dr: DR,   // TODO: Make this an Option?
}

// TODO: implement simplest setup and interface (match arduino example)
// TODO: implement feature complete interface - builder pattern?

impl<SPI: Transfer<u8>, CS: OutputPin, DR: InputPin> Driver<SPI, CS, DR> {
    pub fn new(spi: SPI, cs: CS, dr: DR) -> Result<Self, Error<SPI, CS, DR>> {
        // TODO: return result confirming setup worked
        //  - possibly check chip's ID?
        let mut driver = Self { cs, dr, spi };
        driver.clear_flags()?;

        // TODO: better function names? bitfields?
        driver.init_sys_config()?;
        driver.init_feed_config_2()?;
        driver.init_feed_config_1()?;
        driver.init_z_idle_count()?;

        Ok(driver)
    }

    pub fn data_ready(&self) -> Result<bool, <DR as InputPin>::Error> {
        self.dr.is_high()
    }

    pub fn get_absolute(&mut self) -> Result<Touch, Error<SPI, CS, DR>> {
        let mut buffer = [0; 6];
        self.rap_read(chip::Addr::PacketByte0, &mut buffer)?;
        Ok(Touch {
            // TODO: would bitfields clean this up?
            x: buffer[2] as u16 | ((buffer[4] as u16 & 0x0F) << 8),
            y: buffer[3] as u16 | ((buffer[4] as u16 & 0xF0) << 4),
            z: buffer[5] as u16 & 0x3F,
        })
    }

    pub fn clear_flags(&mut self) -> Result<(), Error<SPI, CS, DR>> {
        self.rap_write(chip::Addr::Status1, 0x00)
        // TODO: delayMicroseconds(50); // TODO: add non-blocking delay?
    }

    fn init_sys_config(&mut self) -> Result<(), Error<SPI, CS, DR>> {
        self.rap_write(chip::Addr::SysConfig1, 0x00)
    }

    fn init_feed_config_1(&mut self) -> Result<(), Error<SPI, CS, DR>> {
        // TODO: better bitfield setting and/or function names
        self.rap_write(chip::Addr::FeedConfig1, 0x03)
    }

    fn init_feed_config_2(&mut self) -> Result<(), Error<SPI, CS, DR>> {
        self.rap_write(chip::Addr::FeedConfig2, 0x1F)
    }

    fn init_z_idle_count(&mut self) -> Result<(), Error<SPI, CS, DR>> {
        self.rap_write(chip::Addr::ZIdle, 0x05)
    }

    fn rap_write(&mut self, address: chip::Addr, data: u8) -> Result<(), Error<SPI, CS, DR>> {
        let mut buffer: [u8; 2] = [chip::WRITE_MASK | address as u8, data];

        self.assert_cs()?;
        let result = self.spi.transfer(&mut buffer);
        self.deassert_cs()?;

        result.map(|_| ()).map_err(Error::Spi)
    }

    fn rap_read(
        &mut self,
        address: chip::Addr,
        buffer: &mut [u8],
    ) -> Result<(), Error<SPI, CS, DR>> {
        let mut cmd_buf: [u8; 3] = [chip::READ_MASK | address as u8, 0xFC, 0xFC]; // 0xFC are filler bytes in Cirque's Arduino example; TODO - try removing it?

        // TODO: handle all the errors from SPI and whatnot

        self.assert_cs()?;
        let mut result = self.spi.transfer(&mut cmd_buf); // TODO: deal with error
        if result.is_ok() {
            result = self.spi.transfer(buffer);
        }
        self.deassert_cs()?;

        result.map(|_| ()).map_err(Error::Spi)
    }

    fn assert_cs(&mut self) -> Result<(), Error<SPI, CS, DR>> {
        self.cs.set_low().map_err(Error::Cs)
    }

    fn deassert_cs(&mut self) -> Result<(), Error<SPI, CS, DR>> {
        self.cs.set_high().map_err(Error::Cs)
    }
}

pub struct Touch {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

mod chip {
    pub const WRITE_MASK: u8 = 0x80;
    pub const READ_MASK: u8 = 0xA0;

    pub enum Addr {
        // TODO: fill this out for completeness
        Status1 = 0x02,
        SysConfig1 = 0x03,
        FeedConfig1 = 0x04,
        FeedConfig2 = 0x05,
        ZIdle = 0x0A,
        PacketByte0 = 0x12,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
