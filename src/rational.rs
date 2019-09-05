pub const TB_27MHZ: Rational = Rational {
    num: 1,
    den: 27_000_000,
};
#[allow(dead_code)]
pub const TB_90KHZ: Rational = Rational {
    num: 1,
    den: 90_000,
};
#[allow(dead_code)]
pub const TB_1MS: Rational = Rational {
    num: 1,
    den: 1_000_000,
};
pub const TB_1NS: Rational = Rational {
    num: 1,
    den: 1_000_000_000,
};

pub struct Rational {
    num: u64,
    den: u64,
}

pub fn rescale(v: u64, src: Rational, dst: Rational) -> u64 {
    let num = u128::from(src.num) * u128::from(dst.den);
    let den = u128::from(src.den) * u128::from(dst.num);

    ((v as f64) * ((num as f64) / (den as f64))) as u64
}
