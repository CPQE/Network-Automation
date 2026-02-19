use std::fs::File;
use std::io::{self, Read};

//incorporate from previous assignment 
pub fn read_key_file(path: &str) -> io::Result<Vec<u8>> {
    let bytes = std::fs::read(path)?;
    if bytes.len() < 8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Key file must contain at least 8 bytes",
        ));
    }
    Ok(bytes[..8].to_vec())
}


//sender functions
pub fn read_file_bytes(path: &str) -> io::Result<Vec<u8>> {
    //this is for the sender to read payload file, convert it into raw bytes, pad if needed, and then calculate checksum
    //then assembling final UDP datagram and writing to an output file. 
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn write_datagram_to_file(path: &str, data: &[u8]) -> io::Result<()> {
    std::fs::write(path, data)
}
//receiver functions: 

//define read_datagram_bytes(){} here for receiver