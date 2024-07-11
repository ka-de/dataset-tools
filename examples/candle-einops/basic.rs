use candle_core::{ Device, Result, Tensor };
use candle_einops::einops;

// Creates a 3D tensor with random values, reshapes it by rearranging its dimensions,
// and then prints the input and output tensors.
// The reshaping operation effectively transposes the tensor.
fn transpose() -> Result<()> {
    // Tensor Creation
    // ---
    // The `Tensor::randn` function creates a tensor filled with
    //random values.
    // The tensor has a shape of (10, 3, 32), meaning it has 10
    // height slices, each with 3 width slices, each of which has
    // 32 channels.
    // The tensor is created on the CPU.
    let input = Tensor::randn(0.0f32, 1.0, (10, 3, 32), &Device::Cpu)?;

    // Tensor Reshaping
    // ---
    // The `einops` macro is used to reshape the tensor. The operation
    // "h w c -> c h w" means that the height, width, and channel
    // dimensions of the input tensor are rearranged to become the channel,
    // height, and width dimensions of the output tensor, respectively.
    let output = einops!("h w c -> c h w", &input);

    println!("Input:\n{input}");
    // Transpose
    // Permute/Transpose dimensions, left side of -> is the original state, right of -> describes the end state
    // (10, 3, 32) -> (32, 10, 3)
    println!("Output:\n{output}");

    Ok(())
}

// Creates a 4D tensor with random values, combines the batch and height dimensions to form a new dimension,
// and then prints the input and output tensors.
// The reshaping operation effectively flattens the batch and height dimensions into one.
fn composition() -> Result<()> {
    // Tensor Creation
    // ---
    // The `Tensor::randn` function is called to create a tensor with random values.
    // The tensor has a shape of (10, 28, 28, 3), meaning it has 10 batches, each with
    // 28 height slices, each of which has 28 width slices and 3 channels.
    // The tensor is created on the CPU device.
    let input: Tensor = Tensor::randn(0.0f32, 1.0, (10, 28, 28, 3), &Device::Cpu)?;
    // Composition
    // Combine dimensions by putting them inside a parenthesis on the right of ->
    // (10, 28, 28, 3) -> (280, 28, 3)
    // The einops macro is used to compose the tensor.
    // The operation "b h w c -> (b h) w c" means that the batch and height dimensions
    // of the input tensor are combined to form a new dimension in the output tensor.
    // The width and channel dimensions remain the same.
    // So, the shape of the output tensor will be (280, 28, 3).
    let output = einops!("b h w c -> (b h) w c", &input);

    println!("Input:\n{input}");
    println!("Output:\n{output}");

    Ok(())
}

// Creates a 4D tensor with random values, transposes the height dimension to the first dimension
// and combines the batch and width dimensions to form a new dimension, and then prints the input
// and output tensors.
// The reshaping operation effectively flattens the batch and width dimensions into one and
// transposes the height dimension to the first dimension
fn transpose_and_composition() -> Result<()> {
    // Tensor Creation
    // ---
    // The `Tensor::randn` function is called to create a tensor with random values.
    // The tensor has a shape of (10, 28, 28, 3), meaning it has 10 batches, each with
    // 28 height slices, each of which has 28 width slices and 3 channels.
    // The tensor is created on the CPU device.
    let input: Tensor = Tensor::randn(0.0f32, 1.0, (10, 28, 28, 3), &Device::Cpu)?;
    // Transpose + Composition
    // ---
    // Transpose a tensor, followed by composing two dimensions into one, in one single expression.
    // (10, 28, 28, 3) -> (28, 280, 3)
    let output = einops!("b h w c -> h (b w) c", &input);

    println!("Input:\n{input}");
    println!("Output:\n{output}");

    Ok(())
}

// This function demonstrates tensor decomposition, which is the
// process of splitting a dimension into two.
fn decomposition() -> Result<()> {
    // Tensor Creation
    // ---
    // The `Tensor::randn` function is called to create a tensor with random values.
    // The tensor has a shape of (10, 28, 28, 3), meaning it has 10 batches, each with
    // 28 height slices, each of which has 28 width slices and 3 channels.
    // The tensor is created on the CPU device.
    let input: Tensor = Tensor::randn(0.0f32, 1.0, (10, 28, 28, 3), &Device::Cpu)?;
    // Decomposition
    // ---
    // Split a dimension into two, by specifying the details inside parenthesis on the left,
    // specify the shape of the new dimensions like so b1:2, b1 is a new dimension with shape 2.
    // (10, 28, 28, 3) -> (2, 5, 28, 28, 3)
    //let output = einops!("(b1:2 b2) h w c -> b1 b2 h w c", &input);

    // New axis can also be specified from variables or fields (struct and enum) using curly braces
    let b1 = 2;
    let output = einops!("({b1} b2) h w c -> {b1} b2 h w c", &input);

    println!("Input:\n{input}");
    println!("Output:\n{output}");

    Ok(())
}

// This function demonstrates how to perform all the operations
// discussed so far in a single expression.
fn decomposition_transpose_composition() -> Result<()> {
    // Tensor Creation
    // ---
    // The `Tensor::randn` function is called to create a tensor with random values.
    // The tensor has a shape of (10, 28, 28, 3), meaning it has 10 batches, each with
    // 28 height slices, each of which has 28 width slices and 3 channels.
    // The tensor is created on the CPU device.
    let input: Tensor = Tensor::randn(0.0f32, 1.0, (10, 28, 28, 3), &Device::Cpu)?;
    // The einops macro is used to decompose, transpose, and
    // compose the tensor. The operation
    // "b h (w w2:2) c -> (h w2) (b w) c" means that the width
    // dimension of the input tensor is split into two new
    // dimensions, one of which is transposed to the first
    // dimension in the output tensor.
    // The batch and the other half of the width dimension are
    // combined to form a new dimension.
    // The channel dimension remains the same. So, the shape of the output tensor will be (56, 140, 3).
    // (10, 28, 28, 3) -> (56, 140, 3)
    let output = einops!("b h (w w2:2) c -> (h w2) (b w) c", &input);

    println!("Input:\n{input}");
    println!("Output:\n{output}");

    Ok(())
}

fn main() {
    //let _ = transpose();
    //let _ = composition();
    //let _ = transpose_and_composition();
    //let _ = decomposition();
    //let _ = decomposition_transpose_composition();
}
