pub fn maj_tokenize(
    text: &str
) -> Result<Vec<String>, (i64, &'static str)> {
    let mut tokens = Vec::new();
    let mut buffer = String::new();
    let mut ignore_count = 0;
    let mut line = 1;

    for (i, c) in text.chars().enumerate() {
        if ignore_count > 0 {
            ignore_count -= 1;
        } else {
            match c {
                // Read macros
                ';' => {
                    let mut count = 1;
                    loop {
                        let c = text.chars().nth(i + count);
                        match c {
                            Some('\n') | None => {
                                break;
                            },
                            _ => {
                                count += 1;
                            }
                        }
                    }
                    ignore_count = count;
                },
                '\"' => {
                    // Keep fetching string
                    buffer.push('"');
                    let mut count = 1;
                    let mut ignore_next = false;
                    loop {
                        let c = text.chars().nth(i + count);
                        if ignore_next {
                            count += 2;
                            ignore_next = false;
                        } else {
                            match c {
                                Some('"') => {
                                    buffer.push('"');
                                    tokens.push(buffer.clone());
                                    buffer = String::new();
                                    ignore_count = count;
                                    break;
                                },
                                Some('\\') => {
                                    buffer.push('\\');
                                    if let Some(next) = text.chars().nth(i + count + 1) {
                                        buffer.push(next);
                                    } else {
                                        return Err((line, "Unexpected EOF while reading escaped character on string constant"));
                                    }
                                    ignore_next = true;
                                },
                                Some(c) => {
                                    count += 1;
                                    buffer.push(c);
                                },
                                None => {
                                    return Err((line, "Unexpected EOF while reading string constant"));
                                },
                            }
                        }
                    }
                },
                '#' => {
                    buffer.push('#');
                    match text.chars().nth(i + 1) {
                        Some('\\') => {
                            buffer.push('\\');
                            if let Some(c) = text.chars().nth(i + 2) {
                                buffer.push(c);
                            } else {
                                return Err((line, "Unexpected EOF while reading character constant"));
                            }
                            // Keep fetching until white space or EOF
                            let mut count = 2;
                            loop {
                                let c = text.chars().nth(i + 1 + count);
                                match c {
                                    // Every delimiter on  ):
                                    // This needs a clever way to never include
                                    // delimiters.
                                    Some(' ')  |
                                    Some('\n') |
                                    Some('\t') |
                                    Some(')')  |
                                    Some('(') |
                                    Some(']') |
                                    Some('[') |
                                    Some('"') |
                                    None => {
                                        if buffer.len() > 2 {
                                            tokens.push(buffer.clone());
                                            buffer = String::new();
                                            ignore_count = count;
                                            break;
                                        } else {
                                            return Err((line, "Unexpected end of character constant"));
                                        }
                                    },
                                    Some(c) => {
                                        buffer.push(c);
                                        count += 1;
                                    },
                                }
                            }
                        },
                        _ => return Err((line, "Unexpected character while reading character constant")),
                    }
                    // Parse character
                    //unimplemented!("Character tokenization");
                },
                '\'' => {
                    if buffer != "" {
                        tokens.push(buffer.clone());
                        buffer = String::new();
                    }
                    tokens.push(String::from("'"));
                },
                '`' => {
                    if buffer != "" {
                        tokens.push(buffer.clone());
                        buffer = String::new();
                    }
                    tokens.push(String::from("`"));
                }
                ',' => {
                    if buffer != "" {
                        tokens.push(buffer.clone());
                        buffer = String::new();
                    }
                    let nextchar = text.chars().nth(i + 1);
                    tokens.push(String::from(
                        match nextchar {
                            Some(c) => {
                                if c == '@' {
                                    ignore_count = 1;
                                    ",@"
                                } else { "," }
                            },
                            None => ",",
                        }));
                },
                '@' => {
                    // Syntax error: @ alone
                    return Err((line, "'@' should be preceeded by ','"));
                },
                '(' => {
                    if buffer != "" {
                        tokens.push(buffer.clone());
                        buffer = String::new();
                    }
                    tokens.push(String::from("("));
                },
                ')' => {
                    if buffer != "" {
                        tokens.push(buffer.clone());
                        buffer = String::new();
                    }
                    tokens.push(String::from(")"));
                },
                '[' => {
                    if buffer != "" {
                        tokens.push(buffer.clone());
                        buffer = String::new();
                    }
                    tokens.push(String::from("["));
                },
                ']' => {
                    if buffer != "" {
                        tokens.push(buffer.clone());
                        buffer = String::new();
                    }
                    tokens.push(String::from("]"));
                },
                
                // Delimiters
                ' ' | '\n' | '\t' => {
                    if buffer != "" {
                        tokens.push(buffer.clone());
                        buffer = String::new();
                    }
                    if c == '\n' {
                        line += 1;
                    }
                },

                // Anything else is pushed
                _ => buffer.push(c),
            }
        }
    }
    if buffer != "" {
        tokens.push(buffer.clone());
    }
    Ok(tokens)
}

pub fn maj_tokenize_file<'a>(
    filename: &str,
    mut buffer: &'a mut String
) -> Result<Vec<String>, (i64, &'static str)> {
    use std::fs::File;
    use std::io::Read;
    match File::open(filename) {
        Ok(mut file) => {
            match file.read_to_string(&mut buffer) {
                Ok(_) => {
                    // Remove shebang line
                    if buffer.len() >= 2 && &buffer[0..2] == "#!" {
                        *buffer = buffer.replacen("#!", ";;", 1);
                    }
                    maj_tokenize(buffer.as_ref())
                },
                Err(_) => Err((0, "Cannot read file")),
            }
        },
        Err(_) => Err((0, "Cannot open file")),
    }
}
