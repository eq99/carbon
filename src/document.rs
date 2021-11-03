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
#[derive(Debug)]
pub struct Document(Vec<String>);

impl Document {
    /// Create a Document object from file.
    /// Example:
    /// ```ignore
    /// let doc = Document::from_fs(s!(".carbon/test1.md"));
    /// println!("{:?}", doc);
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
    /// 0. A
    /// 1. B
    /// 2. C
    /// 3. D
    /// 4. E
    pub fn show(&self) {
        let Document(lines) = self;
        for line in lines.iter().enumerate() {
            println!("{}. {}", line.0, line.1);
        }
    }

    pub fn as_vec_ref(&self) -> &Vec<String> {
        let Self(vec) = self;
        vec
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
    /// Step 2: Choose the right common parts greedily by common_len.
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
        let other_vec_ref = self.as_vec_ref();

        // step 1: Find common parts
        let mut commons = vec![];
        let mut common_len = 0;
        let mut this_line_num = 0;
        let mut other_line_num = 0;
        let mut other_total_num = this_vec_ref.len();
        let mut this_total_num = other_vec_ref.len();
        // diagonal and upper
        for i in 0..other_total_num {
            for j in 0..this_total_num {
                if i + j < other_total_num
                    && (this_vec_ref[j].len() == other_vec_ref[i + j].len()
                        && this_vec_ref[j] == other_vec_ref[i + j])
                {
                    if common_len == 0 {
                        this_line_num = i + j;
                        other_line_num = j;
                    }
                    common_len += 1;
                } else {
                    if common_len > 0 {
                        commons.push((other_line_num, this_line_num, common_len));
                        common_len = 0;
                    }
                }
            }
        }

        // below diagonal
        for i in 1..this_total_num {
            for j in 0..other_total_num {
                if i + j < this_total_num
                    && (this_vec_ref[i + j].len() == other_vec_ref[j].len()
                        && this_vec_ref[i + j] == other_vec_ref[j])
                {
                    if common_len == 0 {
                        this_line_num = j;
                        other_line_num = i + j;
                    }
                    common_len += 1;
                } else {
                    if common_len > 0 {
                        commons.push((other_line_num, this_line_num, common_len));
                        common_len = 0;
                    }
                }
            }
        }

        // Step 2: choose the right parts
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

        // step 3,4: construct the changes and return patch
        let mut changes: Vec<Change> = vec![];
        if picked.len() < 1 {
            if this_total_num > 0 {
                changes = if other_total_num == 0 {
                    // create from new file
                    vec![(0, 0, 0, this_total_num, vec![], (*this_vec_ref).clone())]
                } else {
                    // changed all.
                    vec![(
                        0,
                        other_total_num,
                        0,
                        this_total_num,
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
        if picked[0].0 > 0 || picked[0].2 > 0 {
            let mut removed_lines = vec![];
            for idx in 0..picked[0].0 {
                removed_lines.push(other_vec_ref[idx].clone());
            }
            let mut added_lines = vec![];
            for idx in 0..picked[0].2 {
                added_lines.push(this_vec_ref[idx].clone());
            }
            changes.push((0, picked[0].0, 0, picked[0].2, removed_lines, added_lines));
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
            if picked[idx + 1].0 > end_old || picked[idx + 1].2 > end_new {
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
        if other_total_num > end_old || this_total_num > end_new {
            let mut removed_lines = vec![];
            for idx in end_old..other_line_num {
                removed_lines.push(other_vec_ref[idx].clone());
            }
            let mut added_lines = vec![];
            for idx in end_new..this_total_num {
                added_lines.push(this_vec_ref[idx].clone());
            }
            changes.push((
                end_old,
                other_total_num - end_old,
                end_new,
                this_total_num - end_new,
                removed_lines,
                added_lines,
            ));
        }
        Patch::from_vec(changes)
    }
}

impl Add<Patch> for Document {
    type Output = Self;
    /// 
    fn add(self, pathc: Patch) -> Self {
        let lines = vec![];
        Self(lines)
    }
}

#[cfg(test)]
mod tests {
    use self::super::*;
    #[test]
    // #[ignore]
    fn test_from_fs() {
        let doc = Document::from_fs(s!(".carbon/test1.md"));
        doc.show();
        println!("{:?}", doc);
    }
}
