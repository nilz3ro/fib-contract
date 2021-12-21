#![no_std]

pub struct Fibonacci {
    prev: u64,
    curr: u64,
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.prev + self.curr;
        self.prev = self.curr;
        self.curr = next;

        Some(self.curr)
    }
}

impl Fibonacci {
    pub fn new() -> Self {
        Self { prev: 0, curr: 1 }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fib() {
        let mut f = Fibonacci::new();

        assert_eq!(f.nth(9), Some(89));
    }
}
