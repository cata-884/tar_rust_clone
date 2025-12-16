mod header;
mod archive;
mod extract;

use std::env;
use std::path::{Path, PathBuf};
use std::process;

fn print_usage() {
    println!("Usage:");
    println!("  rtar pack <path> [-c] [-v] [-r] [-f output_file]");
    println!("  rtar unpack <archive> [-v] [-f destination_folder]");
}

fn main() {
    let args : Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    let command = &args[1];
    match command.as_str(){
        "pack" => {
            if args.len() < 3 {
                eprint!("Error: source directory lipsa");
                print_usage();
                process::exit(1);
            }
            let src = Path::new(&args[2]);
            //default
            let mut compress = false;
            let mut verbose = false;
            let mut append = false;
            let mut output = "output.tar".to_string();
            let mut i = 3;
            while i< args.len(){
                match args[i].as_str(){
                    "-c" => compress = true,
                    "-v" =>verbose = true,
                    "-r" => append = true,
                    "-f" => {
                        if i+1 < args.len(){
                            output = args[i+1].clone();
                            i+=1;
                        }
                        else{
                            eprintln!("nu avem fisier dupa -f");
                            process::exit(1);
                        }
                    },
                    //daca nu primesc explicit -f dar primesc o locatie de output 
                    //ex : 
                    _ => {
                        if output == "output.tar" && !args[i].starts_with('-') {
                            output = args[i].clone();
                        }
                    }
                }
                i+=1;
            }
            if compress && !output.ends_with(".gz"){
                output.push_str(".gz");
            }
            let res = if append {
                if compress {
                    eprintln!("nu pot folosi -r(append) si -c(compress)");
                    process::exit(1);
                }
                archive::pack_append(src, &output, verbose)
            }
            else{
                archive::pack_create(src, &output, compress, verbose)
            };

            if let Err(e) = res {
                eprintln!("eroare packing - {e}");
                process::exit(1);
            }
            else if verbose{
                println!("Gata");
            }
        },
        "unpack" => {
            if args.len() < 3 {
                eprint!("Error: fisier arhiva lipsa");
                print_usage();
                process::exit(1);
            }
            let file = Path::new(&args[2]);
            let mut verbose = false;
            let mut dest_folder = PathBuf::from("");
            let mut i = 3;
            while i< args.len(){
                match args[i].as_str(){
                    "-v" => verbose = true,
                    "-f" => {
                        if i+1 < args.len(){
                            dest_folder = PathBuf::from(&args[i+1]);
                            i+=1;
                        }
                        else{
                            eprintln!("nu s-a dat destinatie fisier dupa -f");
                        }
                    },
                    _ => {}
                }
                i+=1;
            }
            if let Err(e) = extract::unpack_archive(file, &dest_folder, verbose){
                eprintln!("Eroare unpacking - {e}");
                process::exit(1);
            }
            else if verbose{
                println!("Gata");
            }
        },
        "help" | "--help" | "-h" => {
            print_usage();
        },
        _ => {
            eprintln!("Comanda necunoscuta: {}", command);
            print_usage();
            process::exit(1);
        },
        
    }
}