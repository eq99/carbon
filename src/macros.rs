#![allow(unused)]

//! This mod defines a list of macrs,
//! See more details about how to use macros:
//! https://stackoverflow.com/a/31749071

/// This macro is a shotcut to create a String.
macro_rules! s {
    ($s:expr) => {
        String::from($s)
    };
}

pub(crate) use s;
