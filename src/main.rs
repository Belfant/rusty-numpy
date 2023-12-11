use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::{str, mem};
use regex::Regex; 
use serde::Deserialize;

// There's a difference in how many byte the header is in version 1 and 2

// Struct to store the header data of the npy file
#[derive(Deserialize)]
struct NumpyHeader {
    descr: String,
    fortran_order: bool,
    shape: Vec<usize>,
}

#[derive(Debug)]
enum NumpyData {
    IntVec1D(Vec<i32>),
    FloatVec1D(Vec<f64>),
    IntVec2D(Vec<Vec<i32>>),
    FloatVec2D(Vec<Vec<f64>>),
    // ToDo add more dimensions
}

fn parse_header(header_str: &str) -> Result<NumpyHeader, serde_json::Error> {
    // Regular expressions for transformations
    let re_bool = Regex::new(r"False|True").unwrap();
    let re_tuple = Regex::new(r"\(([^\)]*)\)").unwrap();
    let re_trailing_comma = Regex::new(r",\s*([}\]])").unwrap();

    // Replace Python booleans with Rust booleans
    let mut transformed_str = re_bool.replace_all(header_str, |caps: &regex::Captures| {
        match caps.get(0).map_or("", |m| m.as_str()) {
            "True" => "true",
            "False" => "false",
            _ => "",
        }
    }).to_string();

    // Transform Python tuples into Rust vectors
    transformed_str = re_tuple.replace_all(&transformed_str, |caps: &regex::Captures| {
        "[".to_owned() + caps.get(1).map_or("", |m| m.as_str()) + "]"
    }).to_string();

    // Remove trailing commas before a closing brace or bracket
    transformed_str = re_trailing_comma.replace_all(&transformed_str, "$1").to_string();

    // Replace single quotes with double quotes for JSON compatibility
    transformed_str = transformed_str.replace('\'', "\"");

    // Debug print
    println!("Transformed string: {}", transformed_str);

    // Deserialize and handle errors
    serde_json::from_str(&transformed_str)
}

// Read all the neseccary information from the header of the npy file
fn read_numpy_header<P: AsRef<Path>>(path: P) -> io::Result<(NumpyHeader, usize)> {
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
        return Err(io::Error::new(io::ErrorKind::InvalidInput, 
        format!("Incompatible npy file version {}.{}", major_version, minor_version)));
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

    println!("Magic string: {:?}", magic_string);
    println!("Major version: {}", major_version);
    println!("Minor version: {}", minor_version);
    println!("Header lenght: {:?}", header_lenght);
    // println!("{}", header_data);

    // read the numpy header data into the struct - be aware of unwrap
    let header_dict = parse_header(header_data).unwrap();

    let header_len_size = if major_version == 1 {2} else {4};
    let data_start_pos = 6 + 2 + header_len_size + header_lenght;

    // REMEMBER <i4 describes the data type of the array. < means its stored in little endian order, i stands for integer and 4 stands for the size in bytes of each element in the array
    // So 4 means each integer is stored using 4 bytes (or 32bits)
    println!("{}", header_dict.descr);
    println!("{}", header_dict.fortran_order);
    println!("{:?}", header_dict.shape);

    Ok((header_dict, data_start_pos))

}


fn parse_vectors(header: NumpyHeader, file_path: &str, data_start_pos: usize) -> io::Result<NumpyData> {
    let mut file = File::open(file_path)?;
    file.seek(SeekFrom::Start(data_start_pos as u64))?;

    match header.descr.as_str() {
        "<i4" => {
            let flat_data: Vec<i32> = read_data::<i32>(&mut file)?;
            match header.shape.len() {
                1 => Ok(NumpyData::IntVec1D(flat_data)),
                2 => Ok(NumpyData::IntVec2D(reshape_vector(flat_data, &header.shape))),
                _ => Err(io::Error::new(io::ErrorKind::Other, "Unsupported array dimension")),
            }
        },
        "<f8" => {
            let flat_data: Vec<f64> = read_data::<f64>(&mut file)?;
            match header.shape.len() {
                1 => Ok(NumpyData::FloatVec1D(flat_data)),
                2 => Ok(NumpyData::FloatVec2D(reshape_vector(flat_data, &header.shape))),
                _ => Err(io::Error::new(io::ErrorKind::Other, "Unsupported array dimension")),
            }
        },
        _ => Err(io::Error::new(io::ErrorKind::Other, "Unsupported data type")),
    }
}

fn read_data<T>(file: &mut File) -> io::Result<Vec<T>>
where
    T: FromLeBytes + Default,
{
    let mut data = Vec::new();
    let mut buffer = vec![0u8; mem::size_of::<T>()];

    while let Ok(()) = file.read_exact(&mut buffer) {
        data.push(T::from_le_bytes(&buffer));
    }

    Ok(data)
}

fn reshape_vector<T>(flat_vec: Vec<T>, shape: &[usize]) -> Vec<Vec<T>>
where
    T: Clone,
{
    if shape.len() != 2 {
        panic!("Currently only 2D arrays are supported");
    }

    let _rows = shape[0];
    let cols = shape[1];

    flat_vec
        .chunks(cols)
        .map(|chunk| chunk.to_vec())
        .collect()
}

trait FromLeBytes {
    fn from_le_bytes(bytes: &[u8]) -> Self;
}

impl FromLeBytes for i32 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        i32::from_le_bytes(bytes.try_into().expect("Invalid byte slice length"))
    }
}

impl FromLeBytes for f64 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        f64::from_le_bytes(bytes.try_into().expect("Invalid byte slice length"))
    }
}

fn main() {

    let file_path = "data/array_2.npy";
    let (dict, siffra) = read_numpy_header(file_path).unwrap();
    println!("Start pos: {}", siffra);
    println!("info gathered from the header: {} {} {:?}", dict.descr, dict.fortran_order, dict.shape);

    match parse_vectors(dict, file_path, siffra) {
        Ok(vectors) => println!("Parsed data: {:?}", vectors),
        Err(e) => println!("Errors parsing data: {}", e),
    }

}
