pub mod binary_util;

use base64::{engine::general_purpose as base64_engine, Engine as _};
use binary_util::Binary;
use serde::{Deserialize, Serialize};
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::collections::HashMap;
use std::string::ToString;

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, tag_no_case, take_till1, take_while_m_n},
    character::complete::multispace0,
    combinator::{map, peek, value as n_value},
    error::context,
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair},
    IResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DValue {
    /// None
    None,

    /// String
    ///
    /// ```
    /// use datastruct::DValue;
    /// DValue::String("hello world".to_string());
    /// ```
    String(String),

    /// Number
    ///
    /// ```
    /// use datastruct::DValue;
    /// DValue::Number(10_f64);
    /// ```
    Number(f64),

    /// Boolean
    ///
    /// ```
    /// use datastruct::DValue;
    /// DValue::Boolean(true);
    /// ```
    Boolean(bool),

    /// List
    ///
    /// ```
    /// use datastruct::DValue;
    /// DValue::List(vec![
    /// DValue::Number(1.0),
    /// DValue::Number(2.0),
    /// DValue::Number(3.0),
    /// ]);
    /// ```
    List(Vec<DValue>),

    /// Dict
    ///
    /// ```
    /// use datastruct::DValue;
    /// DValue::Dict(std::collections::HashMap::new());
    /// ```
    Dict(HashMap<String, DValue>),

    /// Tuple
    ///
    /// ```
    /// use datastruct::DValue;
    ///
    /// DValue::Tuple((
    /// Box::new(DValue::Boolean(true)),
    /// Box::new(DValue::Boolean(false)),
    ///
    /// ));
    /// ```
    Tuple((Box<DValue>, Box<DValue>)),

    /// Binary
    ///
    /// ```
    ///
    /// use datastruct::{DValue ,binary_util::Binary};
    ///
    /// DValue::BinaryUtil(Binary::new(vec![72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100]));
    ///
    /// // or
    ///
    ///  let from_b64 = "SGVsbG8gV29ybGQ=".to_string();
    ///
    /// DValue::BinaryUtil(Binary::from_b64(from_b64).unwrap());
    /// ```
    ///
    BinaryUtil(Binary),
}

impl ToString for DValue {
    fn to_string(&self) -> String {
        match self {
            DValue::None => "none".to_string(),
            DValue::String(str) => format!("\"{}\"", str),
            DValue::Number(num) => num.to_string(),
            DValue::Boolean(bool) => match bool {
                true => "true".to_string(),
                false => "false".to_string(),
            },
            DValue::List(list) => {
                let elements: Vec<String> = list.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(","))
            }
            DValue::Dict(dict) => {
                let entries: Vec<String> = dict
                    .iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", entries.join(","))
            }

            DValue::Tuple(v) => {
                format!("({}, {})", v.0.to_string(), v.1.to_string())
            }
            DValue::BinaryUtil(val) => val.to_string(),
        }
    }
}

impl Ord for DValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight()
            .partial_cmp(&other.weight())
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for DValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DValue {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for DValue {}

impl DValue {
    pub fn from(data: &str) -> Self {
        let data = if data.starts_with("b:") && data.ends_with(':') {
            let base64_content = &data[2..data.len() - 1];
            match base64_engine::STANDARD.decode(base64_content) {
                Ok(decoded) => match String::from_utf8(decoded) {
                    Ok(s) => s,
                    Err(_) => return Self::None,
                },
                Err(_) => return Self::None,
            }
        } else {
            data.to_string()
        };

        match ValueParser::parse(&data) {
            Ok((_, v)) => v,
            Err(_) => Self::None,
        }
    }

    pub fn from_json(data: &str) -> Self {
        serde_json::from_str(data).unwrap_or(Self::None)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap_or(String::from("None"))
    }

    pub fn weight(&self) -> f64 {
        match self {
            DValue::Number(num) => *num,
            DValue::List(items) => items
                .iter()
                .map(|item| item.weight())
                .map(|w| if w == f64::MAX { 0.0 } else { w })
                .sum(),
            DValue::Dict(entries) => entries
                .values()
                .map(|item| item.weight())
                .map(|w| if w == f64::MAX { 0.0 } else { w })
                .sum(),

            DValue::Tuple(v) => {
                let first_weight = v.0.weight();
                let second_weight = v.1.weight();

                let first_weight = if first_weight == f64::MAX {
                    0.0
                } else {
                    first_weight
                };
                let second_weight = if second_weight == f64::MAX {
                    0.0
                } else {
                    second_weight
                };

                first_weight + second_weight
            }

            _ => f64::MAX,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            DValue::None => 0,
            DValue::String(str) => str.len(),
            DValue::Number(_) => 8,
            DValue::Boolean(_) => 1,

            DValue::List(list) => {
                let mut result = 0;

                for item in list {
                    result += item.size();
                }
                result
            }
            DValue::Dict(dict) => {
                let mut result = 0;
                for item in dict {
                    result += item.1.size();
                }
                result
            }
            DValue::Tuple(tuple) => tuple.0.size() + tuple.1.size(),
            DValue::BinaryUtil(bin) => bin.size(),
        }
    }

    pub fn datatype(&self) -> String {
        return match self {
            DValue::None => "None",
            DValue::String(_) => "String",
            DValue::Number(_) => "Number",
            DValue::Boolean(_) => "Boolean",
            DValue::List(_) => "List",
            DValue::Dict(_) => "Dict",
            DValue::Tuple(_) => "Tuple",
            DValue::BinaryUtil(_) => "Binary",
        }
        .to_string();
    }

    pub fn as_string(&self) -> Option<String> {
        return match self {
            DValue::String(val) => Some(val.to_string()),
            _ => None,
        };
    }

    pub fn as_number(&self) -> Option<f64> {
        return match self {
            DValue::Number(val) => Some(*val),
            _ => None,
        };
    }

    pub fn as_bool(&self) -> Option<bool> {
        return match self {
            DValue::Boolean(val) => Some(*val),
            _ => None,
        };
    }

    pub fn as_tuple(&self) -> Option<(Box<DValue>, Box<DValue>)> {
        return match self {
            DValue::Tuple(val) => Some(val.clone()),
            _ => None,
        };
    }

    pub fn as_list(&self) -> Option<Vec<DValue>> {
        return match self {
            DValue::List(val) => Some(val.clone()),
            _ => None,
        };
    }

    pub fn as_dict(&self) -> Option<HashMap<String, DValue>> {
        return match self {
            DValue::Dict(val) => Some(val.clone()),
            _ => None,
        };
    }
}

struct ValueParser {}

impl ValueParser {
    fn normal(msg: &str) -> IResult<&str, &str> {
        take_till1(|c: char| c == '\\' || c == '"' || c.is_ascii_control())(msg)
    }

    fn escapable(i: &str) -> IResult<&str, &str> {
        context(
            "escaped",
            alt((
                tag("\""),
                tag("\\"),
                tag("/"),
                tag("b"),
                tag("f"),
                tag("n"),
                tag("r"),
                tag("t"),
                ValueParser::parse_hex,
            )),
        )(i)
    }

    fn string_format(msg: &str) -> IResult<&str, &str> {
        escaped(ValueParser::normal, '\\', ValueParser::escapable)(msg)
    }

    fn parse_hex(msg: &str) -> IResult<&str, &str> {
        context(
            "hex string",
            preceded(
                peek(tag("u")),
                take_while_m_n(5, 5, |c: char| c.is_ascii_hexdigit() || c == 'u'),
            ),
        )(msg)
    }

    fn parse_str(msg: &str) -> IResult<&str, &str> {
        context(
            "string",
            alt((
                tag("\"\""),
                delimited(tag("\""), ValueParser::string_format, tag("\"")),
            )),
        )(msg)
    }

    fn parse_bin(msg: &str) -> IResult<&str, Binary> {
        let result: (&str, &str) = context(
            "binary",
            alt((
                tag("binary!()"),
                delimited(
                    tag("binary!("),
                    take_till1(|c: char| c == '\\' || c == ')' || c.is_ascii_control()),
                    tag(")"),
                ),
            )),
        )(msg)?;

        Ok((
            result.0,
            Binary::from_b64(result.1.to_string()).unwrap_or(Binary::new(vec![])),
        ))
    }

    fn parse_num(msg: &str) -> IResult<&str, f64> {
        double(msg)
    }

    fn parse_bool(msg: &str) -> IResult<&str, bool> {
        let true_parser = n_value(true, tag_no_case("true"));
        let false_parser = n_value(false, tag_no_case("false"));
        alt((true_parser, false_parser))(msg)
    }

    fn parse_list(msg: &str) -> IResult<&str, Vec<DValue>> {
        context(
            "list",
            delimited(
                tag("["),
                separated_list0(
                    tag(","),
                    delimited(multispace0, ValueParser::parse, multispace0),
                ),
                tag("]"),
            ),
        )(msg)
    }

    fn parse_dict(msg: &str) -> IResult<&str, HashMap<String, DValue>> {
        context(
            "object",
            delimited(
                tag("{"),
                map(
                    separated_list0(
                        tag(","),
                        separated_pair(
                            delimited(multispace0, ValueParser::parse_str, multispace0),
                            tag(":"),
                            delimited(multispace0, ValueParser::parse, multispace0),
                        ),
                    ),
                    |tuple_vec: Vec<(&str, DValue)>| {
                        tuple_vec
                            .into_iter()
                            .map(|(k, v)| (String::from(k), v))
                            .collect()
                    },
                ),
                tag("}"),
            ),
        )(msg)
    }

    fn parse_tuple(msg: &str) -> IResult<&str, (Box<DValue>, Box<DValue>)> {
        context(
            "tuple",
            delimited(
                tag("("),
                map(
                    separated_pair(
                        delimited(multispace0, ValueParser::parse, multispace0),
                        tag(","),
                        delimited(multispace0, ValueParser::parse, multispace0),
                    ),
                    |pair: (DValue, DValue)| (Box::new(pair.0), Box::new(pair.1)),
                ),
                tag(")"),
            ),
        )(msg)
    }

    fn parse(msg: &str) -> IResult<&str, DValue> {
        context(
            "value",
            delimited(
                multispace0,
                alt((
                    map(ValueParser::parse_num, DValue::Number),
                    map(ValueParser::parse_bool, DValue::Boolean),
                    map(ValueParser::parse_str, |s| DValue::String(String::from(s))),
                    map(ValueParser::parse_list, DValue::List),
                    map(ValueParser::parse_dict, DValue::Dict),
                    map(ValueParser::parse_tuple, DValue::Tuple),
                    map(ValueParser::parse_bin, DValue::BinaryUtil),
                )),
                multispace0,
            ),
        )(&msg)
    }
}

#[cfg(test)]
mod test {

    use crate::{binary_util::Binary, DValue, ValueParser};

    #[test]
    fn parse_list() {
        assert_eq!(
            ValueParser::parse("[1,2,3,4,5]"),
            Ok((
                "",
                DValue::List(vec![
                    DValue::Number(1.0),
                    DValue::Number(2.0),
                    DValue::Number(3.0),
                    DValue::Number(4.0),
                    DValue::Number(5.0),
                ])
            ))
        );
    }

    #[test]
    fn parse_tuple() {
        assert_eq!(
            ValueParser::parse("(true,1)"),
            Ok((
                "",
                DValue::Tuple((
                    Box::new(DValue::Boolean(true)),
                    Box::new(DValue::Number(1_f64))
                ))
            ))
        );
    }

    #[test]
    fn parse_binary() {
        let message = "binary!(bWVtZW50byBtb3Jp)";
        assert_eq!(
            ValueParser::parse(message),
            Ok((
                "",
                DValue::BinaryUtil(Binary::new(
                    [109, 101, 109, 101, 110, 116, 111, 32, 109, 111, 114, 105].to_vec()
                ))
            ))
        )
    }
    #[test]
    fn parse_to_json() {
        let value = DValue::List(vec![
            DValue::Number(3.0),
            DValue::Number(6.0),
            DValue::Number(9.0),
        ]);
        let expected_json = r#"{"List":[{"Number":3.0},{"Number":6.0},{"Number":9.0}]}"#;
        assert_eq!(value.to_json(), expected_json);
    }
}
