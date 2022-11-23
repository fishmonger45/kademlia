use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Error, Formatter, Write};

pub const ID_SIZE: usize = 20;

/// Node identification
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Id([u8; ID_SIZE]);

impl Id {
    /// Create a new [`Id`]
    pub fn new(xs: [u8; 20]) -> Self {
        Id(xs)
    }

    /// Create a new [`Id`] from [`rand::thread_rng`]
    pub fn random() -> Self {
        Id(thread_rng().gen::<[u8; ID_SIZE]>())
    }

    /// Find the XOR distance between two [`Id`]s via the number of prefix zero bits
    pub fn distance(&self, x: &Self) -> usize {
        Id(self
            .0
            .iter()
            .zip(x.0.iter())
            .map(|(a, b)| a ^ b)
            .collect::<Vec<u8>>()
            .try_into()
            .expect("need 20 bytes for Id"))
        .leading_zeros()
    }

    /// Number of prefix zero bits between two [`Id`]s
    pub fn leading_zeros(&self) -> usize {
        for i in 0..20 {
            for j in (0..8).rev() {
                if (self.0[i] >> j) & 0x01 != 0 {
                    return i * 8 + (7 - j);
                }
            }
        }

        ID_SIZE * 8
    }

    pub fn hex(&self) -> String {
        let mut s = String::from("0x");
        for b in self.0 {
            write!(&mut s, "{0:02x}", b).expect("unable to write bytes to format id as hex repr");
        }
        s
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "0x")?;
        for b in self.0.iter() {
            write!(f, "{0:02x}", b)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Reflexivity, Symmetry, Transitivity
    #[test]
    fn distance() {
        let x = Id::new([1u8; 20]);
        let y = Id::new([4u8; 20]);
        let z = Id::new([5u8; 20]);
        assert_eq!(x.distance(&x), ID_SIZE * 8);
        assert_eq!(x.distance(&y), 5);
        assert_eq!(x.distance(&y), y.distance(&x));
        assert!((x.distance(&y) + y.distance(&z)) >= x.distance(&z));
        assert!((x.distance(&y) + y.distance(&z)) >= x.distance(&z));
    }

    #[test]
    fn leading_zeros() {
        assert_eq!(Id([0; 20]).leading_zeros(), ID_SIZE * 8);
        assert_eq!(Id([255; 20]).leading_zeros(), 0);

        let mut xs = [0u8; 20];
        xs[5] = 0x0F;
        assert_eq!(Id(xs).leading_zeros(), 5 * 8 + 4);

        let mut xs = [0u8; 20];
        xs[5] = 0xF0;
        assert_eq!(Id(xs).leading_zeros(), 5 * 8);
    }

    #[test]
    fn hex() {
        let x = Id::new([0u8; 20]);
        assert_eq!(
            x.hex(),
            format!("0x{}", (0..40).map(|_| "0").collect::<String>())
        );
        assert_eq!(x.hex(), format!("{:?}", x))
    }
}
