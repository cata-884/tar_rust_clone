//responsabil pentru unpack

use std::path::Path;
use std::io;
use std::io::{Read};
use std::fs::{self, File};
use flate2::read::GzDecoder;
use crate::header::BLOCK_SIZE;
//argumente: archive_path - calea catre fisier de tip .tar sau .tar.gz
//b8 -> b10 : "0000100\0" -> 64
fn parse_octal(slice : &[u8]) ->u64{
    //bytes to string, scapand de caracterele invalide
    let s = String::from_utf8_lossy(slice);
    //skip la null terminator si spatiu
    let trimmed = s.trim_matches(|c: char| c == '\0' || c.is_whitespace());
    //converim in b8
    u64::from_str_radix(trimmed, 8).unwrap_or(0)
}
//parse name byte by byte si apoi taiem \0
fn parse_name(slice: &[u8])->String {
    String::from_utf8_lossy(slice).trim_matches('\0').to_string()
}

fn unpack_stream<R: Read>(mut reader : R, destination: &Path, verbose: bool) -> io::Result<()>{
    let mut header_buf = [0u8; BLOCK_SIZE as usize];
    //daca nu avem un folder destinatie
    if !destination.exists() && destination != Path::new(""){
        fs::create_dir_all(destination)?;
    }
    loop {
        //citim header-ul
        let bytes_read = reader.read(&mut header_buf)?;
        if bytes_read == 0{
            break;
        }
        //verificam daca am gasit un bloc gol
        if header_buf.iter().all(|&b| b == 0){
            continue;
        }
        let name = parse_name(&header_buf[0..100]);
        let size = parse_octal(&header_buf[124..136]);
        let type_flag = header_buf[156];

        if name.is_empty() {
            continue;
        }
        println!("extragere pentru {name}, size - {size}");
        //target path
        let path = destination.join(name.clone());
        if verbose {
            println!("Extragere: {} -> {:?}", name, path);
        }
        //director in standard tar
        if type_flag == b'5' || name.ends_with('/'){
            fs::create_dir_all(path)?;
        }
        else{
            //fisier normal
            
            //daca director parinte exista
            if let Some(parent) = path.parent() && !parent.exists(){
                fs::create_dir_all(parent)?;
            }
            let mut out_file = File::create(&path)?;
            let mut limiter = (&mut reader).take(size);
            //citim din stream exact 'size' bytes pe care ii punem in out_file
            io::copy(&mut limiter, &mut out_file)?;
            let padding = (512 - (size % 512)) % 512;
            if padding > 0 {
                let mut skip_buf = [0u8; 512];
                reader.read_exact(&mut skip_buf[0..padding as usize])?;
            }
        }
    }

    Ok(())
}
pub fn unpack_archive(path : &Path, destination: &Path, verbose: bool) -> io::Result<()>{
    println!("path - {:?}", path);
    let file = File::open(path)?;
    let is_gzip = path.extension().is_some_and(|ext| ext == "gz");
    if is_gzip {
        if verbose {println!("format gzip. init decoder");}
        unpack_stream(GzDecoder::new(file), destination, verbose)?;
    }
    else{
        if verbose {println!("este tar. citim direct"); }
        unpack_stream(file, destination, verbose)?;
    }
    Ok(())
}