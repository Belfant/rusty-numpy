# Rusty Numpy: N-dimensional Array Parser

## Overview
This Rust library is an in-development project for reading and parsing NumPy array files (.npy format).  It focuses on converting NumPy arrays into native Rust vectors with specific data types, offering an intuitive and efficient way to handle NumPy data in Rust applications.

## Features
- Reads .npy files and converts them into native Rust vectors.
- Supports specific NumPy data types such as Int32 and Float64.
- Capable of parsing 1D, 2D, 3D, and n-dimensional arrays into corresponding Rust data structures.
- Handles little-endian encoded data.
- Currently supports a subset of NumPy types, with plans to expand.

## Usage (Example)
Here's how you might use the library in its current form:

```rust
use rusty_numpy::{process_numpy_file, NumpyArrayResult};

fn main() -> std::io::Result<()> {

    let file_path = "path/to/your/numpy/file.npy";

    match process_numpy_file(file_path)? {
        NumpyArrayResult::Vec1D(array) => println!("1D Array: {:?}", array),
        NumpyArrayResult::Vec2D(array) => println!("2D Array: {:?}", array),
        // Other cases
    }
    Ok(())
}
```

## Project Status
This library is a personal project, developed primarily during my spare time and for my own enjoyment. As such, it is in a state of active development. Users should be aware that both the API and the feature set are subject to change as the project evolves

### Getting started: 
- clone the repository to your local machine.
- build the library: Cargo build inside the project directory
