use core::cmp::Ordering;
use std::convert::Into;

#[derive(Debug, Copy, Clone)]
pub struct Amount(i64);

impl Default for Amount {
    fn default() -> Self {
        Amount(0)
    }
}

impl Amount {
    pub fn new(val: f64) -> Option<Self> {
        if val.is_nan() || val.is_infinite() {
            return None;
        }
        let vf = val * 10000.0 + 0.5 * val.signum();
        if vf < (i64::MIN as f64) || vf > (i64::MAX as f64) {
            return None;
        }
        Some(Amount(vf as i64))
    }

    #[must_use = "this returns the result of the operation, ithout modifying the original"]
    pub const fn checked_add(self, rhs: Amount) -> Option<Amount> {
        match self.0.checked_add(rhs.0) {
            None => None,
            Some(v) => Some(Amount(v)),
        }
    }

    #[must_use = "this returns the result of the operation, ithout modifying the original"]
    pub const fn checked_sub(self, rhs: Amount) -> Option<Amount> {
        match self.0.checked_sub(rhs.0) {
            None => None,
            Some(v) => Some(Amount(v)),
        }
    }
}

impl std::cmp::PartialEq for Amount {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Amount {}

impl std::cmp::PartialOrd for Amount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl std::cmp::Ord for Amount {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Into<f64> for Amount {
    fn into(self) -> f64 {
        self.0 as f64 / 10000.0
    }
}
