use safetensors::SafeTensors;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let stats_flag = args.contains(&String::from("--stats")) || args.contains(&String::from("-s"));

    // Check if file path is provided
    if args.len() < 2 {
        return Err(
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "No file path provided"))
        );
    }

    let file_path = Path::new(&args[1]);
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let tensors = SafeTensors::deserialize(&buffer)?;
    for (name, tensor) in tensors.tensors() {
        println!("{name}");
        if stats_flag {
            println!("Shape: {:?}", tensor.shape());
            println!("Dtype: {:?}", tensor.dtype());
            println!("Data size: {} bytes", tensor.data().len());
            println!();
        }
    }
    Ok(())
}
