#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Philox4x32_10(pub [u32; 2]);

impl Philox4x32_10 {
    pub fn gen(&self, mut ctr: [u32; 4]) -> [u32; 4] {
        let mut key = self.0;
        for _ in 0..10 {
            round(&mut key, &mut ctr);
        }
        ctr
    }

    pub fn gen_f32s(&self, ctr: [u32; 4]) -> [f32; 4] {
        self.gen(ctr).map(|x| (x >> 8) as f32 * (f32::EPSILON / 2.))
    }
}

fn round(key: &mut [u32; 2], ctr: &mut [u32; 4]) {
    let (r0, l1) = mulhilo(ctr[2], 0xD2511F53);
    let (r1, l0) = mulhilo(ctr[0], 0xCD9E8D57);

    let ctr1 = ctr[1];
    let ctr3 = ctr[3];

    ctr[0] = r0 ^ key[1] ^ ctr3;
    ctr[1] = l0;
    ctr[2] = r1 ^ key[0] ^ ctr1;
    ctr[3] = l1;

    key[0] = key[0].wrapping_add(0x9E3779B9);
    key[1] = key[1].wrapping_add(0xBB67AE85);
}

fn mulhilo(a: u32, b: u32) -> (u32, u32) {
    let p = u64::from(a).wrapping_mul(b.into());
    ((p >> 32) as u32, p as u32)
}
