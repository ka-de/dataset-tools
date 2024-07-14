use safetensors::SafeTensors;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use std::path::Path;
use dataset_tools::open_safetensors;

#[tokio::main]
async fn main() -> Result<()> {
    // Specify the path to your SafeTensors file
    let file_path = Path::new(
        "E:\\projects\\yiff_toolkit\\compass_loras\\dth-v1e400\\dth-v1e400.safetensors"
    );

    // Open the SafeTensors file
    let tensors = open_safetensors(file_path).await?;

    /*
    // Print out all the blocks (tensors)
    println!("Tensors in the file:");
    for (name, tensor) in tensors.tensors() {
        println!("Tensor name: {}", name);
        println!("  Shape: {:?}", tensor.shape());
        println!("  Dtype: {:?}", tensor.dtype());
        println!("  Data size: {} bytes", tensor.data().len());
        println!();
    }
	*/

    for name in tensors.tensors() {
        println!("{name}");
    }

    Ok(())
}
