use std::path::Path;
use memmap2::Mmap;
use safetensors::SafeTensors;
use tokio::fs::File;
use tokio::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Specify the path to your SafeTensors file
    let file_path = Path::new(
        "E:\\projects\\yiff_toolkit\\compass_loras\\dth-v1e400\\dth-v1e400.safetensors"
    );

    // Open the file asynchronously
    let file = File::open(file_path).await?;

    // Convert the async file to a standard File
    let std_file = file.into_std().await;

    // Create a memory map of the file
    let mmap = unsafe { Mmap::map(&std_file)? };

    // Deserialize the SafeTensors file
    let tensors = SafeTensors::deserialize(&mmap).map_err(|e|
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    )?;

    // Print out all the blocks (tensors)
    println!("Tensors in the file:");
    for (name, tensor) in tensors.tensors() {
        println!("Tensor name: {}", name);
        println!("  Shape: {:?}", tensor.shape());
        println!("  Dtype: {:?}", tensor.dtype());
        println!("  Data size: {} bytes", tensor.data().len());
        println!();
    }

    Ok(())
}
