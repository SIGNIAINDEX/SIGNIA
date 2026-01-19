pub fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_adds() {
        assert_eq!(add(2, 3), 5);
    }
}
