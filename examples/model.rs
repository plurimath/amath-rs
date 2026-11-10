#[cfg(feature = "models")]
fn main() {
    let parser = amath::Parser::new().unwrap();
    let model = parser
        .parse_model(&std::env::args().nth(1).unwrap_or("alpha+1".into()), ".")
        .unwrap();
    println!("{}", amath::typed::to_portable_json(&model).unwrap());
}
#[cfg(not(feature = "models"))]
fn main() {
    eprintln!("enable --features models");
    std::process::exit(1)
}
