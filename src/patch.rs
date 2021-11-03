#![allow(unused)]

use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};
use std::ops::{Add, Sub};

use crate::document::Document;
/// commons: the common lines of two files.
/// 2^16 = 65536, The number of lines in a file is enough.
/// 0. line_num of common part in old file,
/// 1. line_num of common part in new file,
/// 2. length of the common part
type Common = (u16, u16, u16);

/// Delta: the changes of two files.
/// 0. start line number of Delta in old file,
/// 1. start line number of Delta in new file,
/// 2. line lenth removed in old file,
/// 3. line lenth added in new file
type Diff = (u16, u16, u16, u16);

/// An edit makes a Change.
/// 0. line_num in old document
/// 1. line count removed from old document
/// 2. line_num in new document
/// 3. line count added from new document
/// 4. removed lines
/// 5. added lines
pub type Change = (usize, usize, usize, usize, Vec<String>, Vec<String>);

/// A Patch consits of a list of Changes
#[derive(Debug)]
pub struct Patch(Vec<Change>);

impl Patch {
    pub fn from_vec(vec: Vec<Change>) -> Self {
        Self(vec)
    }
}
/*
impl Add for Patch {
    type Output = Self;
    fn add(self, doc: Document) -> Self {
        let lines = vec![];
        Self(lines)
    }
}
*/
#[cfg(test)]
mod tests {
    use self::super::*;
    #[test]
    // #[ignore]
    fn test_from_fs() {}
}
