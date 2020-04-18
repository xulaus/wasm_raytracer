// Based on XoShiRo256

#[derive(Clone)]
pub struct RandomSeq {
    state: [u64; 4],
}

impl RandomSeq {
    pub fn new() -> RandomSeq {
        RandomSeq {
            state: [31415, 27182, 141142, 17320],
        }
    }

    pub fn next(&mut self) -> u64 {
        fn rol64(x: u64, k: u64) -> u64 {
            (x << k) | (x >> (64 - k))
        }
        let result = rol64(self.state[1] * 5, 7) * 9;
        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = rol64(self.state[3], 45);

        return result;
    }
}
