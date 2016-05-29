// Main driver of the program.

//
// Author: Joshua Holmes
//

use std::str;
use std::env;

mod rom;
mod util;

use rom::Rom;

fn main() {
    // get the filename from the command args and load the ROM
    let args: Vec<_> = env::args().collect();
    let filename = &args[1];

    let rom_file: Rom = match Rom::from_file_path(filename) {
        Err(e) => panic!("Failed to load ROM file. Error message: {:?}", e),
        Ok(rom) => rom,
    };

    // print out all the ROM info
    println!("\nTitle: {}", rom_file.title);
    println!("New style cartridge: {}", rom_file.new_cartridge);
    println!("Manufacturer code: {}", rom_file.manufacturer_code);
    println!("CGB flag: {:?}", rom_file.cgb_flag);
    println!("New licensee code: {}", rom_file.new_licensee_code);
    println!("SGB flag: {:?}", rom_file.sgb_flag);
    println!("Cartridge type: {:?}", rom_file.cartridge_type);
    println!("ROM size: {:?}", rom_file.rom_size);
    println!("RAM size: {:?}", rom_file.ram_size);
    println!("Destination code: {:?}", rom_file.destination_code);
    println!("Old licensee code: {:#X}", rom_file.old_licensee_code);
    println!("Mask ROM version number: {:#X}", rom_file.mask_rom_version_number);
    println!("Header checksum: {:#X} (valid: {})", rom_file.header_checksum, rom_file.is_header_checksum_valid());
    println!("Global checksum: {:#X} (valid: {})", rom_file.global_checksum, rom_file.is_global_checksum_valid());
    println!("Valid Nintendo logo: {}", rom_file.is_nintendo_logo_valid());
}