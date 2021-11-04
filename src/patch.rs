#![allow(unused)]

use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};
use std::ops::{Add, Sub};

use crate::document::Document;

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
    pub fn as_vec_ref(&self) -> &Vec<Change> {
        let Patch(vec) = self;
        vec
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
