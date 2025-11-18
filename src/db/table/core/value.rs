use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
    Integer,
    Real,
    Text,
    Blob,
    Null,
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
    Null,
}

impl Value {
    pub fn get_type(&self) -> DataType {
        match self {
            Value::Integer(_) => DataType::Integer,
            Value::Real(_) => DataType::Real,
            Value::Text(_) => DataType::Text,
            Value::Blob(_) => DataType::Blob,
            Value::Null => DataType::Null,
        }
    }

    pub fn numeric_to_f64(&self) -> Option<f64> {
        match self {
            Value::Integer(_) | Value::Real(_) => self.cast_to_real(),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        self.get_type() == DataType::Null
    }

    pub fn cast_to_blob(&self) -> Option<Vec<u8>> {
        match self {
            Value::Null => None,
            Value::Blob(val) => Some(val.clone()),
            _ => self
                .cast_to_text()
                .map_or(None, |text| Some(text.into_bytes())),
        }
    }

    pub fn cast_to_text(&self) -> Option<String> {
        match self {
            Value::Null => None,
            Value::Text(val) => Some(val.clone()),
            Value::Blob(val) => String::from_utf8(val.clone()).ok(),
            Value::Integer(val) => Some(val.to_string()),
            Value::Real(val) => Some(val.to_string()),
        }
    }

    pub fn cast_to_real(&self) -> Option<f64> {
        match self {
            Value::Null => None,
            Value::Real(val) => Some(*val),
            Value::Integer(val) => Some(*val as f64),
            Value::Blob(_) => self
                .cast_to_text()
                .map_or(None, |text| Value::Text(text).cast_to_real()),
            Value::Text(val) => {
                let mut index: usize = 0;
                let mut has_period = false;
                for c in val.trim().chars() {
                    if c == '-' && index == 0
                        || c == '+' && index == 0
                        || c.is_ascii_digit()
                        || c == '.' && !has_period
                    {
                        index += 1;
                        if c == '.' {
                            has_period = true;
                        }
                        continue;
                    }
                    break;
                }
                if index == 0 {
                    Some(0.0)
                } else {
                    Some(val.trim()[0..index].parse::<f64>().unwrap_or(0.0))
                }
            }
        }
    }

    pub fn cast_to_real_lossless(&self) -> Option<f64> {
        match self {
            Value::Null | Value::Real(_) | Value::Integer(_) => self.cast_to_real(),
            Value::Blob(_) => {
                Value::Text(self.cast_to_text().unwrap_or(String::new())).cast_to_real_lossless()
            }
            Value::Text(val) => val.parse::<f64>().ok(),
        }
    }

    pub fn cast_to_int(&self) -> Option<i64> {
        match self {
            Value::Null => None,
            Value::Integer(val) => Some(*val),
            Value::Real(val) => {
                if *val > i64::MAX as f64 {
                    Some(i64::MAX)
                } else if *val < i64::MIN as f64 {
                    Some(i64::MIN)
                } else {
                    Some(*val as i64)
                }
            }
            Value::Blob(_) => self
                .cast_to_text()
                .map_or(None, |text| Value::Text(text).cast_to_int()),
            Value::Text(val) => {
                let (index, _) = val
                    .trim()
                    .chars()
                    .enumerate()
                    .find(|(index, c)| {
                        !c.is_ascii_digit()
                            && !(*c == '-' && *index == 0)
                            && !(*c == '+' && *index == 0)
                    })
                    .unwrap_or((val.len(), '\0'));
                if index == 0 {
                    Some(0)
                } else {
                    // Cast to real and then to int, since this handles values greater than i64::MAX or less than i64::MIN
                    Value::Real(
                        Value::Text(val.trim()[0..index].to_string())
                            .cast_to_real()
                            .unwrap_or(0.0),
                    )
                    .cast_to_int()
                }
            }
        }
    }

    pub fn cast_to_int_lossless(&self) -> Option<i64> {
        match self {
            Value::Null | Value::Real(_) | Value::Integer(_) => self.cast_to_int(),
            Value::Blob(_) => {
                Value::Text(self.cast_to_text().unwrap_or(String::new())).cast_to_int_lossless()
            }
            Value::Text(val) => val.parse::<i64>().ok(),
        }
    }

    pub fn exactly_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Blob(first), Value::Blob(second)) => first == second,
            (Value::Text(first), Value::Text(second)) => first == second,
            (Value::Integer(first), Value::Integer(second)) => first == second,
            (Value::Real(first), Value::Real(second)) => first == second,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // NULL handling
            (Value::Null, _) => Some(Ordering::Less), // NULL < NULL is also true
            (_, Value::Null) => Some(Ordering::Greater),
            // Same data types & int/real mixing
            (Value::Integer(a), Value::Integer(b)) => a.partial_cmp(b),
            (Value::Real(a), Value::Real(b)) => {
                if a.is_nan() && b.is_nan() {
                    Some(Ordering::Equal)
                } else if a.is_nan() {
                    Some(Ordering::Less)
                } else if b.is_nan() {
                    Some(Ordering::Greater)
                } else {
                    a.partial_cmp(b)
                }
            }
            (Value::Text(a), Value::Text(b)) => a.partial_cmp(b),
            (Value::Blob(a), Value::Blob(b)) => a.partial_cmp(b),
            // Int/Real mixing
            (Value::Integer(_), Value::Real(b)) => {
                self.cast_to_real().unwrap_or(0.0).partial_cmp(b)
            }
            (Value::Real(a), Value::Integer(_)) => {
                a.partial_cmp(&other.cast_to_real().unwrap_or(0.0))
            }
            // Mixing of incompatible data types
            (Value::Integer(_), Value::Text(_))
            | (Value::Integer(_), Value::Blob(_))
            | (Value::Real(_), Value::Text(_))
            | (Value::Real(_), Value::Blob(_)) => Some(Ordering::Less),
            (Value::Text(_), Value::Integer(_))
            | (Value::Blob(_), Value::Integer(_))
            | (Value::Text(_), Value::Real(_))
            | (Value::Blob(_), Value::Real(_)) => Some(Ordering::Greater),
            (Value::Text(_), Value::Blob(_)) => Some(Ordering::Less),
            (Value::Blob(_), Value::Text(_)) => Some(Ordering::Greater),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Integer(i) => {
                0u8.hash(state);
                i.hash(state);
            }
            Value::Real(f) => {
                1u8.hash(state);
                if f.is_nan() {
                    u64::MAX.hash(state);
                } else {
                    f.to_bits().hash(state);
                }
            }
            Value::Text(s) => {
                2u8.hash(state);
                s.hash(state);
            }
            Value::Blob(b) => {
                3u8.hash(state);
                b.hash(state);
            }
            Value::Null => {
                4u8.hash(state);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_partialord_behaves_as_expected() {
        // NULL with anything
        assert!(Value::Null.partial_cmp(&Value::Null) == Some(Ordering::Less));
        assert!(Value::Null.partial_cmp(&Value::Integer(0)) == Some(Ordering::Less));
        assert!(Value::Text("a".to_string()).partial_cmp(&Value::Null) == Some(Ordering::Greater));

        // Integer/Real and Text/Blob
        assert!(
            Value::Text("0".to_string()).partial_cmp(&Value::Real(0.0)) == Some(Ordering::Greater)
        );
        assert!(
            Value::Integer(999).partial_cmp(&Value::Text("1".to_string())) == Some(Ordering::Less)
        );
        assert!(Value::Blob(vec![0x01]).partial_cmp(&Value::Real(9.9)) == Some(Ordering::Greater));

        // Integer/Real and Integer/Real
        assert!(Value::Integer(42).partial_cmp(&Value::Real(42.01)) == Some(Ordering::Less));
        assert!(Value::Real(42.0).partial_cmp(&Value::Integer(42)) == Some(Ordering::Equal));
        assert!(Value::Integer(42).partial_cmp(&Value::Real(2.0)) == Some(Ordering::Greater));

        // Text and Blob
        assert!(
            Value::Text("abcd".to_string()).partial_cmp(&Value::Blob(vec![b'a', b'b', b'c', b'a']))
                == Some(Ordering::Less)
        );
        assert!(
            Value::Blob(vec![b'a', b'b', b'c', b'd']).partial_cmp(&Value::Text("abce".to_string()))
                == Some(Ordering::Greater)
        );

        // Text and Text
        assert!(
            Value::Text("abcd".to_string()).partial_cmp(&Value::Text("abce".to_string()))
                == Some(Ordering::Less)
        );
        assert!(
            Value::Text("abcd".to_string()).partial_cmp(&Value::Text("abc".to_string()))
                == Some(Ordering::Greater)
        );
        assert!(
            Value::Text("abcd".to_string()).partial_cmp(&Value::Text("abd".to_string()))
                == Some(Ordering::Less)
        );
        assert!(
            Value::Text("A".to_string()).partial_cmp(&Value::Text("1".to_string()))
                == Some(Ordering::Greater)
        );
        assert!(
            Value::Text("1234xyz".to_string()).partial_cmp(&Value::Text("1234xyz".to_string()))
                == Some(Ordering::Equal)
        );
        assert!(
            Value::Text("".to_string()).partial_cmp(&Value::Text("".to_string()))
                == Some(Ordering::Equal)
        );

        // Blob and Blob
        assert!(
            Value::Blob(vec![1, 2, 3]).partial_cmp(&Value::Blob(vec![1, 2, 3]))
                == Some(Ordering::Equal)
        );
        assert!(
            Value::Blob(vec![1, 2, 3]).partial_cmp(&Value::Blob(vec![1, 2, 3, 4]))
                == Some(Ordering::Less)
        );
        assert!(
            Value::Blob(vec![1, 2, 3]).partial_cmp(&Value::Blob(vec![1, 2, 2, 1]))
                == Some(Ordering::Greater)
        );
        assert!(Value::Blob(vec![]).partial_cmp(&Value::Blob(vec![])) == Some(Ordering::Equal));
        assert!(Value::Blob(vec![]).partial_cmp(&Value::Blob(vec![1])) == Some(Ordering::Less));
    }

    #[test]
    fn value_partialeq_behaves_as_expected() {
        assert!(Value::Null != Value::Null);
        assert!(Value::Integer(567) == Value::Real(567.0));
        assert!(Value::Integer(567) != Value::Text("567".to_string()));
    }

    #[test]
    fn value_exactlyeq_behaves_as_expected() {
        assert!(Value::Null.exactly_equal(&Value::Null));
        assert!(!Value::Integer(1).exactly_equal(&Value::Real(1.0)));
        assert!(Value::Blob(vec![1, 2, 3]).exactly_equal(&Value::Blob(vec![1, 2, 3])));
    }

    #[test]
    fn cast_to_blob_behaves_as_expected() {
        assert!(Value::Null.cast_to_text().is_none());
        assert_eq!(
            Value::Blob(vec![0x00, 0x01, 0x42]).cast_to_blob(),
            Some(vec![0x00, 0x01, 0x42])
        );
        assert_eq!(
            Value::Text("abc".to_string()).cast_to_blob(),
            Some(vec![0x61, 0x62, 0x63])
        );
        assert_eq!(Value::Integer(42).cast_to_blob(), Some(vec![0x34, 0x32]));
        assert_eq!(
            Value::Real(123.456789000).cast_to_blob(),
            Some(vec![
                0x31, 0x32, 0x33, 0x2e, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39
            ])
        );
    }

    #[test]
    fn cast_to_text_behaves_as_expected() {
        assert!(Value::Null.cast_to_text().is_none());
        assert_eq!(
            Value::Text("test".to_string()).cast_to_text(),
            Some("test".to_string())
        );
        assert_eq!(
            Value::Blob(vec![0x61, 0x62, 0x63]).cast_to_text(),
            Some("abc".to_string())
        );
        assert_eq!(Value::Integer(42).cast_to_text(), Some("42".to_string()));
        assert_eq!(
            Value::Real(123.456789000).cast_to_text(),
            Some("123.456789".to_string())
        );
    }

    #[test]
    fn cast_to_real_behaves_as_expected() {
        assert!(Value::Null.cast_to_real().is_none());
        assert_eq!(Value::Real(123.456789).cast_to_real(), Some(123.456789));
        assert_eq!(
            Value::Blob(vec![0x61, 0x62, 0x63]).cast_to_real(),
            Some(0.0)
        );
        assert_eq!(
            Value::Blob(vec![
                0x31, 0x32, 0x33, 0x2e, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x61, 0x62, 0x63
            ])
            .cast_to_real(),
            Some(123.456789)
        );
        assert_eq!(Value::Text("".to_string()).cast_to_real(), Some(0.0));
        assert_eq!(
            Value::Text("    -.543.21.9.abc".to_string()).cast_to_real(),
            Some(-0.543)
        );
        assert_eq!(
            Value::Text("    1000test".to_string()).cast_to_real(),
            Some(1000.0)
        );
        assert_eq!(
            Value::Text("-1234.567test".to_string()).cast_to_real(),
            Some(-1234.567)
        );
        assert_eq!(
            Value::Text("+0.246".to_string()).cast_to_real(),
            Some(0.246)
        );
        assert_eq!(Value::Integer(-42).cast_to_real(), Some(-42.0));
    }

    #[test]
    fn cast_to_real_lossless_behaves_as_expected() {
        assert!(Value::Null.cast_to_real_lossless().is_none());
        assert!(
            Value::Blob(vec![0x61, 0x62, 0x63])
                .cast_to_real_lossless()
                .is_none()
        );
        assert!(
            Value::Blob(vec![
                0x31, 0x32, 0x33, 0x2e, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x61, 0x62, 0x63
            ])
            .cast_to_real_lossless()
            .is_none()
        );
        assert_eq!(
            Value::Blob(vec![
                0x31, 0x32, 0x33, 0x2e, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39
            ])
            .cast_to_real_lossless(),
            Some(123.456789)
        );
        assert!(
            Value::Text("".to_string())
                .cast_to_real_lossless()
                .is_none()
        );
        assert!(
            Value::Text("-.543.21.9.abc".to_string())
                .cast_to_real_lossless()
                .is_none()
        );
        assert_eq!(
            Value::Text("-.543".to_string()).cast_to_real_lossless(),
            Some(-0.543)
        );
        assert!(
            Value::Text("1000test".to_string())
                .cast_to_real_lossless()
                .is_none()
        );
        assert_eq!(
            Value::Text("1000".to_string()).cast_to_real_lossless(),
            Some(1000.0)
        );
        assert!(
            Value::Text("  1000".to_string())
                .cast_to_real_lossless()
                .is_none()
        );
        assert!(
            Value::Text("-1234.567test".to_string())
                .cast_to_real_lossless()
                .is_none()
        );
        assert_eq!(
            Value::Text("-1234.567".to_string()).cast_to_real_lossless(),
            Some(-1234.567)
        );
        assert_eq!(
            Value::Text("+0.246".to_string()).cast_to_real_lossless(),
            Some(0.246)
        );
    }

    #[test]
    fn cast_to_int_behaves_as_expected() {
        assert!(Value::Null.cast_to_int().is_none());
        assert_eq!(Value::Integer(42).cast_to_int(), Some(42));
        assert_eq!(Value::Blob(vec![0x61, 0x62, 0x63]).cast_to_int(), Some(0));
        assert_eq!(
            Value::Blob(vec![
                0x31, 0x32, 0x33, 0x2e, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x61, 0x62, 0x63
            ])
            .cast_to_int(),
            Some(123)
        );
        assert_eq!(Value::Text("".to_string()).cast_to_int(), Some(0));
        assert_eq!(
            Value::Text("  1234.567test".to_string()).cast_to_int(),
            Some(1234)
        );
        assert_eq!(Value::Text(".43test".to_string()).cast_to_int(), Some(0));
        assert_eq!(Value::Text("+1.246".to_string()).cast_to_int(), Some(1));
        assert_eq!(
            Value::Text("-1234.567test".to_string()).cast_to_int(),
            Some(-1234)
        );
        assert_eq!(
            Value::Text("9223372036854775807".to_string()).cast_to_int(),
            Some(9223372036854775807)
        );
        assert_eq!(
            Value::Text("9223372036854775808".to_string()).cast_to_int(),
            Some(9223372036854775807)
        );
        assert_eq!(
            Value::Text("92233720368547758066543210".to_string()).cast_to_int(),
            Some(9223372036854775807)
        );
        assert_eq!(
            Value::Text("-9223372036854775808".to_string()).cast_to_int(),
            Some(-9223372036854775808)
        );
        assert_eq!(
            Value::Text("-9223372036854775810".to_string()).cast_to_int(),
            Some(-9223372036854775808)
        );
        assert_eq!(
            Value::Text("-922337203685477580001".to_string()).cast_to_int(),
            Some(-9223372036854775808)
        );
        assert_eq!(Value::Real(123.4).cast_to_int(), Some(123));
        assert_eq!(Value::Real(123.9).cast_to_int(), Some(123));
        assert_eq!(Value::Real(-123.4).cast_to_int(), Some(-123));
        assert_eq!(Value::Real(-123.9).cast_to_int(), Some(-123));
        assert_eq!(Value::Real(1e18).cast_to_int(), Some(1000000000000000000));
        assert_eq!(Value::Real(1e19).cast_to_int(), Some(9223372036854775807));
        assert_eq!(Value::Real(-1e18).cast_to_int(), Some(-1000000000000000000));
        assert_eq!(Value::Real(-1e19).cast_to_int(), Some(-9223372036854775808));
    }

    #[test]
    fn cast_to_int_lossless_behaves_as_expected() {
        assert!(Value::Null.cast_to_int_lossless().is_none());
        assert!(
            Value::Blob(vec![0x61, 0x62, 0x63])
                .cast_to_int_lossless()
                .is_none()
        );
        assert!(
            Value::Blob(vec![
                0x31, 0x32, 0x33, 0x2e, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x61, 0x62, 0x63
            ])
            .cast_to_int_lossless()
            .is_none()
        );
        assert_eq!(
            Value::Blob(vec![0x31, 0x32, 0x33]).cast_to_int_lossless(),
            Some(123)
        );
        assert!(Value::Text("".to_string()).cast_to_int_lossless().is_none());
        assert!(
            Value::Text("1234.567test".to_string())
                .cast_to_int_lossless()
                .is_none()
        );
        assert!(
            Value::Text("1234.567".to_string())
                .cast_to_int_lossless()
                .is_none()
        );
        assert_eq!(
            Value::Text("1234".to_string()).cast_to_int_lossless(),
            Some(1234)
        );
        assert!(
            Value::Text("  1234".to_string())
                .cast_to_int_lossless()
                .is_none()
        ); // TODO: does lossless mode ignore trailing whitespace? Standard doesn't have much info
        assert!(
            Value::Text(".43test".to_string())
                .cast_to_int_lossless()
                .is_none()
        );
        assert!(
            Value::Text("+1.246".to_string())
                .cast_to_int_lossless()
                .is_none()
        );
        assert_eq!(
            Value::Text("+1".to_string()).cast_to_int_lossless(),
            Some(1)
        );
        assert!(
            Value::Text("-1234.567test".to_string())
                .cast_to_int_lossless()
                .is_none()
        );
        assert!(
            Value::Text("-1234.567".to_string())
                .cast_to_int_lossless()
                .is_none()
        );
        assert_eq!(
            Value::Text("-1234".to_string()).cast_to_int_lossless(),
            Some(-1234)
        );
        assert_eq!(
            Value::Text("9223372036854775807".to_string()).cast_to_int_lossless(),
            Some(9223372036854775807)
        );
        assert!(
            Value::Text("9223372036854775808".to_string())
                .cast_to_int_lossless()
                .is_none()
        );
        assert!(
            Value::Text("92233720368547758066543210".to_string())
                .cast_to_int_lossless()
                .is_none()
        );
        assert_eq!(
            Value::Text("-9223372036854775808".to_string()).cast_to_int_lossless(),
            Some(-9223372036854775808)
        );
        assert!(
            Value::Text("-9223372036854775810".to_string())
                .cast_to_int_lossless()
                .is_none()
        );
        assert!(
            Value::Text("-922337203685477580001".to_string())
                .cast_to_int_lossless()
                .is_none()
        );
    }
}
