use std::ops::BitOr;

/// Parses a string of the form "A0"/"AA7"/etc. into a tuple of (column, row).
pub fn parse_coords(coords: &str) -> Result<(usize, usize), String> {
    let column_str = coords
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>();
    let row_str = coords
        .chars()
        .skip_while(|c| c.is_ascii_alphabetic())
        .collect::<String>();
    if column_str.is_empty() || row_str.is_empty() {
        return Err(format!("Invalid coordinates: {coords}"));
    }

    let column = column_str
        .chars()
        .fold(0usize, |col, c| col * 26 + (c as usize - 'A' as usize + 1))
        - 1;
    let row = row_str
        .parse::<usize>()
        .map_err(|e| format!("Invalid row: {row_str}: {e}"))?;

    Ok((column, row))
}

/// Converts a tuple of (column, row) into a string of the form "A1"/"AA7"/etc.
pub fn coords_to_string(col: usize, row: usize) -> String {
    let mut n = col + 1;
    let mut col_str = String::new();
    while n > 0 {
        n -= 1;
        col_str.insert(0, (b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    format!("{col_str}{row}")
}

/// An unordered set of unique elements. This is necessary because `HashSet` does not implement `Hash`.
#[derive(Hash, Debug, Clone, Eq)]
pub struct Set<T: Eq + Clone> {
    inner: Vec<T>,
}

impl<T: Eq + Clone> Set<T> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn insert(&mut self, value: T) {
        if !self.inner.contains(&value) {
            self.inner.push(value);
        }
    }

    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        for value in iter {
            self.insert(value);
        }
    }

    pub fn remove(&mut self, value: T) {
        self.inner.retain(|v| v != &value);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.inner.iter().all(|v| other.inner.contains(v))
    }
}

impl<T: Eq + Clone> FromIterator<T> for Set<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl<T: Eq + Clone> PartialEq for Set<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.len() == other.inner.len() && self.inner.iter().all(|v| other.inner.contains(v))
    }
}

impl<T: Eq + Clone> BitOr for Set<T> {
    type Output = Self;

    /// Returns the union of `self` and `other`. Overload of the `Set::or` method.
    fn bitor(self, other: Self) -> Self::Output {
        let mut new_set = self.clone();
        new_set.extend(other.iter().cloned());
        new_set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_coords_and_coords_to_string() {
        let pairs = vec![("A0", (0, 0)), ("AA7", (26, 7)), ("AB1", (27, 1))];

        for (coords, expected) in pairs {
            assert_eq!(parse_coords(coords).unwrap(), expected);
            assert_eq!(coords_to_string(expected.0, expected.1), coords);
        }
    }
}
