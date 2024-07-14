use safetensors::SafeTensors;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Specify the path to your SafeTensors file
    let file_path =
        Path::new("E:\\projects\\yiff_toolkit\\compass_loras\\dth-v1e400\\dth-v1e400.safetensors");

    // Read the file into a buffer
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Deserialize the SafeTensors file
    let tensors = SafeTensors::deserialize(&buffer)?;

    // Print out all the blocks (tensors)
    //println!("Tensors in the file:");
    for (name, tensor) in tensors.tensors() {
        println!("{name}");
        //println!("Tensor name: {}", name);
        //println!("  Shape: {:?}", tensor.shape());
        //println!("  Dtype: {:?}", tensor.dtype());
        //println!("  Data size: {} bytes", tensor.data().len());
        //println!();
    }

    Ok(())
}
