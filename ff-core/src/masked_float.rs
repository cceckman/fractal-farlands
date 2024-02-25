// Sizes and masks to select the components of an IEEE f64.
const EXPONENT: usize = 11;
const FRACTION: usize = 52;
const SIGN_MASK: u64 = 1 << 63;
const EXPONENT_SIGN_MASK: u64 = 1 << 62;
const EXPONENT_MASK: u64 = ((1 << EXPONENT) - 1) << FRACTION;
const FRACTION_MASK: u64 = (1 << FRACTION) - 1;

// Generate all necessary exponent and fraction bitmasks at compile-time.
const EXPONENT_MASKS: [u64; EXPONENT] = {
    let mut arr = [0u64; EXPONENT];
    let mut i = 0;
    while i < EXPONENT {
        arr[EXPONENT - i - 1] = ((1 << i) - 1) << (62 - i) as u64;
        i += 1;
    }
    arr
};
const FRACTION_MASKS: [u64; FRACTION] = {
    let mut arr = [0u64; FRACTION];
    let mut i = 0;
    while i < FRACTION {
        arr[FRACTION - i - 1] = (!((1 << i) - 1)) & FRACTION_MASK;
        i += 1;
    }
    arr
};

/// Masked variant of an f64 that limits the available number of usable exponent
/// and fraction bits. The underlying float type is f64, so the number of
/// Exponent bits (E) must be 10 or fewer, and the number of fraction bits
/// must be 52 or fewer.
/// IEEE float's exponents use an offset-binary approach for the exponent,
/// i.e, the 11 exponent bits represent an unsigned number that has 1023
/// subtracted from it, so masking out some bits is not as easy as it is
/// for the fractional bits:
///
/// For exponents less than 1023, the MSB will not be set, and these will end
/// up as negative numbers.
///
/// To limit these to a smaller bitwidth, we want to clamp the values when they would
/// overflow, i.e., if it were 8-bit, 127 biased,
///
///  0b0000_0001 would be the smallest normal exponent, 2^(-126).
///
/// If we want to force this into acting like a 6-bit value, we have to set all
/// values less than 64 to be 64, i.e.,  0b0010_0000, allowing any values larger
/// than that to stay as they are.
///
/// For positive exponents, we have to instead clamp above, i.e.,
///   0b1010_0101 becomes 0b1010_000
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct MaskedFloat<const E: usize, const M: usize> {
    val: f64,
}

impl<const E: usize, const F: usize> MaskedFloat<E, F> {
    pub fn new(val: f64) -> Self {
        let bits = val.to_bits();
        let sign = bits & (SIGN_MASK | EXPONENT_SIGN_MASK);
        let exp = if bits & EXPONENT_SIGN_MASK != 0 {
            if bits & EXPONENT_MASKS[E] != 0 {
                ((1 << E + FRACTION) - 1) & EXPONENT_MASK
            } else {
                (bits & !EXPONENT_MASKS[E]) & EXPONENT_MASK
            }
        } else {
            // bits & EXPONENT_SIGN_MASK == 0
            if bits & EXPONENT_MASKS[E] != EXPONENT_MASKS[E] {
                EXPONENT_MASKS[E]
            } else {
                (bits | EXPONENT_MASKS[E]) & EXPONENT_MASK
            }
        };
        let frac = bits & FRACTION_MASKS[F];
        Self {
            val: f64::from_bits(sign | exp | frac),
        }
    }

    pub fn to_f64(&self) -> f64 {
        self.val
    }
}

impl<const E: usize, const F: usize> std::ops::Add for MaskedFloat<E, F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            val: self.val + other.val,
        }
    }
}

impl<const E: usize, const F: usize> std::ops::Sub for MaskedFloat<E, F> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            val: self.val - other.val,
        }
    }
}

impl<const E: usize, const F: usize> std::ops::Mul for MaskedFloat<E, F> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            val: self.val * other.val,
        }
    }
}

impl<const E: usize, const F: usize> std::ops::Div for MaskedFloat<E, F> {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            val: self.val / other.val,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_one_plus_one() {
        let one = MaskedFloat::<10, 10>::new(1.0);
        let two = MaskedFloat::<10, 10>::new(2.0);
        let epsilon = MaskedFloat::<10, 10>::new(0.001);

        assert!((one + one).to_f64() - 2.0 < 0.001);
        assert!(one + one - two < epsilon);
    }

    #[test]
    fn test_masking_big() {
        // Shouldn't be able to represent 2^64 with 6 bits of exponent.
        let f = f64::powf(2.1, 64.0);
        let too_big = MaskedFloat::<6, 50>::new(f);
        assert!(too_big.to_f64() < f);

        // Should be able to represent 2^64 with 8 bits of exponent.
        let ok = MaskedFloat::<8, 50>::new(f);
        assert!(ok.to_f64() - f < 0.001);
    }

    #[test]
    fn test_masking_small() {
        // Shouldn't be able to represent 2^-64 with 4 bits of exponent.
        let f = f64::powf(2.1, -64.0);
        let too_small = MaskedFloat::<6, 50>::new(f);
        assert!(too_small.to_f64() > f);

        // Should be able to represent 2^-64 with 8 bits of exponent.
        let ok = MaskedFloat::<8, 50>::new(f);
        assert!(ok.to_f64() - f < 0.001);
    }
}
