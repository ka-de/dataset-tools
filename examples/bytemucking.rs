/// An example of using cast to transmute a `u32` to an `f32`
fn casting_example() {
    use bytemuck::cast;

    let x: u32 = 0x3f800000;
    let y: f32 = cast(x); // y == 1.0

    assert_eq!(y, 1.0);
    println!("x: {} y: {}", x, y);
}

/// Casting a reference to a different type
fn cast_ref_example() {
    use bytemuck::{ cast_ref, Pod, Zeroable };

    #[derive(Copy, Clone, Pod, Zeroable)]
    #[repr(C)]
    struct Point {
        x: f32,
        y: f32,
    }

    let point = Point { x: 1.0, y: 2.0 };
    let point_ref: &Point = &point;
    let raw_bytes: &[u8; std::mem::size_of::<Point>()] = cast_ref(point_ref);
    assert_eq!(raw_bytes, &[0, 0, 128, 63, 0, 0, 0, 64]);
    println!("Raw bytes: {:?}", raw_bytes);
}

/// Casting a mutable slice
fn cast_slice_mut_example() {
    use bytemuck::{ cast_slice_mut, Pod, Zeroable };

    #[derive(Copy, Clone, Pod, Zeroable, Debug, PartialEq)]
    #[repr(C)]
    struct Color {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    }

    let mut colors = [
        Color { r: 255, g: 0, b: 0, a: 255 },
        Color { r: 0, g: 255, b: 0, a: 255 },
    ];
    println!("Original colors: {:?}", colors);

    let color_slice: &mut [Color] = &mut colors;
    let bytes_slice: &mut [u8] = cast_slice_mut(color_slice);
    // Modify the color values through the byte slice
    bytes_slice[0] = 128; // Change the red component of the first color

    assert_eq!(colors, [
        Color { r: 128, g: 0, b: 0, a: 255 },
        Color { r: 0, g: 255, b: 0, a: 255 },
    ]);
    println!("Modified colors: {:?}", colors);
}

/// Using try_cast for fallible casting
fn try_cast_example() {
    use bytemuck::{ try_cast, Pod, Zeroable };

    #[derive(Copy, Clone, Pod, Zeroable, Debug)]
    #[repr(C)]
    struct Vertex {
        position: [f32; 3],
        tex_coords: [f32; 2],
    }

    let vertex_data = [0.0f32; 5]; // Some vertex data
    match try_cast::<[f32; 5], Vertex>(vertex_data) {
        Ok(vertex) => println!("Vertex: {:?}", vertex),
        Err(e) => eprintln!("Failed to cast: {:?}", e),
    }
}

fn main() {
    casting_example();
    cast_ref_example();
    cast_slice_mut_example();
    try_cast_example();
}

#[cfg(test)]
mod tests {
    use bytemuck::{ cast, cast_ref, cast_slice_mut, try_cast, Pod, Zeroable };

    #[test]
    fn test_casting_example() {
        let x: u32 = 0x3f800000;
        let y: f32 = cast(x);
        assert_eq!(y, 1.0);
    }

    #[test]
    fn test_cast_ref_example() {
        #[derive(Copy, Clone, Pod, Zeroable)]
        #[repr(C)]
        struct Point {
            x: f32,
            y: f32,
        }

        let point = Point { x: 1.0, y: 2.0 };
        let point_ref: &Point = &point;
        let raw_bytes: &[u8; std::mem::size_of::<Point>()] = cast_ref(point_ref);
        assert_eq!(raw_bytes, &[0, 0, 128, 63, 0, 0, 0, 64]);
    }

    #[test]
    fn test_cast_slice_mut_example() {
        #[derive(Copy, Clone, Pod, Zeroable, Debug, PartialEq)]
        #[repr(C)]
        struct Color {
            r: u8,
            g: u8,
            b: u8,
            a: u8,
        }

        let mut colors = [
            Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
            Color {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            },
        ];

        let color_slice: &mut [Color] = &mut colors;
        let bytes_slice: &mut [u8] = cast_slice_mut(color_slice);
        bytes_slice[0] = 128;

        assert_eq!(colors, [
            Color {
                r: 128,
                g: 0,
                b: 0,
                a: 255,
            },
            Color {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            },
        ]);
    }

    #[test]
    fn test_try_cast_example() {
        #[derive(Copy, Clone, Pod, Zeroable, Debug, PartialEq)]
        #[repr(C)]
        struct Vertex {
            position: [f32; 3],
            tex_coords: [f32; 2],
        }

        let vertex_data = [0.0f32; 5];
        match try_cast::<[f32; 5], Vertex>(vertex_data) {
            Ok(vertex) =>
                assert_eq!(vertex, Vertex {
                    position: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                }),
            Err(_) => panic!("Failed to cast"),
        }
    }
}
