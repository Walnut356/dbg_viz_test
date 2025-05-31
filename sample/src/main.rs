#[derive(Debug)]
struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
enum CEnum {
    Val1,
    Val2,
    Val3,
}

#[derive(Debug)]
enum SumType {
    Bare,
    Tuple(u8),
    Struct { a: u8, b: i32 },
}

union UnionType {
    a: u32,
    b: [u8; 4],
}

fn main() {
    // ------------------------- basic numeric types ------------------------ //
    let u8_v: u8 = 1;
    let u16_v: u16 = 2;
    let u32_v: u32 = 3;
    let u64_v: u64 = 4;
    let u128_v: u128 = 5;
    let usize_v: usize = 6;

    let i8_v: i8 = -1;
    let i16_v: i16 = -2;
    let i32_v: i32 = -3;
    let i64_v: i64 = -4;
    let i128_v: i128 = -5;
    let isize_v: isize = -6;

    let f32_v: f32 = 0.5;
    let f64_v: f64 = 1.5;

    // ------------------------------- arrays ------------------------------- //

    let array_v: [u8; 7] = [0, 1, 2, 3, 255, 254, 243];
    let array2_v: [i32; 5] = [1, 2, 3, i32::MAX, i32::MIN];

    let slice_v = array_v.as_slice();

    // ------------------------------- tuples ------------------------------- //

    let tuple_1 = (1u8, 20.0f32);

    // ------------------------- user-defined types ------------------------- //

    let point= Point {x: 5.5, y: 10.25};

    let c_enum_1 = CEnum::Val1;
    let c_enum_2 = CEnum::Val2;
    let c_enum_3 = CEnum::Val3;

    let sum_1 = SumType::Bare;
    let sum_2 = SumType::Tuple(50);
    let sum_3 = SumType::Struct { a: 10, b: -50 };

    // ------------------------------ std types ----------------------------- //

    // -------------------------- refs and pointers ------------------------- //

    println!("done");
}
