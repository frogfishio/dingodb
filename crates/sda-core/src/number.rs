use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExactNum(BigRational);

#[derive(Debug, Clone)]
pub enum ParseNumError {
    InvalidFormat,
    InvalidDigits,
    ZeroDenominator,
}

impl fmt::Display for ParseNumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "invalid numeric format"),
            Self::InvalidDigits => write!(f, "invalid numeric digits"),
            Self::ZeroDenominator => write!(f, "zero denominator"),
        }
    }
}

impl std::error::Error for ParseNumError {}

impl ExactNum {
    pub fn parse_literal(src: &str) -> Result<Self, ParseNumError> {
        let src = src.trim();
        if src.is_empty() {
            return Err(ParseNumError::InvalidFormat);
        }

        let (sign, unsigned) = if let Some(rest) = src.strip_prefix('-') {
            (-1i8, rest)
        } else if let Some(rest) = src.strip_prefix('+') {
            (1i8, rest)
        } else {
            (1i8, src)
        };

        let (mantissa, exponent) = if let Some((mantissa, exponent)) = unsigned.split_once(['e', 'E']) {
            let parsed_exp = exponent.parse::<i64>().map_err(|_| ParseNumError::InvalidFormat)?;
            (mantissa, parsed_exp)
        } else {
            (unsigned, 0i64)
        };

        let (int_part, frac_part) = if let Some((int_part, frac_part)) = mantissa.split_once('.') {
            (int_part, frac_part)
        } else {
            (mantissa, "")
        };

        if int_part.is_empty() && frac_part.is_empty() {
            return Err(ParseNumError::InvalidFormat);
        }

        if !int_part.chars().all(|c| c.is_ascii_digit()) || !frac_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(ParseNumError::InvalidDigits);
        }

        let digits = format!("{int_part}{frac_part}");
        let mut numerator = if digits.is_empty() {
            BigInt::zero()
        } else {
            BigInt::from_str(&digits).map_err(|_| ParseNumError::InvalidDigits)?
        };

        let scale = frac_part.len() as i64 - exponent;
        let denominator = if scale <= 0 {
            numerator *= pow10((-scale) as u32);
            BigInt::one()
        } else {
            pow10(scale as u32)
        };

        if sign < 0 {
            numerator = -numerator;
        }

        Ok(Self(BigRational::new(numerator, denominator)))
    }

    pub fn parse_canonical(src: &str) -> Result<Self, ParseNumError> {
        if let Some((numerator, denominator)) = src.split_once('/') {
            let numerator = BigInt::from_str(numerator).map_err(|_| ParseNumError::InvalidDigits)?;
            let denominator = BigInt::from_str(denominator).map_err(|_| ParseNumError::InvalidDigits)?;
            if denominator.is_zero() {
                return Err(ParseNumError::ZeroDenominator);
            }
            Ok(Self(BigRational::new(numerator, denominator)))
        } else {
            Self::parse_literal(src)
        }
    }

    pub fn from_usize(value: usize) -> Self {
        Self(BigRational::from_integer(BigInt::from(value)))
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn add(&self, other: &Self) -> Self {
        Self(self.0.clone() + other.0.clone())
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self(self.0.clone() - other.0.clone())
    }

    pub fn mul(&self, other: &Self) -> Self {
        Self(self.0.clone() * other.0.clone())
    }

    pub fn div(&self, other: &Self) -> Self {
        Self(self.0.clone() / other.0.clone())
    }

    pub fn neg(&self) -> Self {
        Self(-self.0.clone())
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        if let Some(decimal) = self.finite_decimal_string() {
            if let Ok(number) = serde_json::Number::from_str(&decimal) {
                return serde_json::Value::Number(number);
            }
        }

        serde_json::json!({
            "$type": "num",
            "$value": self.canonical_string(),
        })
    }

    pub fn canonical_string(&self) -> String {
        if let Some(decimal) = self.finite_decimal_string() {
            decimal
        } else {
            format!("{}/{}", self.0.numer(), self.0.denom())
        }
    }

    fn finite_decimal_string(&self) -> Option<String> {
        let numerator = self.0.numer().clone();
        let mut denominator = self.0.denom().clone();

        if denominator == BigInt::one() {
            return Some(numerator.to_string());
        }

        let two = BigInt::from(2u8);
        let five = BigInt::from(5u8);
        let mut twos = 0u32;
        let mut fives = 0u32;

        while (&denominator % &two).is_zero() {
            denominator /= &two;
            twos += 1;
        }

        while (&denominator % &five).is_zero() {
            denominator /= &five;
            fives += 1;
        }

        if denominator != BigInt::one() {
            return None;
        }

        let scale = twos.max(fives);
        let mut adjusted = numerator.abs();
        if scale > twos {
            adjusted *= five.pow(scale - twos);
        }
        if scale > fives {
            adjusted *= two.pow(scale - fives);
        }

        let digits = adjusted.to_string();
        let negative = numerator.is_negative();
        let sign = if negative { "-" } else { "" };

        if scale == 0 {
            return Some(format!("{sign}{digits}"));
        }

        let scale = scale as usize;
        if digits.len() <= scale {
            let padding = "0".repeat(scale - digits.len());
            Some(format!("{sign}0.{padding}{digits}"))
        } else {
            let split = digits.len() - scale;
            Some(format!("{sign}{}.{}", &digits[..split], &digits[split..]))
        }
    }
}

impl fmt::Display for ExactNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.canonical_string())
    }
}

fn pow10(exp: u32) -> BigInt {
    BigInt::from(10u8).pow(exp)
}