use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::str;
use serde::{Serialize, Deserialize};

// Struct to store the header data of the npy file
struct NumpyHeader {
    descr: String,
    fortran_order: bool,
    shape: Vec<usize>,
    // Also include version and magic string? 
}

// Read all the neseccary information from the header of the npy file
fn read_numpy_header<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let mut file = File::open(path)?;
    
    // Read the magic string
    let mut magic_string: [u8; 6] = [0u8; 6];    
    file.read_exact(&mut magic_string)?;
    if &magic_string != b"\x93NUMPY" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Magic String"));
    }
    
    // Read the major and minor version of the npy file format
    // the file pointer is already in the right position after reading the first 6 bytes
    let mut version: [u8; 2] = [0; 2];
    file.read_exact(&mut version)?;
    let major_version = version[0];

    let minor_version = version[1];

    // Read the header lenght 
    let mut header_len_arr: [u8; 2] = [0; 2];
    file.read_exact(&mut header_len_arr)?;
    let header_lenght = u16::from_le_bytes(header_len_arr) as usize;

    // Read the array format (ASCII string)
    let mut header_data_bytes = vec![0u8; header_lenght];
    file.read_exact(&mut header_data_bytes)?;

    // Convert the array data to string
    let header_data = str::from_utf8(&header_data_bytes)
    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in header"))?;

    println!("Magic string: {:?}", magic_string);
    println!("Major version: {}", major_version);
    println!("Minor version: {}", minor_version);
    println!("Header lenght: {:?}", header_lenght);

    println!("{}", header_data);

    Ok(())

}

fn main() {
    println!("Hello, world!");
    read_numpy_header("D:/rusty_numpy/data/array_1.npy");
}
