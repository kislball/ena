// Gets at which line character located at given index is.
pub fn get_line(str: &str, at: usize) -> i32 {
    let mut i = 0 as usize;
    let string = String::from(str);
    let mut line = 1;

    loop {
        if i >= at {
            break line;
        }
        let ch = string.chars().nth(i);

        match ch {
            Some(t) => {
                if t == '\n' {
                    line += 1;
                }
            },
            _ => break line
        }

        i += 1;
    }
}