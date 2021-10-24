#![allow(unused)]

#[derive(Debug)]
struct Common(u32, String);

pub fn common(old_content: &str, new_content: &str) {
    let mut commons: Vec<Common> = vec![];

    let mut sub = String::new();
    // cdx is the start index of common substring in old content.
    let mut cdx = 0u32;
    let mut idx = 0u32; // inner loop counter
    let mut odx = 0u32; // outer loop counter
    let mut new_iter = new_content.chars().peekable();
    while (!new_iter.peek().is_none()) {
        for (old_ch, new_ch) in old_content.chars().zip(new_iter.clone()) {
            if old_ch == new_ch {
                if sub.is_empty() {
                    cdx = idx;
                }
                sub.push(old_ch);
            } else {
                if sub.len() > 0 {
                    commons.push(Common(cdx, sub.clone()));
                    sub.clear();
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
    let mut old_iter = old_content.chars().skip(1).peekable();
    while (!old_iter.peek().is_none()) {
        for (new_ch, old_ch) in new_content.chars().zip(old_iter.clone()) {
            if old_ch == new_ch {
                if sub.is_empty() {
                    cdx = odx + idx;
                }
                sub.push(old_ch);
            } else {
                if sub.len() > 0 {
                    commons.push(Common(cdx, sub.clone()));
                    sub.clear();
                }
            }
            idx += 1;
        }
        old_iter.next();
        odx += 1;
        idx = 0;
    }
    println!("{:?}", commons);
}

#[cfg(test)]
mod tests {
    use self::super::*;
    #[test]
    fn test_common() {
        common("apkleses", "appleses");
        common("cappleses", "caplekses");
    }
}
