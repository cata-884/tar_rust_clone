mod archive;
mod extract;
mod header;

use std::env;
use std::path::PathBuf;
use std::process;

enum Mode {
    Pack {
        src: PathBuf,
        output: String,
        compress: bool,
        verbose: bool,
        append: bool,
    },
    Unpack {
        archive: PathBuf,
        dest: PathBuf,
        verbose: bool,
    },
    Help,
}

struct Config {
    mode: Mode,
}

impl Config {
    fn new(mut args: env::Args) -> Result<Config, String> {
        args.next();

        let command = match args.next() {
            Some(cmd) => cmd,
            None => return Ok(Config { mode: Mode::Help }),
        };

        match command.as_str() {
            "pack" => parse_pack(args),
            "unpack" => parse_unpack(args),
            "help" | "--help" | "-h" => Ok(Config { mode: Mode::Help }),
            _ => Err(format!("Unknown command: '{}'", command)),
        }
    }
}

fn parse_pack(mut args: env::Args) -> Result<Config, String> {
    let src_str = args.next().ok_or("Missing source directory")?;
    let src = PathBuf::from(src_str);

    let mut output = String::from("output.tar");
    let mut compress = false;
    let mut verbose = false;
    let mut append = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-c" => compress = true,
            "-v" => verbose = true,
            "-r" => append = true,
            "-f" => {
                output = args.next().ok_or("Missing filename after -f")?;
            }
            val if !val.starts_with('-') && output == "output.tar" => {
                output = val.to_string();
            }
            _ => return Err(format!("Unknown option: {}", arg)),
        }
    }

    if compress && append {
        return Err(String::from("Cannot use append (-r) with compression (-c)"));
    }

    if compress && !output.ends_with(".gz") {
        output.push_str(".gz");
    }

    Ok(Config {
        mode: Mode::Pack {
            src,
            output,
            compress,
            verbose,
            append,
        },
    })
}

fn parse_unpack(mut args: env::Args) -> Result<Config, String> {
    let archive_str = args.next().ok_or("Missing archive file")?;
    let archive = PathBuf::from(archive_str);

    let mut dest = PathBuf::from(""); 
    let mut verbose = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-v" => verbose = true,
            "-f" => {
                let d = args.next().ok_or("Missing destination path after -f")?;
                dest = PathBuf::from(d);
            }
            _ => return Err(format!("Unknown option: {}", arg)),
        }
    }

    Ok(Config {
        mode: Mode::Unpack {
            archive,
            dest,
            verbose,
        },
    })
}

fn print_usage() {
    println!("Usage:");
    println!("  rtar pack <source_dir> [-c] [-v] [-r] [-f output_file]");
    println!("  rtar unpack <archive_file> [-v] [-f destination_folder]");
    println!("\nOptions:");
    println!("  -c  Compress using GZIP");
    println!("  -v  Verbose output");
    println!("  -r  Append to existing archive");
    println!("  -f  Specify output file or destination");
}

fn main() {
    let config = match Config::new(env::args()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            print_usage();
            process::exit(1);
        }
    };

    let result = match config.mode {
        Mode::Pack {
            src,
            output,
            compress,
            verbose,
            append,
        } => {
            if append {
                archive::pack_append(&src, &output, verbose)
            } else {
                archive::pack_create(&src, &output, compress, verbose)
            }
        }
        Mode::Unpack {
            archive,
            dest,
            verbose,
        } => extract::unpack_archive(&archive, &dest, verbose),
        Mode::Help => {
            print_usage();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Operation failed: {}", e);
        process::exit(1);
    } 
}
