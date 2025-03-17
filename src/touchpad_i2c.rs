use std::fmt::Debug;
use std::io::ErrorKind::{NotFound, PermissionDenied};

use anyhow::{Context, Error, Result};
use i2cdev::core::I2CDevice;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

#[derive(Debug, Clone, Copy)]
pub enum Brightness {
    Off = 0,
    On = 1,
    Low = 65,
    Half = 68,
    Full = 72,
}

impl Default for Brightness {
    fn default() -> Self {
        Brightness::On
    }
}

impl std::fmt::Display for Brightness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Brightness::*;
        let level = match self {
            Off => "Off",
            On => "On",
            Low => "Low",
            Half => "Half",
            Full => "Full",
        };
        f.write_str(level)
    }
}

impl Brightness {
    fn next(&self) -> Self {
        use Brightness::*;
        match self {
            Off => Default::default(), // Jump to default
            On => Low,
            Low => Half,
            Half => Full,
            Full => Low,
        }
    }

    pub fn cycle(&mut self) -> Self {
        *self = self.next();
        *self
    }
}

pub struct TouchpadI2C {
    dev: LinuxI2CDevice,
    i2c_id: u32,
}

impl TouchpadI2C {
    pub fn new(i2c_id: u32) -> Result<Self> {
        const TOUCHPAD_ADDR: u16 = 0x15;
        let dev = unsafe {
            LinuxI2CDevice::force_new(format!("/dev/i2c-{}", i2c_id), TOUCHPAD_ADDR).map_err(
                |err| {
                    let mut context = format!("Unable to open Touchpad I2C at /dev/i2c-{}", i2c_id);
                    let extra_context = match &err {
                        LinuxI2CError::Io(e) => match e.kind() {
                            NotFound => "Is i2c-dev kernel module loaded?",
                            PermissionDenied => "Do you have the permission to read /dev/i2c-*?",
                            _ => "",
                        },
                        LinuxI2CError::Nix(_) => "",
                    };
                    if !extra_context.is_empty() {
                        context.push_str(". ");
                        context.push_str(extra_context);
                    };
                    Error::new(err).context(context)
                },
            )?
        };
        Ok(Self { dev, i2c_id })
    }

    pub fn set_brightness(&mut self, brightness: Brightness) -> Result<()> {
        let msg = [
            0x05,
            0x00,
            0x3d,
            0x03,
            0x06,
            0x00,
            0x07,
            0x00,
            0x0d,
            0x14,
            0x03,
            brightness as u8,
            0xad,
        ];
        self.dev
            .write(&msg)
            .with_context(|| format!("Could not set touchpad brightness to {}", brightness))
    }
}

impl Debug for TouchpadI2C {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("TouchpadI2C: /dev/i2c-{}", self.i2c_id))
    }
}
