Un utilitar simplu de arhivare (TAR) și compresie scris în **Rust**.

### Features

- **Pack**: Creează arhive `.tar` standard (compatibile cu GNU tar).
- **GZIP Compression**: Suportă compresie (`-c`) folosind crate-ul `flate2`.
- **Append**: Adaugă fișiere la o arhivă `.tar` existentă (`-r`).
- **Unpack**: Extrage arhive `.tar` sau `.tar.gz`.

### Quick Run

Ai nevoie de Rust instalat.

```bash
#compilare
cargo build --release

#creare arhiva
./target/release/tar_project pack ./src -v -f output.tar

#extragere arhiva
./target/release/tar_project unpack output.tar -v -f ./extracted_folder
```
