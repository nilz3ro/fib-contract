#![no_std]

pub struct Fibonacci {
    prev: u64,
    curr: u64,
}

impl Fibonacci {
    pub fn new() -> Self {
        Self { prev: 0, curr: 1 }
    }
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

pub struct Threebonacci {
    prev_1: u64,
    prev_2: u64,
    curr: u64,
}

impl Threebonacci {
    pub fn new() -> Self {
        Self {
            curr: 1,
            prev_1: 0,
            prev_2: 0,
        }
    }
}

impl Iterator for Threebonacci {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let next = self.prev_1 + self.prev_2 + self.curr;

        self.prev_2 = self.prev_1;
        self.prev_1 = self.curr;
        self.curr = next;

        Some(self.curr)
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

    #[test]
    fn threebonacci() {
        let mut t = Threebonacci::new();

        assert_eq!(t.next(), Some(1));
        assert_eq!(t.next(), Some(2));
        assert_eq!(t.next(), Some(4));
        assert_eq!(t.next(), Some(7));
    }
}
