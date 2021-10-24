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
    commons: Option<Vec<Common>>,
    deltas: Option<Vec<Delta>>,
}

impl<'a> Differ<'a> {
    pub fn new(old_content: &'a str, new_content: &'a str) -> Self {
        let mut commons = vec![];
        let mut common_len = 0;
        let mut old_line_num = 0;
        let mut new_line_num = 0;
        let mut idx = 0; // inner loop counter
        let mut odx = 0; // outer loop counter
        let mut new_iter = new_content.chars().peekable();
        while (!new_iter.peek().is_none()) {
            for (old_ch, new_ch) in old_content.chars().zip(new_iter.clone()) {
                if old_ch == new_ch {
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

        odx = 1;
        idx = 0;
        common_len = 0;
        let mut old_iter = old_content.chars().skip(1).peekable();
        while (!old_iter.peek().is_none()) {
            for (new_ch, old_ch) in new_content.chars().zip(old_iter.clone()) {
                if old_ch == new_ch {
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

        commons.sort_by_key(|c| -(c.2 as i32)); // cast is safe.

        // pick the good cases
        let mut pickes: Vec<Common> = vec![];
        let mut pickable = true;
        if let Some(c) = commons.get(0) {
            pickes.push(c.clone())
        }
        for candidate in commons {
            for picked in &pickes {
                pickable = ((candidate.0 + candidate.2 - 1 < picked.0
                && candidate.1 + candidate.2 - 1 < picked.1) //top left
                || (candidate.0 > picked.0 + picked.2 - 1
                    && candidate.1 > picked.1 + picked.2 - 1)) // bottom right
                && pickable;
            }
            if pickable {
                pickes.push(candidate);
            }
            pickable = true;
        }

        pickes.sort_by_key(|c| c.0);
        Self {
            old_content: old_content,
            new_content: new_content,
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
            let old_content_len = self.old_content.len() as u16;
            let new_content_len = self.new_content.len() as u16;

            if commons.len() < 1 {
                // see test cases:
                // from_scrath.
                if old_content_len == 0 && new_content_len > 0 {
                    self.deltas = Some(vec![(0, 0, 0, new_content_len)]);
                    return self.deltas.clone();
                } else {
                    panic!("file should not be empty");
                }
            }

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
            if old_content_len > end_old || new_content_len > end_new {
                deltas.push((
                    end_old,
                    end_new,
                    old_content_len - end_old,
                    new_content_len - end_new,
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
        let mut differ_mid = Differ::new("abcdefg", "abxyzdefg");
        println!("{:?}", differ_mid.gen_patch());

        let mut remove_start = Differ::new("aabcdef", "abcdef");
        println!("{:?}", remove_start.gen_patch());

        let mut add_tail = Differ::new("abcd", "abcdefgh");
        println!("{:?}", add_tail.gen_patch());

        let mut reverse = Differ::new("abcdef", "defabc");
        println!("{:?}", reverse.gen_patch());

        let mut more_than_one = Differ::new("abcdefgh", "axcdyyefghkl");
        println!("{:?}", more_than_one.gen_patch());

        let mut from_scratch = Differ::new("", "abc");
        from_scratch.gen_patch();
    }
}
