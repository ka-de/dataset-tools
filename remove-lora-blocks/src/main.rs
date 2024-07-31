//! # LoRA UNet Block Remover
//!
//! This program randomly removes two UNet blocks from a LoRA safetensors file and saves the result
//! as a new file. The new file name includes abbreviations of the removed blocks.
//!
//! ## Usage
//!
//! ```
//! cargo run -- path/to/your/lora_file.safetensors
//! ```
//!
//! ## Features
//!
//! - Reads metadata from safetensors files
//! - Randomly selects and removes one text encoder and one UNet blocks
//! - Generates a new filename with abbreviations of removed blocks
//!
//! ## Example
//!
//! Input file:  obra-compass-v1e400_frockpt1_th-3.55.safetensors
//! Output file: obra-compass-v1e400_frockpt1_th-3.55-te2tmel23sakpldw+luib71tb1a1to0lu.safetensors
//!
//! In this example, the program removed these two layers:
//! - lora_te2_text_model_encoder_layers_23_self_attn_k_proj.lora_down.weight
//! - lora_unet_input_blocks_7_1_transformer_blocks_1_attn1_to_out_0.lora_up.weight

use anyhow::{ Context, Result };
use env_logger::Env;
use log::{ info, warn, debug, error };
use memmap2::Mmap;
use rand::seq::SliceRandom;
use rand::Rng;
use safetensors::tensor::{ SafeTensors, TensorView };
use safetensors::SafeTensorError;
use std::collections::HashMap;
use std::path::Path;
use tokio;
use ndarray::Array;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize env_logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        error!("Incorrect number of arguments provided");
        anyhow::bail!("Usage: {} <path_to_safetensors>", args[0]);
    }

    let input_path = Path::new(&args[1]);
    info!("Processing file: {}", input_path.display());

    process_lora_file(input_path).await?;

    Ok(())
}

async fn process_lora_file(input_path: &Path) -> Result<()> {
    debug!("Opening file: {}", input_path.display());
    let file = tokio::fs::File::open(input_path).await?;
    let mmap = unsafe { Mmap::map(&file)? };

    info!("Deserializing tensors");
    let tensors = SafeTensors::deserialize(&mmap)?;
    let (header_size, metadata) = SafeTensors::read_metadata(&mmap)?;

    let mut rng = rand::thread_rng();
    let keys: Vec<String> = tensors
        .tensors()
        .filter(|(key, _)| (key.starts_with("lora_unet") || key.starts_with("lora_te")))
        .map(|(key, _)| key.to_string())
        .collect();

    info!("Selecting tensors for modification");
    let selected_keys: Vec<String> = keys.choose_multiple(&mut rng, 2).cloned().collect();
    debug!("Selected keys: {:?}", selected_keys);

    info!("Modifying tensors");
    let new_tensors = scale_tensors(&tensors, &selected_keys)?;

    let new_filename = generate_new_filename(input_path, &selected_keys)?;
    let output_path = input_path.with_file_name(new_filename);

    info!("Writing new safetensors file: {}", output_path.display());
    let mut new_file = tokio::fs::File::create(&output_path).await?;
    safetensors::serialize_to_file(&new_tensors, &metadata, &mut new_file).await?;

    info!("Successfully created new file: {}", output_path.display());

    Ok(())
}

fn scale_tensors(
    tensors: &SafeTensors,
    keys_to_scale: &[String]
) -> Result<HashMap<String, TensorView>> {
    let mut new_tensors = HashMap::new();
    let mut rng = rand::thread_rng();

    for (key, tensor) in tensors.tensors() {
        if keys_to_scale.contains(&key.to_string()) {
            debug!("Scaling tensor: {}", key);
            let scale_factor = rng.gen_range(0.5..1.5);
            let data = tensor.data();
            let shape = tensor.shape();

            let arr = match tensor.dtype() {
                safetensors::Dtype::F32 => {
                    let mut arr = Array::from_shape_vec(shape, data.to_vec())?.mapv(|x|
                        f32::from_le_bytes(x.try_into().unwrap())
                    );
                    arr.mapv_inplace(|x| x * scale_factor);
                    arr.mapv(|x| x.to_le_bytes().to_vec()).into_raw_vec()
                }
                // Add other data types as needed
                _ => {
                    warn!("Unsupported dtype for tensor: {}. Leaving unmodified.", key);
                    data.to_vec()
                }
            };

            new_tensors.insert(
                key.to_string(),
                TensorView::new(tensor.dtype(), shape.to_vec(), &arr)?
            );
        } else {
            new_tensors.insert(key.to_string(), tensor);
        }
    }

    Ok(new_tensors)
}

fn generate_new_filename(input_path: &Path, scaled_keys: &[String]) -> Result<String> {
    let original_name = input_path
        .file_stem()
        .context("Invalid file name")?
        .to_str()
        .context("Invalid UTF-8 in file name")?;

    let abbreviations: String = scaled_keys
        .iter()
        .map(|key| abbreviate_key(key))
        .collect::<Result<Vec<String>>>()?
        .join("+");

    Ok(format!("{}-scaled-{}.safetensors", original_name, abbreviations))
}

fn abbreviate_key(key: &str) -> Result<String> {
    let parts: Vec<&str> = key.split(&['_', '.']).collect();
    let mut abbrev = String::new();

    for (i, part) in parts.iter().enumerate() {
        if *part == "te1" || *part == "te2" {
            abbrev.push_str(part);
        } else if
            i == 0 ||
            (i == 1 && (parts[0] == "lora" || parts[0] == "te1" || parts[0] == "te2"))
        {
            continue;
        } else if part.parse::<usize>().is_ok() {
            abbrev.push_str(part);
        } else {
            abbrev.push(part.chars().next().context("Empty part in key")?);
        }
    }

    Ok(abbrev)
}
