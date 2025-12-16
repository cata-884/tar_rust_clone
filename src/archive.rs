//responsabil pentru operatia de pack

use std::path::Path;
use std::fs::{File, OpenOptions };
use std::io::{self, Write, Seek, SeekFrom};
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::header;
//argumente: src_path output_name
//W poate fi File, GzEncoder, etc
fn add_file_to_archive<W: Write> (writer : &mut W, path: &Path, root : &Path, verbose : bool) ->io::Result<()>{
    let relative_path = path.strip_prefix(root).unwrap_or(path);
    let path_str = relative_path.to_str().expect("Non-UTF8 path");
    //ignoram daca e gol
    if path_str.is_empty(){
        return Ok(());
    }
    //extragem detalii despre fisier fara a-l deschide
    let metadata = std::fs::metadata(path)?;
    let size = metadata.len();

    if verbose {
        println!("Adaugare: {}({} bytes)", path_str, size);
    }
    //cream si scrim header-ul
    let header_bytes = header::create_header(path_str, size);
    writer.write_all(&header_bytes)?;
    //copiem continutul fisierului in writer
    let mut f = File::open(path)?;
    io::copy(&mut f, writer)?;

    let padding_necesar = (512 - (size%512))%512;
    if padding_necesar > 0 {
        let zerouri = [0u8; 512];
        writer.write_all(&zerouri[0..padding_necesar as usize])?;
    }
    Ok(())
}
//tar specifica ca arhiva se termina cu 2 blocuri goale de 512 bytes
fn terminare_arhiva<W: Write> (writer: &mut W)->io::Result<()>{
    let zeros = [0u8; 1024];
    writer.write_all(&zeros)?;
    //este totul scris pe disc?
    writer.flush()?;
    Ok(())
}

fn archive_recursive<W: Write> (writer : &mut W, curr_path: &Path, root_path: &Path, verbose : bool) -> io::Result<()>{
    //daca e dir
    if curr_path.is_dir(){
        for it in std::fs::read_dir(curr_path)?{
            let it = it?;
            let path = it.path();
            archive_recursive(writer, &path, root_path, verbose)?;
        }
    }
    //daca e fisier
    else {
        add_file_to_archive(writer, curr_path, root_path, verbose)?;
    }
    Ok(())
}
//creaza 
pub fn pack_create(src: &Path, out_filename: &str, compress: bool , verbose : bool) -> io::Result<()>{
    println!("Creare arhivă: {}", out_filename);
    let file = File::create(out_filename)?;

    if compress {
        if verbose { println!("Compresie GZIP activată"); }
        let mut encoder = GzEncoder::new(file, Compression::default());
        archive_recursive(&mut encoder, src, src, verbose)?;
        terminare_arhiva(&mut encoder)?;
    }
    else {
        if verbose { println!("Format TAR standard"); }
        let mut writer = file;
        archive_recursive(&mut writer, src, src, verbose)?;
        terminare_arhiva(&mut writer)?;
    }
    Ok(())  
}
//-f
pub fn pack_append(src: &Path, output: &str, verbose: bool) -> io::Result<()> {
    if output.ends_with(".gz") {
        return Err(io::Error::other( "Nu se poate face append (-r) pe fisiere comprimate"));
    }

    println!("Adaugare la arhiva: {}", output);
    let mut file = OpenOptions::new().read(true).write(true).open(output)?;

    // Ne ducem la final - 1024 bytes (suprascriem markerul de final vechi)
    let len = file.metadata()?.len();
    if len < 1024 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Arhivă coruptă sau prea mică"));
    }
    file.seek(SeekFrom::Start(len - 1024))?;

    // Adăugăm noile fișiere
    archive_recursive(&mut file, src, src, verbose)?;
    
    // Scriem noul marker de final
    terminare_arhiva(&mut file)?;

    Ok(())
}