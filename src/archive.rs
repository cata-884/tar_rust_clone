use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{self, Write, Seek, SeekFrom};
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::header;

fn append_file<W: Write>(writer: &mut W, path: &Path, root: &Path, verbose: bool) -> io::Result<()> {
    let relative_path = path.strip_prefix(root).unwrap_or(path);
    let path_str = relative_path.to_str().expect("Non-UTF8 path encountered");

    if path_str.is_empty() {
        return Ok(());
    }

    let metadata = std::fs::metadata(path)?;
    let size = metadata.len();

    if verbose {
        println!("Adding: {} ({} bytes)", path_str, size);
    }

    let header_bytes = header::create_header(path_str, size);
    writer.write_all(&header_bytes)?;

    let mut f = File::open(path)?;
    io::copy(&mut f, writer)?;

    let padding_needed = (header::BLOCK_SIZE - (size % header::BLOCK_SIZE)) % header::BLOCK_SIZE;
    if padding_needed > 0 {
        let zeros = [0u8; 512]; 
        writer.write_all(&zeros[0..padding_needed as usize])?;
    }

    Ok(())
}

fn finalize_archive<W: Write>(writer: &mut W) -> io::Result<()> {
    let zeros = [0u8; 1024]; 
    writer.write_all(&zeros)?;
    writer.flush()?;
    Ok(())
}

fn pack_recursive<W: Write>(writer: &mut W, curr_path: &Path, root_path: &Path, verbose: bool) -> io::Result<()> {
    if curr_path.is_dir() {
        for entry in std::fs::read_dir(curr_path)? {
            let entry = entry?;
            let path = entry.path();
            pack_recursive(writer, &path, root_path, verbose)?;
        }
    } else {
        append_file(writer, curr_path, root_path, verbose)?;
    }
    Ok(())
}

pub fn pack_create(src: &Path, out_filename: &str, compress: bool, verbose: bool) -> io::Result<()> {
    if verbose {
        println!("Creating archive: {}", out_filename);
    }
    
    let file = File::create(out_filename)?;

    if compress {
        if verbose { println!("Compression: GZIP enabled"); }
        let mut encoder = GzEncoder::new(file, Compression::default());
        pack_recursive(&mut encoder, src, src, verbose)?;
        finalize_archive(&mut encoder)?;
    } else {
        if verbose { println!("Format: Standard TAR"); }
        let mut writer = file;
        pack_recursive(&mut writer, src, src, verbose)?;
        finalize_archive(&mut writer)?;
    }

    Ok(())
}

pub fn pack_append(src: &Path, output: &str, verbose: bool) -> io::Result<()> {
    if output.ends_with(".gz") {
        return Err(io::Error::other("Cannot append to compressed (.gz) archives"));
    }

    if verbose {
        println!("Appending to archive: {}", output);
    }

    let mut file = OpenOptions::new().read(true).write(true).open(output)?;

    let len = file.metadata()?.len();
    if len < 1024 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Archive corrupted or too small to append"));
    }
    
    file.seek(SeekFrom::Start(len - 1024))?;

    pack_recursive(&mut file, src, src, verbose)?;

    finalize_archive(&mut file)?;

    Ok(())
}