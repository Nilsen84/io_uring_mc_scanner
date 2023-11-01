use gcd::Gcd;
use rand::{thread_rng, Rng};
use std::net::Ipv4Addr;

// https://en.wikipedia.org/wiki/Linear_congruential_generator#c_%E2%89%A0_0

const M: u64 = u32::MAX as u64 + 1;

pub struct IpIterator {
    active: bool,
    c: u64,
    first_pick: u64,
    pick: u64,
}

impl IpIterator {
    pub fn new() -> Self {
        let first_pick = thread_rng().gen_range(0..M);

        Self {
            active: true,
            c: pick_random_coprime(M),
            first_pick,
            pick: first_pick,
        }
    }
}

impl Iterator for IpIterator {
    type Item = Ipv4Addr;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.active {
            return None;
        }

        let current_pick = self.pick;
        self.pick = (current_pick + self.c) % M;

        if self.pick == self.first_pick {
            self.active = false;
        }

        Some(Ipv4Addr::from(self.pick as u32))
    }
}

fn pick_random_coprime(end: u64) -> u64 {
    let lower_range = end / 4;
    let upper_range = end - lower_range;

    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.gen_range(lower_range..=upper_range))
        .find(|&i| end.gcd(i) == 1)
        .unwrap()
}