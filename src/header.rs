//schelete pentru gestionarea header-ului TAR(512 bytes)
pub const BLOCK_SIZE: u64 = 512;
//daca data = 64 si buffer-ul are 8 bytes 
// => scrim : 0000100\0 
fn write_oct(slice: &mut [u8], data: u64){
    let len = slice.len() -1;
    //formatam nr ca string octal cu zerouri in fata
    let s = format!("{:0width$o}", data, width=len);
    let bytes = s.as_bytes();
    //daca fisierul e prea mare, luam doar ultimele cifre
    let start = bytes.len().saturating_sub(len);
    //copiem bytes in slice
    slice[0..len].copy_from_slice(&bytes[start..]);
    slice[len] = 0;
}
pub fn create_header(path : &str, size : u64) -> [u8; 512] {
    //implementare header (nume, mode, size, checksum)
    let mut header = [0u8; 512];
    //copy name (pozitiile 0 - 100)
    let name_bytes = path.as_bytes();
    let name_len = name_bytes.len().min(99);
    header[0..name_len].copy_from_slice(&name_bytes[0..name_len]);
    //permisiuni(100 - 108)
    write_oct(&mut header[100..108], 0o644);
    //uid
    write_oct(&mut header[108..116], 0); //root sau 1000 pentru user generic

    //gid
    write_oct(&mut header[116..124], 0);
    //file size
    write_oct(&mut header[124..136], size);
    //mtime - data modificarii. punem 0 pentru simplitate sau putem lua systemTime
    write_oct(&mut header[136..148], 0);
    //typeflag - 0: fisier normal, 5: director
    header[156] = b'0';
    //semnatura "ustarr" + null pentru a fi recunoscut ca TAR modern
    header[257..263].copy_from_slice(b"ustar\0");
    //version
    header[263..265].copy_from_slice(b"00");
    //checksum
    for byte in header.iter_mut().take(156).skip(148) {
        *byte = b' ';
    }
    let mut sum:u32 = 0;
    for byte in &header{
        sum += *byte as u32;
    }
    let sum_str = format!("{:06o}\0", sum);
    header[148..148+sum_str.len()].copy_from_slice(sum_str.as_bytes());
    header
}