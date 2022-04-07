extern "C" {
    fn cadd(a: u32, b: u32) -> u32;
}

pub fn wcadd(a: u32, b: u32) -> u32 {
    let c = unsafe { cadd(a, b) };
    return c;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
