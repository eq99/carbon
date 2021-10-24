#![allow(unused)]

#[derive(Debug)]
struct Common(i32, i32, i32);

pub fn common(old_content: &str, new_content: &str) {
    let mut commons: Vec<Common> = vec![];

    let mut common_len = 0i32;
    // cdx is the start index of common substring in old content.
    let mut old_line_no = 0i32;
    let mut new_line_no = 0i32;
    let mut idx = 0i32; // inner loop counter
    let mut odx = 0i32; // outer loop counter
    let mut new_iter = new_content.chars().peekable();
    while (!new_iter.peek().is_none()) {
        for (old_ch, new_ch) in old_content.chars().zip(new_iter.clone()) {
            if old_ch == new_ch {
                if common_len == 0 {
                    old_line_no = idx;
                    new_line_no = idx + odx;
                }
                common_len += 1;
            } else {
                if common_len > 0 {
                    commons.push(Common(old_line_no, new_line_no, common_len));
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
                    old_line_no = odx + idx;
                    new_line_no = idx;
                }
                common_len += 1;
            } else {
                if common_len > 0 {
                    commons.push(Common(old_line_no, new_line_no, common_len));
                    common_len = 0;
                }
            }
            idx += 1;
        }
        old_iter.next();
        odx += 1;
        idx = 0;
    }
    commons.sort_by_key(|c| -c.2);

    let mut picked = vec![];
    let mut pickable = true;
    if let Some(c) = commons.get(0) {
        picked.push(c)
    }
    for candidate in &commons {
        for picked_c in &picked {
            pickable = ((candidate.0 + candidate.2 - 1 < picked_c.0
                && candidate.1 + candidate.2 - 1 < picked_c.1)//top left
                || (candidate.0 > picked_c.0 + picked_c.2 - 1
                    && candidate.1 > picked_c.1 + picked_c.2 - 1)) // bottom right
                && pickable;
        }
        if pickable {
            picked.push(candidate);
        }
        pickable = true;
    }
    println!("{:?}", commons);
    picked.sort_by_key(|c| c.0);
    println!("{:?}", picked);

    let mut old_remove = vec![];
    let mut new_add = vec![];
    let old_content_len = old_content.len();
    let new_content_len = new_content.len();

    if picked.len() < 1 {
        panic!("file not changed");
    }
    if picked[0].0 > 0 {
        old_remove.push((0, picked[0].0));
    }
    if picked[0].1 > 0 {
        new_add.push((0, picked[0].1));
    }
    for idx in 0..picked.len() - 1 {
        if picked[idx + 1].0 > picked[idx].0 + picked[idx].2 {
            old_remove.push((
                picked[idx].0 + picked[idx].2,
                picked[idx + 1].0 - picked[idx].0 - picked[idx].2,
            ));
        }
        if picked[idx + 1].1 > picked[idx].1 + picked[idx].2 {
            new_add.push((
                picked[idx].1 + picked[idx].2,
                picked[idx + 1].1 - picked[idx].1 - picked[idx].2,
            ));
        }
    }
    let last_ele = picked[picked.len() - 1];
    if old_content_len > (last_ele.0 + last_ele.2) as usize {
        old_remove.push((
            last_ele.0 + last_ele.2,
            old_content_len as i32 - (last_ele.0 + last_ele.2),
        ));
    }
    if new_content_len > (last_ele.1 + last_ele.2) as usize {
        new_add.push((
            last_ele.1 + last_ele.2,
            old_content_len as i32 - (last_ele.1 + last_ele.2),
        ));
    }

    for removed in old_remove {
        println!("--{:?},{:?}", removed.0, removed.1);
    }
    for added in new_add {
        println!("++{:?},{:?}", added.0, added.1);
    }
}

#[cfg(test)]
mod tests {
    use self::super::*;
    #[test]
    fn test_common() {
        common("abcdefg", "abxyzdefg"); // c--->xyz
        common("abcdefg", "abzdefxg"); // c--->z, +x
    }
}
