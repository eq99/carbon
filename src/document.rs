#![allow(unused)]

use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};

/// The Document struct reprents a real word document.
/// Because operations are line based, We use a Vec<String> to store lines.
#[derive(Debug)]
pub struct Document(Vec<String>);

impl Document {
    /// Create a Document object from file.
    /// Example:
    /// ```ignore
    /// let doc = Document::from_fs(String::from(".carbon/test1.md"));
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
}

#[cfg(test)]
mod tests {
    use self::super::*;
    #[test]
    // #[ignore]
    fn test_from_fs() {
        let doc = Document::from_fs(String::from(".carbon/test1.md"));
        doc.show();
        println!("{:?}", doc);
    }
}
