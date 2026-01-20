use std::path::Path;
use std::io::{self, Read};
use std::fs::{self, File};
use flate2::read::GzDecoder;

use crate::header;

fn parse_octal(slice: &[u8]) -> u64 {
    let s = String::from_utf8_lossy(slice);
    
    let trimmed = s.trim_matches(|c: char| c == '\0' || c.is_whitespace());
    
    u64::from_str_radix(trimmed, 8).unwrap_or(0)
}

fn parse_name(slice: &[u8]) -> String {
    String::from_utf8_lossy(slice)
        .trim_matches('\0')
        .to_string()
}

fn unpack_stream<R: Read>(mut reader: R, destination: &Path, verbose: bool) -> io::Result<()> {
    let mut header_buf = [0u8; header::BLOCK_SIZE as usize];

    if !destination.exists() && destination != Path::new("") {
        fs::create_dir_all(destination)?;
    }

    loop {
        let bytes_read = reader.read(&mut header_buf)?;
        if bytes_read == 0 {
            break; 
        }

        if header_buf.iter().all(|&b| b == 0) {
            continue;
        }

        let name = parse_name(&header_buf[0..100]);
        let size = parse_octal(&header_buf[124..136]);
        let type_flag = header_buf[156];

        if name.is_empty() {
            continue;
        }

        let target_path = destination.join(&name);

        if verbose {
            println!("Extracting: {} ({} bytes) -> {:?}", name, size, target_path);
        }

        if type_flag == b'5' || name.ends_with('/') {
            fs::create_dir_all(&target_path)?;
        } else {
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            let mut out_file = File::create(&target_path)?;
            
            let mut limiter = (&mut reader).take(size);
            io::copy(&mut limiter, &mut out_file)?;

            let padding_needed = (header::BLOCK_SIZE - (size % header::BLOCK_SIZE)) % header::BLOCK_SIZE;
            if padding_needed > 0 {
                let mut skip_buf = [0u8; header::BLOCK_SIZE as usize];
                reader.read_exact(&mut skip_buf[0..padding_needed as usize])?;
            }
        }
    }

    Ok(())
}

pub fn unpack_archive(path: &Path, destination: &Path, verbose: bool) -> io::Result<()> {
    if verbose {
        println!("Opening archive: {:?}", path);
    }

    let file = File::open(path)?;

    let is_gzip = path.extension().map_or(false, |ext| ext == "gz");

    if is_gzip {
        if verbose { println!("Format: GZIP detected. Initializing decoder..."); }
        unpack_stream(GzDecoder::new(file), destination, verbose)?;
    } else {
        if verbose { println!("Format: TAR (Uncompressed)"); }
        unpack_stream(file, destination, verbose)?;
    }

    Ok(())
}