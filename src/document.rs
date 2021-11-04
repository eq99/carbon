#![allow(unused)]

use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};
use std::ops::{Add, Sub};

use crate::macros::s;
use crate::patch::{Change, Patch};

/// The Document struct reprents a real word document.
/// Because operations are line based, We use a Vec<String> to store lines.
#[derive(Debug, Clone)]
pub struct Document(Vec<String>);

impl Document {
    /// Create a Document object from file.
    /// Example:
    /// ```ignore
    /// let doc = Document::from_fs(s!("tests/base"));
    /// assert_eq!(s!("A\nB\nC\nD\nE\nF\n"), doc.to_string());
    /// ```
    pub fn from_fs(file: String) -> Self {
        let input = File::open(file).unwrap();
        let lines_vec: Vec<_> = BufReader::new(input)
            .lines()
            .map(|result| result.unwrap())
            .collect();
        Self(lines_vec)
    }

    /// Display lines with number.
    /// It's useful when debug, example:
    /// ```ignore
    /// let doc = Document::from_fs(s!("tests/base"));
    /// doc.show();
    /// ```
    /// 0. A
    /// 1. B
    /// 2. C
    /// 3. D
    /// 4. E
    /// 5. F
    pub fn show(&self) {
        let Document(lines) = self;
        for line in lines.iter().enumerate() {
            println!("{}. {}", line.0, line.1);
        }
    }

    /// This method is useful to get ref of inner Vec
    /// example:
    /// ```ignore
    /// let doc = Document::from_fs(s!("tests/base"));
    /// let vec_ref = doc.as_vec_ref();
    /// for line in vec_ref {
    ///     println!("{:?}", line);
    /// }
    /// ```
    pub fn as_vec_ref(&self) -> &Vec<String> {
        let Self(vec) = self;
        vec
    }

    /// Convet vec to string
    /// example:
    /// ```ignore
    /// let doc = Document::from_fs(s!("tests/base"));
    /// assert_eq!(s!("A\nB\nC\nD\nE\nF\n"), doc.to_string());
    /// ```
    pub fn to_string(&self) -> String {
        self.as_vec_ref()
            .iter()
            .fold(String::new(), |s, line| format!("{}{}\n", s, line))
    }

    /// Write Document to file system.
    /// Step 1: calculate content sha-256 hash
    /// Step 2: use first two hex code as folder, the rest as file name.
    /// This idea comes from git.
    pub fn write_to_store(&self) {
        // let data = "Some data!";
        // fs::write("/tmp/foo", data).expect("Unable to write file");
        todo!()
    }

    /// Create a Document from store.
    pub fn read_from_store(hash: String) -> Self {
        // let lines = vec![];
        // Self(lines)
        todo!()
    }
}

impl Sub for Document {
    type Output = Patch;
    /// Diff of two document
    /// algorithm notes:
    /// Step 1: find all the common parts of two documents.
    /// 1. compare all sub string of old string with new string.
    /// 2. compare all sub string of new string with old string symmetrically.
    ///
    /// Step 2: Choose the right common parts greedily by common_line_count.
    ///
    /// Step 3: Construct the changes.
    ///
    /// Step 4: return Patch.
    ///
    /// If you don't kown the edtor matrix,
    /// You can read paper: http://www.xmailserver.org/diff2.pdf
    /// The Fig 1. will help you a lot.
    ///
    /// @param{self}: the new document
    /// @param{other}: the old document
    fn sub(self, other: Self) -> Self::Output {
        let this_vec_ref = self.as_vec_ref();
        let other_vec_ref = other.as_vec_ref();

        // step 1: Find common parts
        let mut commons = vec![];
        let mut common_line_count = 0;
        let mut this_line_num = 0;
        let mut other_line_num = 0;
        let mut other_lines_count = other_vec_ref.len();
        let mut this_lines_count = this_vec_ref.len();
        // diagonal and upper
        for i in 0..this_lines_count {
            for j in 0..other_lines_count {
                if i + j < this_lines_count
                    && (this_vec_ref[i + j].len() == other_vec_ref[j].len()
                        && this_vec_ref[i + j] == other_vec_ref[j])
                {
                    if common_line_count == 0 {
                        this_line_num = i + j;
                        other_line_num = j;
                    }
                    common_line_count += 1;
                } else {
                    if common_line_count > 0 {
                        commons.push((other_line_num, this_line_num, common_line_count));
                        common_line_count = 0;
                    }
                }
            }
        }
        // lower diagonal
        for i in 1..other_lines_count {
            for j in 0..this_lines_count {
                if i + j < other_lines_count
                    && (this_vec_ref[j].len() == other_vec_ref[i + j].len()
                        && this_vec_ref[j] == other_vec_ref[i + j])
                {
                    if common_line_count == 0 {
                        this_line_num = j;
                        other_line_num = i + j;
                    }
                    common_line_count += 1;
                } else {
                    if common_line_count > 0 {
                        commons.push((other_line_num, this_line_num, common_line_count));
                        common_line_count = 0;
                    }
                }
            }
        }

        // println!("Commons: {:?}", commons);

        // Step 2: pick out the right parts
        commons.sort_by_key(|c| -(c.2 as i64));
        let mut picked = vec![];
        let mut pickable = true;
        if let Some(c) = commons.get(0) {
            picked.push(c.clone())
        }
        for candidate in commons {
            for picked in &picked {
                //  * * * *
                //  * * * *
                //  * * * *
                //         \
                //          \
                //           * * * *
                //           * * * *
                pickable = ((candidate.0 + candidate.2 - 1 < picked.0
                && candidate.1 + candidate.2 - 1 < picked.1) // top left
                || (candidate.0 > picked.0 + picked.2 - 1
                    && candidate.1 > picked.1 + picked.2 - 1)) // bottom right
                && pickable;
            }

            if pickable {
                picked.push(candidate);
            }
            pickable = true;
        }
        picked.sort_by_key(|c| c.0);

        // println!("Picked: {:?}", picked);

        // step 3,4: construct the changes and return patch
        // Note:
        // Before you read the following code, make sure you have read the doc for type Change in patch.rs
        let mut changes: Vec<Change> = vec![];
        if picked.len() < 1 {
            if this_lines_count > 0 {
                changes = if other_lines_count == 0 {
                    // create from new file
                    vec![(0, 0, 0, this_lines_count, vec![], (*this_vec_ref).clone())]
                } else {
                    // changed all.
                    vec![(
                        0,
                        other_lines_count,
                        0,
                        this_lines_count,
                        (*other_vec_ref).clone(),
                        (*this_vec_ref).clone(),
                    )]
                };
                return Patch::from_vec(changes);
            } else {
                panic!("new file should not be empty");
            }
        }

        // 0<-change->picked[0]<-change->picked[1]<-change->...common_last<-change->end

        // case 1:
        //   0 1 2 3 4 5      0 1 2 3 4 5
        // 0                0   \
        // 1\               1    \
        // 2 \              2     \
        // 3  \             3      \
        if picked[0].0 > 0 || picked[0].1 > 0 {
            let mut removed_lines = vec![];
            for idx in 0..picked[0].0 {
                removed_lines.push(other_vec_ref[idx].clone());
            }
            let mut added_lines = vec![];
            for idx in 0..picked[0].1 {
                added_lines.push(this_vec_ref[idx].clone());
            }
            changes.push((0, picked[0].0, 0, picked[1].1, removed_lines, added_lines));
        }
        //  \
        //   \
        //    \ _ _
        //         \
        //          \
        //           \
        for idx in 0..picked.len() - 1 {
            let end_old = picked[idx].0 + picked[idx].2;
            let end_new = picked[idx].1 + picked[idx].2;
            if picked[idx + 1].0 > end_old || picked[idx + 1].1 > end_new {
                let mut removed_lines = vec![];
                for idx in end_old..picked[idx + 1].0 {
                    removed_lines.push(other_vec_ref[idx].clone());
                }
                let mut added_lines = vec![];
                for idx in end_new..picked[idx + 1].1 {
                    added_lines.push(this_vec_ref[idx].clone());
                }
                changes.push((
                    end_old,
                    picked[idx + 1].0 - end_old,
                    end_new,
                    picked[idx + 1].1 - end_new,
                    removed_lines,
                    added_lines,
                ));
            }
        }

        let last_ele = picked[picked.len() - 1];
        let end_old = last_ele.0 + last_ele.2;
        let end_new = last_ele.1 + last_ele.2;
        if other_lines_count > end_old || this_lines_count > end_new {
            let mut removed_lines = vec![];
            for idx in end_old..other_line_num {
                removed_lines.push(other_vec_ref[idx].clone());
            }
            let mut added_lines = vec![];
            for idx in end_new..this_lines_count {
                added_lines.push(this_vec_ref[idx].clone());
            }
            changes.push((
                end_old,
                other_lines_count - end_old,
                end_new,
                this_lines_count - end_new,
                removed_lines,
                added_lines,
            ));
        }
        Patch::from_vec(changes)
    }
}

impl Add<Patch> for Document {
    type Output = Self;
    /// Apply patch to old document.
    /// We can describe this procedure as:
    /// new_doc - old_doc = patch
    /// old_doc + patch = new_doc
    /// version1 + patch1 + patch2 = version3
    fn add(self, patch: Patch) -> Self {
        let mut lines = vec![];
        let old_vec_ref = self.as_vec_ref();
        let patch_vec_ref = patch.as_vec_ref();
        let mut pre_line_num = 0;

        for idx in 0..patch_vec_ref.len() {
            let (
                old_line_num,
                removed_line_count,
                new_line_num,
                added_line_count,
                removed_lines,
                mut added_lines,
            ) = patch_vec_ref[idx].clone();

            if old_line_num > pre_line_num {
                for jdx in pre_line_num..old_line_num {
                    lines.push(old_vec_ref[jdx].clone());
                }
            }
            lines.append(&mut added_lines);

            pre_line_num = old_line_num + removed_line_count;
        }

        for idx in pre_line_num..old_vec_ref.len() {
            lines.push(old_vec_ref[idx].clone());
        }
        Self(lines)
    }
}

#[cfg(test)]
mod tests {
    use self::super::*;
    #[test]
    // #[ignore]
    fn test_from_fs() {
        let doc = Document::from_fs(s!("tests/base"));
        assert_eq!(s!("A\nB\nC\nD\nE\nF\n"), doc.to_string());
    }
    #[test]
    // #[ignore]
    fn test_change_one() {
        let base = Document::from_fs(s!("tests/base"));
        let new = Document::from_fs(s!("tests/change_one"));

        let patch = new.clone() - base.clone();
        let nnew = base.clone() + patch;
        assert_eq!(new.to_string(), nnew.to_string());
    }
    #[test]
    // #[ignore]
    fn test_create_from_blank() {
        let base = Document::from_fs(s!("tests/base"));
        let blank = Document::from_fs(s!("tests/blank"));

        let patch = base.clone() - blank.clone();
        let nnew = blank.clone() + patch;
        assert_eq!(base.to_string(), nnew.to_string());
    }

    #[test]
    // #[ignore]
    fn test_complex() {
        let base = Document::from_fs(s!("tests/base"));
        let new = Document::from_fs(s!("tests/complex"));

        let patch = new.clone() - base.clone();
        let nnew = base.clone() + patch;
        assert_eq!(new.to_string(), nnew.to_string());
    }
    #[test]
    // #[ignore]
    fn test_change_all() {
        let base = Document::from_fs(s!("tests/base"));
        let new = Document::from_fs(s!("tests/change_all"));

        let patch = new.clone() - base.clone();
        let nnew = base.clone() + patch;
        assert_eq!(new.to_string(), nnew.to_string());
    }
}
