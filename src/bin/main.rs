use server::http::Http;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut server = Http::new();
    server.listen("5500")?;
    Ok(())
}
