fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| "x+1".into());
    match amath::parse(&input) {
        Ok(parsed) => println!("{}", parsed.canonical()),
        Err(error) => {
            eprintln!("{error} (byte offset {:?})", error.byte_offset());
            std::process::exit(1);
        }
    }
}
