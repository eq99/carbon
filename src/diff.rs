#![allow(unused)]

/// commons: the common lines of two files.
/// 1<<16 = 65536, The lines file is enough.
/// 0. start line number of common part in old file,
/// 1. start line number of common part in new file,
/// 2. length of the common part
type Common = (u16, u16, u16);

/// Delta: the changes of two files.
/// 0. start line number of Delta in old file,
/// 1. start line number of Delta in new file,
/// 2. line lenth removed in old file,
/// 3. line lenth added in new file
type Delta = (u16, u16, u16, u16);

#[derive(Debug, Default)]
pub struct Differ<'a> {
    old_content: &'a str,
    new_content: &'a str,
    old_total_line: u16,
    new_total_line: u16,
    commons: Option<Vec<Common>>,
    deltas: Option<Vec<Delta>>,
}

impl<'a> Differ<'a> {
    pub fn new(old_content: &'a str, new_content: &'a str) -> Self {
        /// algorithm notes:
        /// 1. compare all sub string of old string with new string.
        /// 2. compare all sub string of new string with old string symmetrically.
        ///
        /// To help you understand the edtor graph,
        /// You can read paper: http://www.xmailserver.org/diff2.pdf
        /// Fig 1. may help you a lot.
        let mut commons = vec![];
        let mut common_len = 0;
        let mut old_line_num = 0;
        let mut new_line_num = 0;
        let mut idx = 0; // inner loop counter
        let mut odx = 0; // outer loop counter
        let mut new_total_line = 0;
        let mut old_total_line = 0;
        let mut new_iter = new_content.lines().peekable();
        while (!new_iter.peek().is_none()) {
            for (old_line, new_line) in old_content.lines().zip(new_iter.clone()) {
                if old_line.len() == new_line.len() && old_line == new_line {
                    if common_len == 0 {
                        old_line_num = idx;
                        new_line_num = idx + odx;
                    }
                    common_len += 1;
                } else {
                    if common_len > 0 {
                        commons.push((old_line_num, new_line_num, common_len));
                        common_len = 0;
                    }
                }
                idx += 1;
            }
            new_iter.next();
            odx += 1;
            idx = 0;
        }
        new_total_line = odx;
        odx = 1;
        idx = 0;
        common_len = 0;
        let mut old_iter = old_content.lines().skip(1).peekable();
        while (!old_iter.peek().is_none()) {
            for (new_line, old_line) in new_content.lines().zip(old_iter.clone()) {
                if old_line.len() == new_line.len() && (old_line == new_line) {
                    if common_len == 0 {
                        old_line_num = odx + idx;
                        new_line_num = idx;
                    }
                    common_len += 1;
                } else {
                    if common_len > 0 {
                        commons.push((old_line_num, new_line_num, common_len));
                        common_len = 0;
                    }
                }
                idx += 1;
            }
            old_iter.next();
            odx += 1;
            idx = 0;
        }
        // fix odx offset
        old_total_line = if old_content.len() > 0 { odx } else { 0 };

        /// After we get all common parts, we have to choose the useful parts.
        /// Here, we use a greedy policy:
        /// choose the longer part first.
        commons.sort_by_key(|c| -(c.2 as i32)); // cast is safe.
        let mut pickes: Vec<Common> = vec![];
        let mut pickable = true;
        if let Some(c) = commons.get(0) {
            pickes.push(c.clone())
        }
        for candidate in commons {
            for picked in &pickes {
                pickable = ((candidate.0 + candidate.2 - 1 < picked.0
                && candidate.1 + candidate.2 - 1 < picked.1) // top left
                || (candidate.0 > picked.0 + picked.2 - 1
                    && candidate.1 > picked.1 + picked.2 - 1)) // bottom right
                && pickable;
            }
            if pickable {
                pickes.push(candidate);
            }
            pickable = true;
        }
        /// After kick of useless part(with small probability),
        pickes.sort_by_key(|c| c.0);
        Self {
            old_content: old_content,
            new_content: new_content,
            old_total_line: old_total_line,
            new_total_line: new_total_line,
            commons: Some(pickes),
            ..Default::default()
        }
    }

    pub fn get_commons(&mut self) -> Option<Vec<Common>> {
        self.commons.clone()
    }

    pub fn get_deltas(&mut self) -> Option<Vec<Delta>> {
        if self.deltas.is_none() {
            let commons = self.commons.clone().unwrap();
            let mut deltas = vec![];

            if commons.len() < 1 {
                if self.new_total_line > 0 {
                    self.deltas = if self.old_total_line == 0 {
                        // see test cases: from_scrath.
                        Some(vec![(0, 0, 0, self.new_total_line)])
                    } else {
                        // changed all.
                        Some(vec![(0, 0, self.old_total_line, self.new_total_line)])
                    };
                    return self.deltas.clone();
                } else {
                    panic!("new file should not be empty");
                }
            }
            /// 0<-delta->commons[0]<-delta->commons[1]<-delta->...common_last<-delta->end
            if commons[0].0 > 0 || commons[0].1 > 0 {
                deltas.push((0, 0, commons[0].0, commons[0].1));
            }

            for idx in 0..commons.len() - 1 {
                let end_old = commons[idx].0 + commons[idx].2;
                let end_new = commons[idx].1 + commons[idx].2;
                if commons[idx + 1].0 > end_old || commons[idx + 1].1 > end_new {
                    deltas.push((
                        end_old,
                        end_new,
                        commons[idx + 1].0 - end_old,
                        commons[idx + 1].1 - end_new,
                    ));
                }
            }

            let last_ele = commons[commons.len() - 1];
            let end_old = last_ele.0 + last_ele.2;
            let end_new = last_ele.1 + last_ele.2;
            if self.old_total_line > end_old || self.new_total_line > end_new {
                deltas.push((
                    end_old,
                    end_new,
                    self.old_total_line - end_old,
                    self.new_total_line - end_new,
                ));
            }

            self.deltas = Some(deltas);
        }
        self.deltas.clone()
    }

    pub fn gen_patch(&mut self) {
        match self.get_deltas() {
            Some(deltas) => {
                for delta in deltas {
                    println!(
                        "@@{},{}: --{} lines, ++{} lines",
                        delta.0, delta.1, delta.2, delta.3
                    );
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use self::super::*;
    #[test]
    fn test_change() {
        let mut differ_mid = Differ::new("a\nb\nc\nd\ne\nf\ng\n", "a\nb\nx\ny\nz\nd\ne\nf\ng\n");
        println!("{:?}", differ_mid.gen_patch());

        let mut remove_start = Differ::new("a\na\nb\nc\nd\ne\nf", "a\nb\nc\nd\ne\nf\n");
        println!("{:?}", remove_start.gen_patch());

        let mut add_tail = Differ::new("a\nb\nc\nd\n", "a\nb\nc\nd\nf\ng\nh\n");
        println!("{:?}", add_tail.gen_patch());

        let mut reverse = Differ::new("a\nb\nc\nd\ne\nf\n", "d\ne\nf\na\nb\nc\n");
        println!("{:?}", reverse.gen_patch());

        let mut more_than_one = Differ::new(
            "a\nb\nc\nd\ne\nf\ng\nh\n",
            "a\nb\nx\nd\ny\ny\ne\nf\ng\nh\nk\nl\n",
        );
        println!("{:?}", more_than_one.gen_patch());

        let mut from_scratch = Differ::new("", "a\nb\nc\n");
        println!("{:?}", from_scratch.gen_patch());

        println!("{:?}", from_scratch);
        let mut change_all = Differ::new("a\nb\n", "c\nd\n");
        println!("{:?}", change_all.gen_patch());
    }
}
