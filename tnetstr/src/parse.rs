use super::{
    TNetEntry,
    TNetStrError,
    TNetList,
    TNetDictionary,
    TNetFloat,
    DataType,
    TNetData
};
use std::{
    io::Read,
    str
};

const INVALID_LENGTH_VALUE: u32 = 1111111111 as u32;

fn parse_string(bytes: &[u8]) -> Result<&str, TNetStrError> {
    match str::from_utf8(&bytes) {
        Err(_) => Err(TNetStrError::DataNotUTF8Compatible),
        Ok(s) => Ok(s)
    }
}

fn parse_val<T>(bytes: &[u8]) -> Result<T, TNetStrError> where T: std::str::FromStr {
    let s = parse_string(bytes)?;
    match s.parse() {
        Err(_) => Err(TNetStrError::CouldNotParseData),
        Ok(u) => Ok(u)
    }
}

fn parse_float(bytes: &[u8]) -> Result<TNetFloat, TNetStrError> {
    TNetFloat::from_decimal_str(parse_string(bytes)?)
}

fn parse_list(bytes: &[u8]) -> Result<Vec<TNetEntry>, TNetStrError> {
    let mut position = 0;
    let mut list: TNetList = vec![];
    while position < bytes.len() {
        let tnetstr = parse_entry(bytes, &mut position)?;
        list.push(tnetstr);
    }
    Ok(list)
}

fn parse_dictionary(bytes: &[u8]) -> Result<TNetDictionary, TNetStrError> {
    let mut position = 0;
    let mut dict = TNetDictionary::new();
    let len = bytes.len();
    while position < len {
        let key = parse_entry(bytes, &mut position)?;
        if !(position < len) {
            return Err(TNetStrError::UnbalancedDictionary)
        }
        let value = parse_entry(bytes, &mut position)?;
        dict.add(key, value)?;
    }
    Ok(dict)
}

fn read_length_byte(byte: &u8, bytes_read: &mut Vec<u8>) -> Result<Option<u32>, TNetStrError> {
    match byte {
        b':' => {
            match bytes_read.len() > 0 /*length must be specified*/ {
                false => Err(TNetStrError::NoLengthSpecified),
                true => {
                    match str::from_utf8(bytes_read) {
                        Err(_) => Err(TNetStrError::LengthWasNotValidUTF8),
                        Ok(len_str) => match len_str.parse::<u32>() {
                            Err(_) => {
                                println!("Failed to parse: {}", len_str);
                                Err(TNetStrError::CouldNotParseLength)
                            },
                            Ok(len) => Ok(Some(len))
                        }
                    }
                }
            }
        }
        b'0'|b'1'|b'2'|b'3'|b'4'|b'5'|b'6'|b'7'|b'8'|b'9' => {
            bytes_read.push(byte.clone());
            if bytes_read.len() > 9 /*more than 9 digits not allowed by spec*/ {
                Err(TNetStrError::LengthTooLong)
            } else {
                Ok(None)
            }
        },
        u => {
            Err(TNetStrError::NonASCIINumericValueInLengthField(u.clone()))
        }
    }
}

fn parse_length(input: &[u8], position: &mut usize) -> Result<u32, TNetStrError> {
    let mut index: usize = 0;
    let mut bytes_read: Vec<u8> = Vec::with_capacity(9);
    let rest = &input[*position ..];
    for byte in rest {
        index += 1;
        match read_length_byte(byte, &mut bytes_read)? {
            Some(len) => {
                *position += index;
                return Ok(len)
            },
            None => continue
        };
    };

    Err(TNetStrError::LengthTerminatorNotFound)
}

fn parse_entry(input: &[u8], position: &mut usize) -> Result<TNetEntry, TNetStrError> {
    let len = parse_length(input, position)?;
    let data_bytes: Vec<u8> = if len > 0 {
        input[*position..((len as usize) + *position)]
            .iter()
            .map(|b| b.clone())
            .collect::<Vec<u8>>()
    } else {
        vec![]
    };

    *position += len as usize;
    let t = match input.get(*position) {
        Some(t) => t,
        None => return Err(TNetStrError::LengthWasNotAccurate)
    };

    let t = DataType::from_byte(t)?;
    *position += 1;

    let data = match t {
        DataType::Bytes => TNetData::Bytes(data_bytes),
        DataType::Null => TNetData::Null,
        DataType::Dictionary => TNetData::Dictionary(parse_dictionary(&data_bytes)?),
        DataType::List => TNetData::List(parse_list(&data_bytes)?),
        DataType::Integer => TNetData::Integer(parse_val(&data_bytes)?),
        DataType::Float => TNetData::Float(parse_float(&data_bytes)?),
        DataType::Boolean => TNetData::Boolean(parse_val(&data_bytes)?)
    };

    Ok(TNetEntry {
        size: len,
        data_type: t,
        data: data
    })
}

/// Attempts to read the first tnetstring from a slice of bytes
pub fn parse(data: &[u8]) -> Result<TNetEntry, TNetStrError> {
    let mut pos = 0;
    parse_entry(data, &mut pos)
}

/// Attempts to read the next tnetstring from a byte stream
pub fn parse_stream(reader: &mut Read) -> Result<TNetEntry, TNetStrError>
{
    let mut len_chars: Vec<u8> = Vec::with_capacity(9);
    let mut len_buff:[u8; 1] = [0];
    let data_length: u32;
    loop {
        match reader.read_exact(&mut len_buff) {
            Err(e) => return Err(TNetStrError::StreamReadFailed(format!("{:?}", e))),
            Ok(_) => {
                match read_length_byte(&len_buff[0], &mut len_chars)? {
                    Some(l) => {
                        data_length = l;
                        break;
                    },
                    None => continue
                }
            }
        }
    };

    if data_length == INVALID_LENGTH_VALUE {
        return Err(TNetStrError::StreamReadFailed("Reached end of stream without reading a full length value".to_string()))
    }
    // create full tnetentry buffer
    let mut all_buff: Vec<u8> = vec![0; len_chars.len() + (data_length + 2) as usize];

    // copy length characters in
    for i in 0..len_chars.len() {
        all_buff[i] = len_chars[i].clone();
    }
    // set the length terminator byte
    all_buff[len_chars.len()] = b':';

    // read the expected number of bytes for the data content, + 1 for the type declaration
    match reader.read_exact(&mut all_buff[len_chars.len() + 1..]) {
        Err(e) => return Err(TNetStrError::StreamReadFailed(format!("{:?}", e))),
        Ok(_) => {
            let mut pos = 0;
            parse_entry(&all_buff, &mut pos)
        }
    }
}



#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn parses_length_from_start() {
        let s = "10:aaaaaaaaaa,".as_bytes();
        let mut index = 0;
        match parse_length(s, &mut index) {
            Ok(len) => {
                assert_eq!(len, 10);
                assert_eq!(index, 3);
            },
            Err(e) => panic!("Failed to parse length: {}", e)
        }
    }

    #[test]
    fn parses_length_from_middle() {
        let s = "1234#9:aaaaaaaaa,".as_bytes();
        let mut index = 5;
        match parse_length(s, &mut index) {
            Ok(len) => {
                assert_eq!(len, 9);
                assert_eq!(index, 7);
            },
            Err(e) => panic!("Failed to parse length: {}", e)
        }
    }

    #[test]
    fn parses_length_for_null() {
        let s = "0:~".as_bytes();
        let mut index = 0;
        match parse_length(s, &mut index) {
            Ok(len) => {
                assert_eq!(len, 0);
                assert_eq!(index, 2);
            },
            Err(e) => panic!("Failed to parse length: {}", e)
        }
    }

    fn check(input: &[u8], expected: &TNetEntry) {
        match parse_list(input) {
            Err(e) => panic!("Failed to parse: {}", e),
            Ok(entries) => {
                assert_eq!(entries.len(), 1);
                let entry = &entries[0];
                assert_eq!(entry.size, expected.size);
                assert_eq!(entry.data_type, expected.data_type);
                assert_eq!(entry.data, expected.data);
            }
        }
    }

    #[test]
    fn parses_null() {
        let input = "0:~".as_bytes();

        check(input, &TNetEntry {
            size: 0,
            data_type: DataType::Null,
            data: TNetData::Null
        });
    }

    #[test]
    fn parses_bytes() {
        let input = "10:aaaaaaaaaa,".as_bytes();

        check(input, &TNetEntry {
            size: 10,
            data_type: DataType::Bytes,
            data: TNetData::Bytes("aaaaaaaaaa".as_bytes().iter().map(|b|b.clone()).collect())
        });
    }

    #[test]
    fn parses_int() {
        let input = "3:123#".as_bytes();
        check(input, &TNetEntry {
            size: 3,
            data_type: DataType::Integer,
            data: TNetData::Integer(123)
        });
    }

    #[test]
    fn parses_float() {
        let input = "6:12.543^".as_bytes();
        check(input, &TNetEntry {
            size: 6,
            data_type: DataType::Float,
            data: TNetData::Float(TNetFloat{
                integral: 12,
                fractional: 543
            })
        });
    }

    #[test]
    fn parses_bool() {
        let input = "4:true!".as_bytes();
        check(input, &TNetEntry {
            size: 4,
            data_type: DataType::Boolean,
            data: TNetData::Boolean(true)
        });
    }

    #[test]
    fn parses_list() {
        let input = "24:4:true!6:0.4529^5:abcde,]".as_bytes();
        check(input, &TNetEntry {
            size: 24,
            data_type: DataType::List,
            data: TNetData::List(vec![
                TNetEntry {
                    size: 4,
                    data_type: DataType::Boolean,
                    data: TNetData::Boolean(true)
                },
                TNetEntry {
                    size: 6,
                    data_type: DataType::Float,
                    data: TNetData::Float(TNetFloat{integral: 0, fractional: 4529})
                },
                TNetEntry {
                    size: 5,
                    data_type: DataType::Bytes,
                    data: TNetData::Bytes("abcde".as_bytes().iter().map(|b|b.clone()).collect())
                }
            ])
        });
    }

    #[test]
    fn parses_dict() {
        let input = "19:1:a,1:1#3:bbb,2:hi,}".as_bytes();
        check(input, &TNetEntry {
            size: 19,
            data_type: DataType::Dictionary,
            data: TNetData::Dictionary(TNetDictionary::from_vec(vec![
                TNetEntry {
                    size: 1,
                    data_type: DataType::Bytes,
                    data: TNetData::Bytes("a".as_bytes().iter().map(|b|b.clone()).collect())
                },
                TNetEntry {
                    size: 1,
                    data_type: DataType::Integer,
                    data: TNetData::Integer(1)
                },
                TNetEntry {
                    size: 3,
                    data_type: DataType::Bytes,
                    data: TNetData::Bytes("bbb".as_bytes().iter().map(|b|b.clone()).collect())
                },
                TNetEntry {
                    size: 2,
                    data_type: DataType::Bytes,
                    data: TNetData::Bytes("hi".as_bytes().iter().map(|b|b.clone()).collect())
                }
            ]).unwrap())
        });
    }

    #[test]
    fn parses_stream() {
        let mut input = "10:aaaaaaaaaa,".as_bytes();
        let expected = TNetEntry {
            size: 10,
            data_type: DataType::Bytes,
            data: TNetData::Bytes(b"aaaaaaaaaa".iter().map(|b| b.clone()).collect())
        };
        let actual = parse_stream(&mut input).unwrap();

        assert_eq!(expected, actual);
    }

    fn expect_error(test: &str, input: &[u8], expected_error: TNetStrError) {
        let mut pos = 0;
        match parse_entry(input, &mut pos) {
            Err(e) => assert_eq!(e, expected_error),
            _ => panic!("Did not error on {}", test)
        }
    }

    #[test]
    fn error_if_length_not_specified() {
        let input = ":abc,".as_bytes();
        expect_error("missing length", input, TNetStrError::NoLengthSpecified)
    }

    #[test]
    fn error_if_length_not_ascii_numeric() {
        let input = "z:abc,".as_bytes();
        expect_error("non-ascii numeric input", input, TNetStrError::NonASCIINumericValueInLengthField(b'z'))
    }

    #[test]
    fn error_if_length_greater_than_9_digits() {
        let input = "9999999991:abc,".as_bytes();
        expect_error("length greater than 9 digits", input, TNetStrError::LengthTooLong)
    }

    #[test]
    fn error_if_length_not_terminated_with_colon() {
        let input = "11".as_bytes();
        expect_error("missing length terminator", input, TNetStrError::LengthTerminatorNotFound)
    }

    #[test]
    fn error_on_incorrect_length() {
        let input = "2:a,".as_bytes();
        expect_error("missing length terminator", input, TNetStrError::LengthWasNotAccurate)
    }

    #[test]
    fn error_on_unbalanced_dictionary() {
        let input = "4:1:a,}".as_bytes();
        expect_error("unbalanced dictionary", input, TNetStrError::UnbalancedDictionary)
    }

    #[test]
    fn error_on_non_byte_dictionary_key() {
        let input = "8:1:1#1:a,}".as_bytes();
        expect_error("non-byte dictionary key", input, TNetStrError::DictionaryKeyWasNotBytes)
    }
}