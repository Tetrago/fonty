use fonty::Ttf;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.is_empty() {
        return Err("Invalid arguments".into());
    }

    let mut ttf = Ttf::open(std::path::Path::new(&args[1]))?;
    println!("{:?}", ttf.glyph(0)?);

    Ok(())
}
