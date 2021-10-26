#![allow(unused)]

/// commons: the common lines of two files.
/// 2^16 = 65536, The number of lines in a file is enough.
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

/// the patch file format:
/// line with numbers: start line N.O.(from 0) of the changes in old file, number of line removed, number of line add
/// then lines followed by --: removed line in old file.
/// line followed by ++: added line to the old file.
#[derive(Debug)]
pub struct Patch(u16, u16, u16, Vec<String>, Vec<String>);

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

    /// the patch file format:
    /// line with numbers: start line N.O.(from 0) of the changes in old file, number of line removed, number of line add
    /// then lines followed by --: removed line in old file.
    /// line followed by ++: added line to the old file.
    pub fn gen_patch(&mut self) -> Option<String> {
        match self.get_deltas() {
            Some(deltas) => {
                if deltas.len() == 0 {
                    return None;
                }
                let mut patches = vec![];
                let old_line_iter = self.old_content.lines();
                let new_line_iter = self.new_content.lines();
                for delta in deltas {
                    let removes: Vec<_> = old_line_iter
                        .clone()
                        .skip(delta.0 as usize)
                        .take(delta.2 as usize)
                        .collect();
                    let adds: Vec<_> = new_line_iter
                        .clone()
                        .skip(delta.1 as usize)
                        .take(delta.3 as usize)
                        .collect();

                    patches.push((delta.0, delta.2, delta.3, removes, adds))
                }
                let mut patch_file = String::from("");
                for patch in patches {
                    patch_file = format!("{}\0{},{},{}\n", patch_file, patch.0, patch.1, patch.2);
                    if patch.1 != 0 {
                        for line in patch.3 {
                            patch_file = format!("{}{}\n", patch_file, line);
                        }
                    }
                    if patch.2 != 0 {
                        for line in patch.4 {
                            patch_file = format!("{}{}\n", patch_file, line);
                        }
                    }
                }
                Some(patch_file)
            }
            _ => None,
        }
    }
}

pub fn parse_patch_file(patch_file: String) -> Vec<Patch> {
    let mut patches = vec![];
    let mut start_line_num = 0;
    let mut removed_line_num = 0;
    let mut added_line_num = 0;
    let mut removed_lines = vec![];
    let mut added_lines = vec![];
    for line in patch_file.lines() {
        if line.starts_with("\0") {
            if removed_lines.len() > 0 || added_lines.len() > 0 {
                patches.push(Patch(
                    start_line_num,
                    removed_lines.len() as u16,
                    added_lines.len() as u16,
                    removed_lines.clone(),
                    added_lines.clone(),
                ));
                removed_lines.clear();
                added_lines.clear();
            }
            let nums = line
                .strip_prefix("\0")
                .unwrap()
                .split(",")
                .filter_map(|w| w.parse::<u16>().ok())
                .collect::<Vec<_>>();
            if nums.len() == 3 {
                start_line_num = nums[0];
                removed_line_num = nums[1];
                added_line_num = nums[2];
            } else {
                panic!("failed to parse patch file!");
            }
        } else {
            if removed_line_num > 0 {
                removed_lines.push(String::from(line));
                removed_line_num -= 1;
            } else if added_line_num > 0 {
                added_lines.push(String::from(line));
                added_line_num -= 1;
            } else {
                panic!("unrecognized line");
            }
        }
    }
    if removed_lines.len() > 0 || added_lines.len() > 0 {
        patches.push(Patch(
            start_line_num,
            removed_lines.len() as u16,
            added_lines.len() as u16,
            removed_lines.clone(),
            added_lines.clone(),
        ));
        removed_lines.clear();
        added_lines.clear();
    }
    patches
}

pub fn apply_patch(old_file: String, patch: String) -> String {
    let mut new_file = String::from("");
    let patches = parse_patch_file(patch);
    let mut pre_idx = 0u16;
    let mut old_file_iter = old_file.lines();
    let mut multi_line = String::from("");
    for patch in patches {
        let Patch(start_idx, removed_line_num, added_line_num, _, added_lines) = patch;
        if start_idx > pre_idx {
            multi_line = old_file_iter
                .clone()
                .skip(pre_idx as usize)
                .take((start_idx - pre_idx) as usize)
                .fold(String::new(), |s, line| format!("{}{}\n", s, line));
        }
        if added_line_num > 0 {
            for line in added_lines {
                multi_line = format!("{}{}\n", multi_line, line);
            }
        }
        if multi_line.len() > 0 {
            new_file = format!("{}{}", new_file, multi_line);
            multi_line.clear();
        }
        pre_idx = start_idx + removed_line_num;
    }

    multi_line = old_file_iter
        .clone()
        .skip(pre_idx as usize)
        .fold(String::new(), |s, line| format!("{}{}\n", s, line));
    format!("{}{}", new_file, multi_line)
}

#[cfg(test)]
mod tests {
    use self::super::*;
    #[test]
    // #[ignore]
    fn test_mid() {
        let old = "a\nb\nc\nd\ne\nf\ng\n";
        let new = "a\nb\nx\ny\nz\nd\ne\nf\ng\n";
        let mut differ = Differ::new(old, new);
        if let Some(patch_file) = differ.gen_patch() {
            assert_eq!(
                apply_patch(String::from(old), patch_file),
                String::from(new)
            );
        }
    }

    #[test]
    // #[ignore]
    fn test_remove_start() {
        let old = "a\na\nb\nc\nd\ne\nf";
        let new = "a\nb\nc\nd\ne\nf\n";
        let mut differ = Differ::new(old, new);
        if let Some(patch_file) = differ.gen_patch() {
            assert_eq!(
                apply_patch(String::from(old), patch_file),
                String::from(new)
            );
        }
    }
    #[test]
    // #[ignore]
    fn test_add_tail() {
        let old = "a\nb\nc\nd\n";
        let new = "a\nb\nc\nd\nf\ng\nh\n";
        let mut differ = Differ::new(old, new);
        if let Some(patch_file) = differ.gen_patch() {
            assert_eq!(
                apply_patch(String::from(old), patch_file),
                String::from(new)
            );
        }
    }
    #[test]
    // #[ignore]
    fn test_reverse() {
        let old = "a\nb\nc\nd\ne\nf\n";
        let new = "d\ne\nf\na\nb\nc\n";
        let mut differ = Differ::new(old, new);
        if let Some(patch_file) = differ.gen_patch() {
            assert_eq!(
                apply_patch(String::from(old), patch_file),
                String::from(new)
            );
        }
    }
    #[test]
    // #[ignore]
    fn test_new_file() {
        let old = "";
        let new = "a\nc\nd\n";
        let mut differ = Differ::new(old, new);
        if let Some(patch_file) = differ.gen_patch() {
            assert_eq!(
                apply_patch(String::from(old), patch_file),
                String::from(new)
            );
        }
    }
    #[test]
    // #[ignore]
    fn test_change_all() {
        let old = "a\nb\n";
        let new = "c\nd\n";
        let mut differ = Differ::new(old, new);
        if let Some(patch_file) = differ.gen_patch() {
            assert_eq!(
                apply_patch(String::from(old), patch_file),
                String::from(new)
            );
        }
    }
    #[test]
    fn test_blank_line() {
        let old = "a\nb\n\n\n";
        let new = "c\nd\n\n";
        let mut differ = Differ::new(old, new);
        if let Some(patch_file) = differ.gen_patch() {
            assert_eq!(
                apply_patch(String::from(old), patch_file),
                String::from(new)
            );
        }
    }

    #[test]
    //#[ignore]
    fn test_complicated() {
        let old = "a\nb\nc\nd\ne\nf\ng\nh\n";
        let new = "a\nb\nx\nd\ny\ny\ne\nf\ng\nh\nk\nl\n";
        let mut differ = Differ::new(old, new);
        if let Some(patch_file) = differ.gen_patch() {
            assert_eq!(
                apply_patch(String::from(old), patch_file),
                String::from(new)
            );
        }
    }
}
