use std::fs::File;
use std::any::Any;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::{str, mem, vec};
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
enum NumpyType {
    Float64(Vec<f64>),
    Int32(Vec<i32>),
    // Can add more types here
}

#[derive(Debug)]
struct NumpyArray {
    dtype: NumpyType,
    shape: Vec<usize>,
}

impl NumpyArray {
    fn new(header: NumpyHeader, file: &mut File, data_start_pos: usize) -> io::Result<NumpyArray> {
        file.seek(SeekFrom::Start(data_start_pos as u64))?;
        
        // Get the total elements in the array
        let total_elements: usize = header.shape.iter().product();

        println!("total elements in array: {}", total_elements);

        match header.descr.as_str() {
            "<i4" => {
                let mut buffer: Vec<i32> = Vec::with_capacity(total_elements);

                for _ in 0..total_elements {
                    let mut bytes = [0u8; 4]; //i32 is 4 bytes
                    file.read_exact(&mut bytes).unwrap();
                    let value = i32::from_le_bytes(bytes);
                    buffer.push(value);
                }

                println!("This is the array: {:?}", buffer);

                Ok(NumpyArray {
                    dtype: NumpyType::Int32(buffer),
                    shape: header.shape,
                })
            },
            "<f8" => {todo!()},

            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported data type")),
        }
        
        //todo!()
    }

    fn to_nested_vec_i32(&self, start: usize, dimensions: &[usize]) -> Vec<Vec<i32>> {

        // for only 1 dimension
        if dimensions.len() == 1 {
            if let NumpyType::Int32(data) = &self.dtype {
                return vec![data[start..start + dimensions[0]].to_vec()];
            } else {
                panic!("Data type mismatch: Expected Int32");
            }}
            
        todo!()
        }
    
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

    // read the numpy header data into the struct - be aware of unwrap
    let header_dict = parse_header(header_data).unwrap();

    let header_len_size = if major_version == 1 {2} else {4};
    let data_start_pos = 6 + 2 + header_len_size + header_lenght;

    // REMEMBER <i4 describes the data type of the array. < means its stored in little endian order, i stands for integer and 4 stands for the size in bytes of each element in the array
    // So 4 means each integer is stored using 4 bytes (or 32bits)

    Ok((header_dict, data_start_pos))

}

fn main() {

    let file_path = "data/array_3.npy";
    let (dict, siffra) = read_numpy_header(file_path).unwrap();
    println!("Start pos: {}", siffra);
    println!("info gathered from the header: {} {} {:?}", dict.descr, dict.fortran_order, dict.shape);

    let ndarray = NumpyArray::new(dict, &mut File::open(file_path).unwrap(), siffra).unwrap();
    println!("This is the returned nd-array {:?}", ndarray);

    // Check if the data type is Int32 before calling to_nested_vec_i32
    if let NumpyType::Int32(_) = ndarray.dtype {
        let nested_vec = ndarray.to_nested_vec_i32(0, &ndarray.shape);
        println!("Nested Vec (i32): {:?}", nested_vec);
    } else {
        println!("NumpyArray is not of type Int32");
    }
}
