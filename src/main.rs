use rusty_numpy::process_numpy_file;
use std::io;

fn main() -> io::Result<()> {

    let file_path = "data/array_6.npy";
    let _result = process_numpy_file(file_path)?;

    Ok(())

}
