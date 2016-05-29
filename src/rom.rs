// Abstraction for dealing with ROM files.

//
// Author: Joshua Holmes
//

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::str;

use util;

// Header address constants
const ENTRY_POINT_ADDR: usize = 0x0100;
const NINTENDO_LOGO_ADDR: usize = 0x0104;
const TITLE_ADDR: usize = 0x0134;
const MANUFACTURER_CODE_ADDR: usize = 0x013F;
const CGB_FLAG_ADDR: usize = 0x0143;
const NEW_LICENSEE_CODE_ADDR: usize = 0x0144;
const SGB_FLAG_ADDR: usize = 0x0146;
const CARTRIDGE_TYPE_ADDR: usize = 0x0147;
const ROM_SIZE_ADDR: usize = 0x0148;
const RAM_SIZE_ADDR: usize = 0x0149;
const DESTINATION_CODE_ADDR: usize = 0x014A;
const OLD_LICENSEE_CODE_ADDR: usize = 0x014B;
const MASK_ROM_VERSION_NUMBER_ADDR: usize = 0x014C;
const HEADER_CHECKSUM_ADDR: usize = 0x014D;
const GLOBAL_CHECKSUM_ADDR: usize = 0x014E;

/// Flag that says a cartridge is the new format
const NEW_CARTRIDGE_FLAG: u8 = 0x33;

/// Nintendo logo constant -- the header should contain this
const VALID_NINTENDO_LOGO: [u8; 48] = 
    [0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
     0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
     0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E];

/// RomLoadError borrowed from sprocketnes
#[derive(Debug)]
pub enum RomLoadError {
    /// IO error while reading the ROM image
    IoError(io::Error),
    /// The ROM image has an invalid format
    FormatError(String),
}

impl From<io::Error> for RomLoadError {
    fn from(err: io::Error) -> Self {
        RomLoadError::IoError(err)
    }
}

impl From<String> for RomLoadError {
    fn from(err: String) -> Self {
        RomLoadError::FormatError(err)
    }
}

/// Represents the flags that specify GameBoy Color functionality
#[derive(Debug)]
pub enum CgbFlag {
    NoCgb = 0x00,
    SupportsCgb = 0x80,
    CgbOnly = 0xC0,
}

impl CgbFlag {
    fn from_u8(n: u8) -> Option<CgbFlag> {
        use self::CgbFlag::*;

        match n {
            0x00 => Some(NoCgb),
            0x80 => Some(SupportsCgb),
            0xC0 => Some(CgbOnly),
            _ => None
        }
    }
}

/// Represents the flags that specify Super GameBoy functionality
#[derive(Debug)]
pub enum SgbFlag {
    NoSgbSupport = 0x00,
    SgbSupport = 0x03,
}

impl SgbFlag {
    fn from_u8(n: u8) -> Option<SgbFlag> {
        use self::SgbFlag::*;

        match n {
            0x00 => Some(NoSgbSupport),
            0x03 => Some(SgbSupport),
            _ => None
        }
    }
}

/// Represents the various cartridge types that exist
#[derive(Debug)]
pub enum CartridgeType {
    Rom = 0x00,
    Mbc1 = 0x01,
    Mbc1Ram = 0x02,
    Mbc1RamBattery = 0x03,
    Mbc2 = 0x05,
    Mbc2Battery = 0x06,
    RomRam = 0x08,
    RomRamBattery = 0x09,
    Mmm01 = 0x0B,
    Mmm01Ram = 0x0C,
    Mmm01RamBattery = 0x0D,
    Mbc3TimerBattery = 0x0F,
    Mbc3TimerRamBattery = 0x10,
    Mbc3 = 0x11,
    Mbc3Ram = 0x12,
    Mbc3RamBattery = 0x13,
    Mbc4 = 0x15,
    Mbc4Ram = 0x16,
    Mbc4RamBattery = 0x17,
    Mbc5 = 0x19,
    Mbc5Ram = 0x1A,
    Mbc5RamBattery = 0x1B,
    Mbc5Rumble = 0x1C,
    Mbc5RumbleRam = 0x1D,
    Mbc5RumbleRamBattery = 0x1E,
    Mbc6 = 0x20,
    Mbc7SensorRumbleRamBattery = 0x22,
    PocketCamera = 0xFC,
    BandaiTama5 = 0xFD,
    HuC3 = 0xFE,
    HuC1RamBattery = 0xFF,
}

impl CartridgeType {
    fn from_u8(n: u8) -> Option<CartridgeType> {
        use self::CartridgeType::*;

        match n {
            0x00 => Some(Rom),
            0x01 => Some(Mbc1),
            0x02 => Some(Mbc1Ram),
            0x03 => Some(Mbc1RamBattery),
            0x05 => Some(Mbc2),
            0x06 => Some(Mbc2Battery),
            0x08 => Some(RomRam),
            0x09 => Some(RomRamBattery),
            0x0B => Some(Mmm01),
            0x0C => Some(Mmm01Ram),
            0x0D => Some(Mmm01RamBattery),
            0x0F => Some(Mbc3TimerBattery),
            0x10 => Some(Mbc3TimerRamBattery),
            0x11 => Some(Mbc3),
            0x12 => Some(Mbc3Ram),
            0x13 => Some(Mbc3RamBattery),
            0x15 => Some(Mbc4),
            0x16 => Some(Mbc4Ram),
            0x17 => Some(Mbc4RamBattery),
            0x19 => Some(Mbc5),
            0x1A => Some(Mbc5Ram),
            0x1B => Some(Mbc5RamBattery),
            0x1C => Some(Mbc5Rumble),
            0x1D => Some(Mbc5RumbleRam),
            0x1E => Some(Mbc5RumbleRamBattery),
            0x20 => Some(Mbc6),
            0x22 => Some(Mbc7SensorRumbleRamBattery),
            0xFC => Some(PocketCamera),
            0xFD => Some(BandaiTama5),
            0xFE => Some(HuC3),
            0xFF => Some(HuC1RamBattery),
            _ => None
        }
    }
}

/// Represents the varying amounts of ROM sizes that exist
pub enum RomSize {
    RomBanks0 = 0x00,
    RomBanks4 = 0x01,
    RomBanks8 = 0x02,
    RomBanks16 = 0x03,
    RomBanks32 = 0x04,
    RomBanks64 = 0x05,
    RomBanks128 = 0x06,
    RomBanks256 = 0x07,
    RomBanks72 = 0x52,
    RomBanks80 = 0x53,
    RomBanks96 = 0x54,
}

impl RomSize {
    fn from_u8(n: u8) -> Option<RomSize> {
        use self::RomSize::*;

        match n {
            0x00 => Some(RomBanks0),
            0x01 => Some(RomBanks4),
            0x02 => Some(RomBanks8),
            0x03 => Some(RomBanks16),
            0x04 => Some(RomBanks32),
            0x05 => Some(RomBanks64),
            0x06 => Some(RomBanks128),
            0x07 => Some(RomBanks256),
            0x52 => Some(RomBanks72),
            0x53 => Some(RomBanks80),
            0x54 => Some(RomBanks96),
            _ => None
        }
    }
}

impl fmt::Debug for RomSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::RomSize::*;

        write!(f, "{}", match *self {
            RomBanks0 => "32KByte",
            RomBanks4 => "64KByte",
            RomBanks8 => "128KByte",
            RomBanks16 => "256KByte",
            RomBanks32 => "512KByte",
            RomBanks64 => "1MByte",
            RomBanks128 => "2MByte",
            RomBanks256 => "4MByte",
            RomBanks72 => "1.1MByte",
            RomBanks80 => "1.2MByte",
            RomBanks96 => "1.5MByte",
        })
    }
}

/// Represents the vaying amounts of on-cartridge RAM sizes that exist
pub enum RamSize {
    RamNone = 0x00,
    Ram2K = 0x01,
    Ram8K = 0x02,
    Ram32K = 0x03,
    Ram64K = 0x05,
    Ram128K = 0x04,
}

impl RamSize {
    fn from_u8(n: u8) -> Option<RamSize> {
        use self::RamSize::*;

        match n {
            0x00 => Some(RamNone),
            0x01 => Some(Ram2K),
            0x02 => Some(Ram8K),
            0x03 => Some(Ram32K),
            0x04 => Some(Ram128K),
            0x05 => Some(Ram64K),
            _ => None
        }
    }
}

impl fmt::Debug for RamSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::RamSize::*;

        write!(f, "{}", match *self {
            RamNone => "None",
            Ram2K => "2KByte",
            Ram8K => "8KByte",
            Ram32K => "32KByte",
            Ram64K => "64KByte",
            Ram128K => "128KByte",
        })
    }
}

/// Represents the ROM's destination code
#[derive(Debug)]
pub enum DestinationCode {
    Japanese = 0x00,
    NonJapanese = 0x01,
}

impl DestinationCode {
    fn from_u8(n: u8) -> Option<DestinationCode> {
        use self::DestinationCode::*;

        match n {
            0x00 => Some(Japanese),
            0x01 => Some(NonJapanese),
            _ => None
        }
    }
}

/// Represents a ROM file and its header metadata
pub struct Rom {
    pub entry_point: [u8; 4],
    pub nintendo_logo: [u8; 48],
    pub title: String,
    pub manufacturer_code: String,
    pub cgb_flag: CgbFlag,
    pub new_licensee_code: String,
    pub sgb_flag: SgbFlag,
    pub cartridge_type: CartridgeType,
    pub rom_size: RomSize,
    pub ram_size: RamSize,
    pub destination_code: DestinationCode,
    pub old_licensee_code: u8,
    pub mask_rom_version_number: u8,
    pub header_checksum: u8,
    pub global_checksum: u16,
    pub new_cartridge: bool,
    pub rom_data: Vec<u8>,
}

impl Rom {
    /// Takes in a file path string and returns a Rom
    pub fn from_file_path(filepath: &str) -> Result<Rom, RomLoadError> {
        let path = Path::new(filepath);

        let mut file = match File::open(&path) {
            Err(e) => panic!("Couldn't open ROM file. Error message: {}", Error::description(&e)),
            Ok(file) => file,
        };

        Rom::from_file(&mut file)
    }

    /// Takes in a File object and reads the data into a Rom structure
    pub fn from_file(file: &mut File) -> Result<Rom, RomLoadError> {
        // read the ROM into a buffer
        let mut buf = Vec::new();

        match file.read_to_end(&mut buf) {
            Err(e) => panic!("Couldn't read ROM file. Error message: {}", Error::description(&e)),
            Ok(_) => (),
        };

        Rom::from_buffer(buf)
    }

    /// Takes in a u8 vector and returns a Rom structure
    pub fn from_buffer(buf: Vec<u8>) -> Result<Rom, RomLoadError> {
        // if the ROM size is less than or equal to the size needed to simply 
        // store the cartridge header, then it's invalid
        if buf.len() <= GLOBAL_CHECKSUM_ADDR + 1 {
            return Err(RomLoadError::FormatError(format!("ROM file is too small. Size: {} bytes", buf.len())))
        }

        // see if this cartridge is new-style or old-style
        let new_cartridge = buf[OLD_LICENSEE_CODE_ADDR] == NEW_CARTRIDGE_FLAG;
        let title_end_addr = if new_cartridge { MANUFACTURER_CODE_ADDR } else { NEW_LICENSEE_CODE_ADDR };

        // read the multi-byte values into our buffers
        let mut entry_point = [0u8; 4];
        let mut nintendo_logo = [0u8; 48];

        util::get_subarray_of_vector(&mut entry_point, &buf, ENTRY_POINT_ADDR);
        util::get_subarray_of_vector(&mut nintendo_logo, &buf, NINTENDO_LOGO_ADDR);

        // read the enum flags
        let cgb_flag = if new_cartridge { try!(CgbFlag::from_u8(buf[CGB_FLAG_ADDR]).ok_or_else(|| {
            format!("Invalid CGB flag: {:#X}", buf[CGB_FLAG_ADDR])
        })) } else {
            CgbFlag::NoCgb
        };

        let sgb_flag = try!(SgbFlag::from_u8(buf[SGB_FLAG_ADDR]).ok_or_else(|| {
            format!("Invalid SGB flag: {:#X}", buf[SGB_FLAG_ADDR])
        }));

        let cartridge_type = try!(CartridgeType::from_u8(buf[CARTRIDGE_TYPE_ADDR]).ok_or_else(|| {
            format!("Invalid cartridge type flag: {:#X}", buf[CARTRIDGE_TYPE_ADDR])
        }));

        let rom_size = try!(RomSize::from_u8(buf[ROM_SIZE_ADDR]).ok_or_else(|| {
            format!("Invalid ROM size flag: {:#X}", buf[ROM_SIZE_ADDR])
        }));

        let ram_size = try!(RamSize::from_u8(buf[RAM_SIZE_ADDR]).ok_or_else(|| {
            format!("Invalid RAM size flag: {:#X}", buf[RAM_SIZE_ADDR])
        }));

        let destination_code = try!(DestinationCode::from_u8(buf[DESTINATION_CODE_ADDR]).ok_or_else(|| {
            format!("Invalid destination code: {:#X}", buf[DESTINATION_CODE_ADDR])
        }));

        Ok(Rom {
            entry_point: entry_point,
            nintendo_logo: nintendo_logo,
            title: util::bytes_to_string(&buf[TITLE_ADDR..title_end_addr]).to_owned(),
            manufacturer_code: if new_cartridge { util::bytes_to_string(&buf[MANUFACTURER_CODE_ADDR..CGB_FLAG_ADDR]).to_owned() } else { "".to_owned() },
            new_licensee_code: if new_cartridge { util::bytes_to_string(&buf[NEW_LICENSEE_CODE_ADDR..SGB_FLAG_ADDR]).to_owned() } else { "".to_owned() },
            cgb_flag: cgb_flag,
            sgb_flag: sgb_flag,
            cartridge_type: cartridge_type,
            rom_size: rom_size,
            ram_size: ram_size,
            destination_code: destination_code,
            old_licensee_code: buf[OLD_LICENSEE_CODE_ADDR],
            mask_rom_version_number: buf[MASK_ROM_VERSION_NUMBER_ADDR],
            header_checksum: buf[HEADER_CHECKSUM_ADDR],
            global_checksum: ((buf[GLOBAL_CHECKSUM_ADDR] as u16) << 8) | (buf[GLOBAL_CHECKSUM_ADDR + 1] as u16),
            new_cartridge: new_cartridge,
            rom_data: buf,
        })
    }

    /// Says whether the header checksum is valid
    pub fn is_header_checksum_valid(&self) -> bool {
        let mut calculated_header_checksum = 0u16;

        for i in TITLE_ADDR..HEADER_CHECKSUM_ADDR {
            calculated_header_checksum = calculated_header_checksum.wrapping_sub(self.rom_data[i] as u16).wrapping_sub(1);
        }

        (calculated_header_checksum as u8) == self.header_checksum
    }

    /// Says whether the global checksum is valid
    pub fn is_global_checksum_valid(&self) -> bool {
        let mut calculated_global_checksum = 0u16;

        for (i, x) in self.rom_data.iter().enumerate() {
            if i != GLOBAL_CHECKSUM_ADDR && i != (GLOBAL_CHECKSUM_ADDR + 1) {
                calculated_global_checksum = calculated_global_checksum.wrapping_add(*x as u16);
            }
        }

        calculated_global_checksum == self.global_checksum
    }

    /// Says whether the Nintendo logo is valid
    pub fn is_nintendo_logo_valid(&self) -> bool {
        self.nintendo_logo.iter().zip(VALID_NINTENDO_LOGO.iter()).all(|(a, b)| a == b) 
    }
}