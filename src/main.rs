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

    // Works well for 2D arrays
    fn to_nested_vec_i32(&self, start: usize, dimensions: &[usize]) -> Vec<Vec<i32>> {

        // for only 1 dimension
        if dimensions.len() == 1 {
            if let NumpyType::Int32(data) = &self.dtype {
                return vec![data[start..start + dimensions[0]].to_vec()];
            } else {
                panic!("Data type mismatch: Expected Int32");
            }}

        
        // for two dimensions
        let mut nested_vec: Vec<Vec<i32>> = Vec::new();
        let subarray_size: usize = dimensions[1..].iter().product();

        for i in 0..dimensions[0] {
            let sub_start = start + i * subarray_size;
            let sub_end = sub_start + subarray_size;

            if let NumpyType::Int32(data) = &self.dtype {
                let mut sub_vec: Vec<i32> = Vec::new();
                for j in sub_start..sub_end {
                    sub_vec.push(data[j]);
                }
                nested_vec.push(sub_vec);
            } 
            
            else {
                panic!("Data type mismatch: Expected Int32");
            }
        }

        nested_vec

        }
    

    fn to_nested_vec_any_dim_i32(&self, start: usize, dimensions: &[usize]) -> Vec<Box<dyn Any>> {
        if dimensions.is_empty() {
            panic!("Dimensions array is empty");
        }

        // Base case: 1 or 2 dimension(s)
        if dimensions.len() == 1 {
            if let NumpyType::Int32(data) = &self.dtype {
                // Here we should directly create and return Vec<i32>
                return vec![Box::new(data[start..start + dimensions[0]].to_vec()) as Box<dyn Any>];
            } else {
                panic!("Data type mismatch: Expected Int32");
            }
        }

        // The big one, Recursive case - more than one dimension
        let subarray_size: usize = dimensions[1..].iter().product();
        let mut nested_vec: Vec<Box<dyn Any>> = Vec::new();

        for i in 0..dimensions[0] {
            let sub_start = start + i * subarray_size;
            nested_vec.push(Box::new(self.to_nested_vec_any_dim_i32(sub_start, &dimensions[1..])));
        }

        nested_vec

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

fn print_nested_vec(nested_vec: &[Box<dyn Any>], depth: usize) {
    for item in nested_vec {
        if depth == 1 {
            if let Some(subarray) = item.downcast_ref::<Vec<i32>>() {
                println!("Vec<i32> at depth {}: {:?}", depth, subarray);
            } else {
                println!("Failed to downcast to Vec<i32> at depth {}", depth);
            }
        } else {
            if let Some(subarray) = item.downcast_ref::<Vec<Box<dyn Any>>>() {
                println!("Vec<Box<dyn Any>> at depth {}", depth);
                print_nested_vec(subarray, depth - 1);
            } else {
                println!("Failed to downcast to Vec<Box<dyn Any>> at depth {}", depth);
            }
        }
    }
}


fn main() -> io::Result<()> {

    let file_path = "data/array_5.npy";

    // Read the header and data from the .npy file
    let (header, data_start_pos) = read_numpy_header(file_path)?;
    let numpy_array = NumpyArray::new(header, &mut File::open(file_path)?, data_start_pos)?;

    // Check if the dtype is Int32, and call the to_nested_vec_any_dim_i32 function
    if let NumpyType::Int32(_) = numpy_array.dtype {
        let nested_vec = numpy_array.to_nested_vec_any_dim_i32(0, &numpy_array.shape);

        // Recursively print the nested vector
        print_nested_vec(&nested_vec, numpy_array.shape.len());

    } else {
        println!("The numpy array is not of type Int32.");
    }

    Ok(())

}
