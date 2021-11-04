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
    /// create Pathc from changes' vec
    /// example:
    /// ```ignore
    /// let mut changes: Vec<Change> = vec![];
    /// changes.push((0, 0, 0, 0, removed_lines, added_lines));
    /// Patch::from_vec(changes)
    /// ```
    pub fn from_vec(vec: Vec<Change>) -> Self {
        Self(vec)
    }

    /// This method is useful to get ref of inner Vec
    /// example:
    /// ```ignore
    /// let patch = Patch::from_fs(s!("tests/patch"));
    /// let patch_vec_ref = patch.as_vec_ref();
    /// for patch in patch_vec_ref {
    ///     println!("{:?}", patch);
    /// }
    /// ```
    pub fn as_vec_ref(&self) -> &Vec<Change> {
        let Patch(vec) = self;
        vec
    }
}

impl Add<Document> for Patch {
    type Output = Document;
    /// You can read Add trait for Document.
    /// old_doc + patch = new_doc <=> patch + old_doc = new_doc
    fn add(self, doc: Document) -> Document {
        doc + self
    }
}

#[cfg(test)]
mod tests {
    use self::super::*;
    #[test]
    // #[ignore]
    fn test_from_fs() {}
}
