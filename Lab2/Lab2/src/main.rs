


fn main() {
    let val1 = 0x1234;
    let val2 = 0xABCD;

    let result = calculate_checksum(val1, val2);

    println!("Value 1:  {:#06x}", val1);
    println!("Value 2:  {:#06x}", val2);
    println!("Checksum: {:#06x}", result);
}