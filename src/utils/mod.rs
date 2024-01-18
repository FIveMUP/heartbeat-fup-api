pub const fn gcd(a: u8, b: u8) -> u8 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

pub const fn lcm(a: u8, b: u8) -> u8 {
    a / gcd(a, b) * b
}
