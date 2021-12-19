use std::cmp;
use std::fmt;

#[derive(Eq, PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct IndexedValue<T: Eq + Ord + Clone + Copy> {
    pub n: usize,
    pub value: T
}

impl<T: Eq + Ord + Clone + Copy> Ord for IndexedValue<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        if self.value != other.value {
            self.value.cmp(&other.value)
        } else {
            // valueが同じならnの若い方を最大値として扱うため通常の逆順にしている
            other.n.cmp(&self.n)
        }
    }
}

impl<T: fmt::Display + Eq + Ord + Clone + Copy> fmt::Display for IndexedValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (n={})", self.value, self.n)
    }
}

#[cfg(test)]
mod tests {
    use crate::indexed_value::IndexedValue;
    use std::cmp;

    #[test]
    fn different_value() {
        let a = IndexedValue { n: 0, value: 10 };
        let b = IndexedValue { n: 1, value: 20 };
        assert_eq!(cmp::max(a, b), b);
    }
    #[test]
    fn same_value() {
        let a = IndexedValue { n: 0, value: 10 };
        let b = IndexedValue { n: 1, value: 10 };
        assert_eq!(cmp::max(a, b), a);
    }
}