

#[derive(Debug, PartialEq)]
pub enum RespType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RespType>),
}
pub type RespResult = Result<(RespType, usize), String>;

fn read_length(data: &[u8]) -> (usize, usize) {
    let mut length = 0;
    let mut pos = 0;

    for &b in data {
        if b >= b'0' && b <= b'9' {
            length = length * 10 + (b - b'0') as usize;
            pos += 1;
        } else {
            // Reached non-digit, assume it's \r\n, so consume 2 more bytes
            return (length, pos + 2);
        }
    }
    (0, 0)
}

fn read_simple_string(data: &[u8]) -> RespResult {
    // First byte is '+'
    let mut pos = 1;
    while pos < data.len() && data[pos] != b'\r' {
        pos += 1;
    }

    let s = std::str::from_utf8(&data[1..pos])
        .map_err(|e| e.to_string())?
        .to_string();
    Ok((RespType::SimpleString(s), pos + 2)) // +2 for \r\n
}

fn read_error(data: &[u8]) -> RespResult {
    // First byte is '-'
    let mut pos = 1;
    while pos < data.len() && data[pos] != b'\r' {
        pos += 1;
    }

    let s = std::str::from_utf8(&data[1..pos])
        .map_err(|e| e.to_string())?
        .to_string();
    Ok((RespType::Error(s), pos + 2))
}

fn read_int64(data: &[u8]) -> RespResult {
    // First byte is ':'
    let mut pos = 1;
    let mut value: i64 = 0;

    while pos < data.len() && data[pos] != b'\r' {
        value = value * 10 + (data[pos] - b'0') as i64;
        pos += 1;
    }

    Ok((RespType::Integer(value), pos + 2))
}

fn read_bulk_string(data: &[u8]) -> RespResult {
    // First byte is '$'
    let (len, delta) = read_length(&data[1..]);
    let pos = 1 + delta;

    let s = std::str::from_utf8(&data[pos..pos + len])
        .map_err(|e| e.to_string())?
        .to_string();
    // pos + len + 2 (for the trailing \r\n)
    Ok((RespType::BulkString(s), pos + len + 2))
}

fn read_array(data: &[u8]) -> RespResult {
    // First byte is '*'
    let (count, delta) = read_length(&data[1..]);
    let mut pos = 1 + delta;
    
    let mut elems = Vec::with_capacity(count);

    for _ in 0..count {
        let (elem, consumed) = decode_one(&data[pos..])?;
        elems.push(elem);
        pos += consumed;
    }

    Ok((RespType::Array(elems), pos))
}

pub fn decode_one(data: &[u8]) -> RespResult {
    if data.is_empty() {
        return Err("no data".to_string());
    }

    match data[0] {
        b'+' => read_simple_string(data),
        b'-' => read_error(data),
        b':' => read_int64(data),
        b'$' => read_bulk_string(data),
        b'*' => read_array(data),
        _ => Err(format!("unknown RESP type: {}", data[0] as char)),
    }
}

pub fn decode(data: &[u8]) -> Result<RespType, String> {
    if data.is_empty() {
        return Err("no data".to_string());
    }
    let (value, _) = decode_one(data)?;
    Ok(value)
}
impl RespType {

    pub fn to_string_vec(&self) -> Result<Vec<String>, String> {
        match self {
            RespType::Array(arr) => {
                let mut strings = Vec::new();
                for item in arr {
                    if let RespType::BulkString(s) = item {
                        strings.push(s.clone());
                    } else {
                        return Err("Expected array of bulk strings".to_string());
                    }
                }
                Ok(strings)
            }
            _ => Err("Expected RESP array".to_string()),
        }
    }


    pub fn encode_string(value: &str, is_simple: bool) -> Vec<u8> {
        if is_simple {
            let mut res = Vec::with_capacity(1 + value.len() + 2);
            res.push(b'+');
            res.extend_from_slice(value.as_bytes());
            res.extend_from_slice(b"\r\n");
            res
        } else {
            let len_str = value.len().to_string();
            let mut res = Vec::with_capacity(1 + len_str.len() + 2 + value.len() + 2);
            res.push(b'$');
            res.extend_from_slice(len_str.as_bytes());
            res.extend_from_slice(b"\r\n");
            res.extend_from_slice(value.as_bytes());
            res.extend_from_slice(b"\r\n");
            res
        }
    }
}