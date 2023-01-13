use std::{collections::BTreeSet, fmt::Display, str::FromStr};

/// List of ids, supporting nice parsing from cli.
///
/// - 1
/// - 1,2
/// - 1-4,6
#[derive(Debug, Clone, Default)]
pub struct Ids(pub Vec<i32>);

impl FromStr for Ids {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ids = BTreeSet::new();

        for s in s.split(',') {
            if let Some((start, end)) = s.split_once('-') {
                if let Some((start, end)) = start.parse::<i32>().ok().zip(end.parse().ok()) {
                    ids.extend(start..=end)
                }
            } else if let Ok(id) = s.parse() {
                ids.insert(id);
            }
        }

        Ok(Self(ids.into_iter().collect()))
    }
}

impl Display for Ids {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.0.iter().map(|id| id.to_string()).collect::<Vec<_>>();
        write!(f, "{}", s.join(","))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use expect_test::{expect, Expect};

    use super::Ids;

    fn check(actual: &str, expect: Expect) {
        let actual = format!("{:?}", Ids::from_str(actual));
        expect.assert_eq(&actual);
    }

    #[test]
    fn test_single() {
        check("1", expect!["Ok(Ids([1]))"]);
    }

    #[test]
    fn test_commas() {
        check("1,2", expect!["Ok(Ids([1, 2]))"]);
    }

    #[test]
    fn test_comma_ordering() {
        check("5,1,2", expect!["Ok(Ids([1, 2, 5]))"]);
    }

    #[test]
    fn test_comma_duplicates() {
        check("1,1", expect!["Ok(Ids([1]))"]);
    }

    #[test]
    fn test_range() {
        check("1-2", expect!["Ok(Ids([1, 2]))"]);
    }

    #[test]
    fn test_range_ordering() {
        check(
            "10-14,1-3",
            expect!["Ok(Ids([1, 2, 3, 10, 11, 12, 13, 14]))"],
        );
    }

    #[test]
    fn test_mixed() {
        check("1-4,2", expect!["Ok(Ids([1, 2, 3, 4]))"]);
    }
}
