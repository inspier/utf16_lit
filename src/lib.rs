//! Provides a macro for making utf-16 literals.
//!
//! ```rust
//! use utf16_lit::{utf16, utf16_null};
//!
//! const EXAMPLE: &[u16] = &utf16!("example");
//!
//! const EXAMPLE_NULL: &[u16] = &utf16_null!("example");
//!
//! fn main() {
//!   let v: Vec<u16> = "example".encode_utf16().collect();
//!   assert_eq!(v, EXAMPLE);
//!
//!   let v: Vec<u16> = "example".encode_utf16().chain(Some(0)).collect();
//!   assert_eq!(v, EXAMPLE_NULL);
//!   let v: Vec<u16> = "example\0".encode_utf16().collect();
//!   assert_eq!(v, EXAMPLE_NULL);
//!
//!   // You don't even need to assign the output to a const.
//!   assert_eq!(utf16!("This works")[0], 'T' as u8 as u16);
//! }
//! ```

#[doc(hidden)]
pub const fn always_true() -> bool {
    true
}

#[doc(hidden)]
pub const fn wide_len(s: &str) -> usize {
    let s = s.as_bytes();
    let mut length: usize = 0;
    let mut index: usize = 0;
    while index < s.len() {
        let mut chr = 0;
        if s[index] & 0x80 == 0x00 {
            chr = s[index] as u32;
            index += 1;
        } else if s[index] & 0xe0 == 0xc0 {
            chr = (s[index] as u32 & 0x1f) << 6 | (s[index + 1] as u32 & 0x3f);
            index += 2;
        } else if s[index] & 0xf0 == 0xe0 {
            chr = (s[index] as u32 & 0x0f) << 12
                | (s[index + 1] as u32 & 0x3f) << 6
                | (s[index + 2] as u32 & 0x3f);
            index += 3;
        } else if s[index] & 0xf8 == 0xf0 {
            chr = (s[index] as u32 & 0x07) << 18
                | (s[index + 1] as u32 & 0x3f) << 12
                | (s[index + 2] as u32 & 0x3f) << 6
                | (s[index + 3] as u32 & 0x3f);
            index += 4;
        } else {
            ["Invalid literal provided."][(always_true() as usize)];
        };
        length += [1, 2][(chr >= 0x10000) as usize];
    }
    length
}

#[doc(hidden)]
#[macro_export]
macro_rules! length {
    ($arg:expr) => {{
        $crate::wide_len($arg)
    }};
}

/// Turns a string literal into a `[u16]` literal.
///
/// If you want to have a "null terminated" string (such as for some parts of
/// Windows FFI) then you should use [`utf16_null!`](utf16_null!).
// Inspired by code from https://github.com/CasualX/obfstr/blob/master/src/lib.rs.
#[macro_export]
macro_rules! utf16 {
    ($arg:expr) => {{
        const ARRAY_LENGTH: usize = $crate::length!($arg);
        const RESULT: [u16; ARRAY_LENGTH] = {
            pub const fn wide(s: &str) -> [u16; ARRAY_LENGTH] {
                let s = s.as_bytes();
                let mut data = [0u16; ARRAY_LENGTH];
                let mut char_index: usize = 0;
                let mut data_index: usize = 0;
                while char_index < s.len() {
                    let mut chr = 0;
                    if s[char_index] & 0x80 == 0x00 {
                        chr = s[char_index] as u32;
                        char_index += 1;
                    } else if s[char_index] & 0xe0 == 0xc0 {
                        chr =
                            (s[char_index] as u32 & 0x1f) << 6 | (s[char_index + 1] as u32 & 0x3f);
                        char_index += 2;
                    } else if s[char_index] & 0xf0 == 0xe0 {
                        chr = (s[char_index] as u32 & 0x0f) << 12
                            | (s[char_index + 1] as u32 & 0x3f) << 6
                            | (s[char_index + 2] as u32 & 0x3f);
                        char_index += 3;
                    } else if s[char_index] & 0xf8 == 0xf0 {
                        chr = (s[char_index] as u32 & 0x07) << 18
                            | (s[char_index + 1] as u32 & 0x3f) << 12
                            | (s[char_index + 2] as u32 & 0x3f) << 6
                            | (s[char_index + 3] as u32 & 0x3f);
                        char_index += 4;
                    } else {
                        ["Invalid literal provided."][($crate::always_true() as usize)];
                    };
                    if chr >= 0x10000 {
                        data[data_index] = (0xD800 + (chr - 0x10000) / 0x400) as u16;
                        data[data_index + 1] = (0xDC00 + (chr - 0x10000) % 0x400) as u16;
                        data_index += 2;
                    } else {
                        data[data_index] = chr as u16;
                        data_index += 1;
                    }
                }
                data
            }
            wide($arg)
        };
        RESULT
    }};
}

/// Turns a string literal into a `[u16]` literal with a null on the end.
///
/// If you do **not** want to have a null terminator added to the string then
/// you should use [`utf16!`](utf16!).
#[macro_export]
macro_rules! utf16_null {
    ($arg:expr) => {{
        const U16: &[u16] = &$crate::utf16!($arg);
        const RESULT: [u16; U16.len() + 1] = {
            let mut data = [0u16; U16.len() + 1];
            let mut i = 0;
            while i < data.len() - 1 {
                data[i] = U16[i];
                i += 1;
            }
            data
        };
        RESULT
    }};
}
