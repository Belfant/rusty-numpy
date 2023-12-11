use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::str;
use std::result;
use serde::de::value::Error;
use serde_json::Result;
use serde::Deserialize;

// Struct to store the header data of the npy file
#[derive(Deserialize)]
struct NumpyHeader {
    descr: String,
    fortran_order: bool,
    shape: Vec<usize>,
}

fn parse_header(header_str: &str) -> Result<NumpyHeader> {
    let json_str = header_str
        .replace('\'', "\"")
        .replace(", }", " }")
        .replace("(", "[")
        .replace(")", "]")
        .replace("True", "true")
        .replace("False", "false");

    // Debug print
    // println!("Transformed JSON string: {}", json_str);

    // Attempt to deserialize and catch errors for debugging
    match serde_json::from_str(&json_str) {
        Ok(header) => Ok(header),
        Err(e) => {
            println!("Error parsing JSON: {:?}", e);
            Err(e)
        }
    }
}

// Read all the neseccary information from the header of the npy file
fn read_numpy_header<P: AsRef<Path>>(path: P) -> result::Result<NumpyHeader, io::Error> {
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

    if major_version != 1 {
        io::Error::new(io::ErrorKind::InvalidInput, 
        format!("Incompatible npy file version {}.{}", major_version, minor_version));
    }

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

    // println!("Magic string: {:?}", magic_string);
    // println!("Major version: {}", major_version);
    // println!("Minor version: {}", minor_version);
    // println!("Header lenght: {:?}", header_lenght);
    // println!("{}", header_data);

    // read the numpy header data into the struct - be aware of unwrap
    let header_dict = parse_header(header_data).unwrap();

    // REMEMBER <i4 describes the data type of the array. < means its stored in little endian order, i stands for integer and 4 stands for the size in bytes of each element in the array
    // So 4 means each integer is stored using 4 bytes (or 32bits)
    //println!("{}", header_dict.descr);
    // println!("{}", header_dict.fortran_order);
    // println!("{:?}", header_dict.shape);

    Ok(header_dict)

}

fn parse_vectors(header: NumpyHeader) {
    todo!()
}

fn main() {

    let dict = read_numpy_header("D:/rusty_numpy/data/array_1.npy").unwrap();
    println!("info gathered from the header: {} {} {:?}", dict.descr, dict.fortran_order, dict.shape);
    // parse_vectors(dict);
}
