use std::fs::File;
use std::io::{Seek, SeekFrom, Read, ErrorKind, Cursor, Write};
use std::env::args;
use inflate::inflate_bytes_zlib;

pub fn read_anvil_header(fd: &mut File) -> Result<[(usize, usize); 1024], std::io::Error> {
    let mut result = [(0usize, 0usize); 1024];
    let mut buffer = [0u8; 4096];
    let mut numbuf = [0u8; 4];
    fd.read(&mut buffer)?;

    let mut c = Cursor::new(buffer.to_vec());
    for i in 0..1024 {
        c.read_exact(&mut numbuf)?;
        let entry = u32::from_be_bytes(numbuf);
        let offset = entry >> 8;
        let offset = (offset * 4096) as usize;

        let size = entry & 0xFF;
        let size = (size * 4096) as usize;

        result[i] = (offset, size);
    }

    Ok(result)
}


pub fn decompress_chunk(data: &mut Vec<u8>, scheme: u8) -> Result<Vec<u8>, std::io::Error> {
    match scheme {
        0 | 3 => { // No compression
            Ok(data.clone())
        },
        2 => { // Deflate(zlib)
            return if let Ok(decoded_data) = inflate_bytes_zlib(&*data) {
                Ok(decoded_data)
            } else {
                Err(std::io::Error::new(
                    ErrorKind::Other,
                    "Failed to decompress"
                ))
            }
        },
        _ => Err(std::io::Error::new(
            ErrorKind::Other,
            "Convert to string failed"
        ))
    }
}
pub fn coord_normalizer(region_x: i64, region_z: i64, chunk: usize) -> (i64, i64) {
    let chunk = chunk as i64;
    let chunk_x = chunk & 0b11111;
    let chunk_z = (chunk >> 5) & 0b11111;

    let mut region_x = region_x * 32;
    let mut region_z = region_z * 32;

    region_x += chunk_x;
    region_z += chunk_z;

    return (region_x, region_z);
}


fn index_mca(start: String, target: String) -> std::io::Result<()> {
    // This function just indexes all functions in the mca and declares them available, to be later verified by verify_available
    let mut file = File::open(start)?;

    let chunks_in_region = read_anvil_header(&mut file)?;
    for i in 0..1024 {
        let coords = coord_normalizer(0, 0, i);
        let (offset, size) = chunks_in_region[i];
        file.seek(SeekFrom::Start(offset as u64))?;
        if size != 0 {
            let mut sector_buffer = vec![0u8; size];
            file.read(&mut sector_buffer)?;

            let mut compressed_size_buffer = [0u8; 4];
            let mut compression_type_buffer = [0u8];
            let mut cursor = Cursor::new(sector_buffer);
            cursor.read_exact(&mut compressed_size_buffer)?;
            cursor.read_exact(&mut compression_type_buffer)?;
            let compressed_size = i32::from_be_bytes(compressed_size_buffer) as usize;
            let compression_scheme = compression_type_buffer[0];

            let mut compressed_data_buffer = vec![0u8; compressed_size];
            cursor.read_exact(&mut compressed_data_buffer)?;

            let data = decompress_chunk(
                &mut compressed_data_buffer,
                compression_scheme
            )?;

            let mut write_fd = File::create(format!("{}/{}.{}.nbt", target, coords.0, coords.1))?;
            write_fd.write(&*data)?;

            drop(write_fd);
        }
        //fd.seek(SeekFrom::Start(offset as u64));
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args : Vec<String> = args().collect();
    if args.len() < 3 {
        panic!("Not enough parameters. Usage: program <input.mca> <outputdir>");
    }

    index_mca(args[1].clone(), args[2].clone())?;
    Ok(())
}
