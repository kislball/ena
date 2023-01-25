// Gets at which line character located at given index is.
pub fn get_line(str: &str, at: usize) -> (usize, usize) {
    let mut i = 0_usize;
    let string = String::from(str);
    let mut line: usize = 1;
    let mut within_line: usize = 1;

    loop {
        if i >= at {
            break (line, within_line);
        }
        let ch = string.chars().nth(i);

        match ch {
            Some(t) => {
                if t == '\n' {
                    line += 1;
                    within_line = 1;
                } else {
                    within_line += 1;
                }
            }
            _ => break (line, within_line),
        }

        i += 1;
    }
}
