use crate::interpreter::ast::OrderByDirection;
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

#[derive(Debug, PartialOrd, Clone)]
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

    pub fn compare(&self, other: &Value, direction: &OrderByDirection) -> Ordering {
        let result = match (self, other) {
            // NULL handling
            (Value::Null, _) => Ordering::Less, // NULL < NULL is also true
            (_, Value::Null) => Ordering::Greater,
            // Same data types & int/real mixing
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            (Value::Real(a), Value::Real(b)) => {
                if a.is_nan() && b.is_nan() {
                    Ordering::Equal
                } else if a.is_nan() {
                    Ordering::Less
                } else if b.is_nan() {
                    Ordering::Greater
                } else {
                    a.partial_cmp(b).unwrap_or(Ordering::Equal)
                }
            }
            (Value::Text(a), Value::Text(b)) => a.cmp(b),
            (Value::Blob(a), Value::Blob(b)) => a.cmp(b),
            // Int/Real mixing
            (Value::Integer(_), Value::Real(b)) => self
                .cast_to_real()
                .unwrap_or(0.0)
                .partial_cmp(b)
                .unwrap_or(Ordering::Equal),
            (Value::Real(a), Value::Integer(_)) => a
                .partial_cmp(&other.cast_to_real().unwrap_or(0.0))
                .unwrap_or(Ordering::Equal),
            // Mixing of incompatible data types
            (Value::Integer(_), Value::Text(_))
            | (Value::Integer(_), Value::Blob(_))
            | (Value::Real(_), Value::Text(_))
            | (Value::Real(_), Value::Blob(_)) => Ordering::Less,
            (Value::Text(_), Value::Integer(_))
            | (Value::Blob(_), Value::Integer(_))
            | (Value::Text(_), Value::Real(_))
            | (Value::Blob(_), Value::Real(_)) => Ordering::Greater,
            (Value::Text(_), Value::Blob(_)) => Ordering::Less,
            (Value::Blob(_), Value::Text(_)) => Ordering::Greater,
        };

        if direction == &OrderByDirection::Desc {
            result.reverse()
        } else {
            result
        }
    }

    pub fn numeric_to_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(_) | Value::Real(_) => self.cast_to_int(),
            _ => None,
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
                let mut has_negative = false;
                for c in val.trim().chars() {
                    if c == '-' && index == 0
                        || c.is_ascii_digit()
                        || c == '.' && index != 0 && !has_period && !(index == 1 && has_negative)
                    {
                        index += 1;
                        if c == '-' {
                            has_negative = true;
                        } else if c == '.' {
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
                    .find(|(index, c)| !c.is_ascii_digit() && !(*c == '-' && *index == 0))
                    .unwrap_or((0, ' '));
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
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Real(a), Value::Real(b)) => {
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Blob(a), Value::Blob(b)) => a == b,
            (Value::Null, Value::Null) => true, // TODO: Bad - NULL == NULL should be false but this breaks assert_eq!
            _ => false,
        }
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
