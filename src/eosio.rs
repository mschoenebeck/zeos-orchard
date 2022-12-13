//! Helper functions to deal with EOSIO/Antelope related stuff.

use std::cmp::min;

/// Rust equivalent of: cdt/libraries/eosiolib/core/eosio/name.hpp -> name.char_to_value()
/// See also: https://github.com/AntelopeIO/cdt/blob/c010d6fae2656f212f78d01c41812734934eb54c/libraries/eosiolib/core/eosio/name.hpp#L108
pub fn char_to_value(c: u8) -> u8
{
    if c == '.' as u8
    {
       return 0;
    }
    else if  c >= '1' as u8 && c <= '5' as u8 
    {
       return (c - '1' as u8) + 1;
    }
    else if c >= 'a' as u8 && c <= 'z' as u8 
    {
       return (c - 'a' as u8) + 6;
    }
    // character is not in allowed character set for names
    return 0;
 }

/// Rust equivalent of: cdt/libraries/eosiolib/core/eosio/name.hpp -> name() constructor
/// See also: https://github.com/AntelopeIO/cdt/blob/c010d6fae2656f212f78d01c41812734934eb54c/libraries/eosiolib/core/eosio/name.hpp#L77
pub fn name_to_value(str: &String) -> u64
{
    if str.len() > 13
    {
        // string is too long to be a valid name
        return 0;
    }
    if str.is_empty()
    {
        return 0;
    }
    let mut value = 0;
    let n = min(str.len(), 12);
    for i in 0..n
    {
        value <<= 5;
        value |= char_to_value(str.as_bytes()[i]) as u64;
    }
        value <<= 4 + 5*(12 - n);
        if str.len() == 13
        {
            let v = char_to_value(str.as_bytes()[12]) as u64;
            if v > 0x0F
            {
                // thirteenth character in name cannot be a letter that comes after j
                return 0;
            }
            value |= v;
        }
    value
}

/// Converts an EOSIO encoded name to human readable string
pub fn value_to_name(value: u64) -> String
{
    let charmap = vec!['.', '1', '2', '3', '4', '5', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];
    let mask = 0xF800000000000000;

    let mut v = value;
    let mut str = "".to_string();
    for i in 0..13
    {
        if v == 0
        {
            return str;
        }

        let indx = (v & mask) >> (if i == 12 { 60 } else { 59 });
        str.push(charmap[indx as usize]);

        v <<= 5;
    }
    str
}

/// Converts a string to EOSIO symbol code
pub fn string_to_symbol_code(str: &String) -> u64
{
    let mut value = 0;
    if str.len() > 7 {
        // string is too long to be a valid symbol_code
        return 0;
    }
    for itr in str.chars().rev() {
        if itr < 'A' || itr > 'Z' {
           // only uppercase letters allowed in symbol_code string
           return 0;
        }
        value <<= 8;
        value |= itr as u64;
    }
    value
}

/// Converts a string and precision to EOSIO symbol
pub fn string_to_symbol(str: &String, precision: u8) -> u64
{
    (string_to_symbol_code(str) << 8) | precision as u64
}

/// Converts an EOSIO symbol code to string
pub fn symbol_code_to_string(raw: u64) -> String
{
    let mut v = raw;
    let mut s = "".to_string();
    while v > 0
    {
        s.push((v & 0xFF) as u8 as char);
        v >>= 8;
    }
    s
}

/// Converts an EOSIO symbol to string and precision
pub fn symbol_to_string_precision(raw: u64) -> (String, u8)
{
    (symbol_code_to_string(raw >> 8), (raw & 0xFF) as u8)
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test1()
    {
        assert_eq!(name_to_value(&"eosio".to_string()), 6138663577826885632);
        assert_eq!(name_to_value(&"eosio.msig".to_string()), 6138663587900751872);
        assert_eq!(name_to_value(&"eosio.token".to_string()), 6138663591592764928);
        assert_eq!(string_to_symbol_code(&"ZEOSZEOS".to_string()), 0);
        assert_eq!(string_to_symbol_code(&"eos".to_string()), 0);
        assert_eq!(string_to_symbol_code(&"EOS".to_string()), 5459781);
        assert_eq!(string_to_symbol(&"EOS".to_string(), 4), 1397703940);
        assert_eq!(string_to_symbol(&"ZEOS".to_string(), 4), 357812230660);
        assert_eq!(symbol_code_to_string(5459781), "EOS".to_string());
        assert_eq!(symbol_to_string_precision(357812230660), ("ZEOS".to_string(), 4));
        assert_eq!(value_to_name(6138663577826885632), "eosio".to_string());
        assert_eq!(value_to_name(6138663587900751872), "eosio.msig".to_string());
        assert_eq!(value_to_name(6138663591592764928), "eosio.token".to_string());
    }
}