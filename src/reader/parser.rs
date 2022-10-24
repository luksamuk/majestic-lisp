use gc::Gc;
use crate::core::{ MajState, Maj };
use crate::axioms::predicates::maj_errorp;
use crate::axioms::primitives::maj_err;

fn maj_read_one<'a>(
    mut state: &mut MajState,
    tokens: &'a [String]
) -> Result<(Gc<Maj>, &'a [String]), &'static str> {
    if tokens.len() == 0 {
        return Ok((Maj::nil(), tokens));
    }

    let mut tokens = tokens.clone();
    let first = tokens.first().unwrap();
    match first.as_ref() {
        "[" => {
            tokens = &tokens[1..];
            // Empty vector
            if let Some(t) = tokens.first() {
                if t == "]" {
                    use crate::maj_list;
                    return Ok(
                        (maj_list!(
                            Maj::vector_sym()),
                         &tokens[1..]));
                }
            }
            // Keep collecting to build a non-empty vector
            let mut vector_elts = Vec::new();
            loop {
                match maj_read_one(&mut state, tokens) {
                    Ok((obj, slice)) => {
                        tokens = slice;
                        vector_elts.push(obj);
                    },
                    Err(msg) => {
                        return Err(msg);
                    },  
                }
                match tokens.first() {
                    Some(t) => {
                        match t.as_ref() {
                            "]" => {
                                return Ok(
                                    (maj_parser_into_vector(vector_elts),
                                     &tokens[1..]));
                            },
                            // TODO: Check dotted?
                            _ => {},
                        }
                    },
                    None => return Err("Unmatched open bracket"),
                }
            }
        },
        "(" => {
            tokens = &tokens[1..];
            // Empty list
            if let Some(t) = tokens.first() {
                if t == ")" {
                    return Ok((Maj::nil(), &tokens[1..]));
                }
            }
            // Keep collecting to build a non-empty list
            let mut list = Vec::new();
            loop {
                match maj_read_one(&mut state, tokens) {
                    Ok((obj, slice)) => {
                        tokens = slice;
                        list.push(obj);
                    },
                    Err(msg) => {
                        return Err(msg);
                    },
                }
                match tokens.first() {
                    Some(t) => {
                        match t.as_ref() {
                            ")" =>
                                return Ok((maj_parser_into_list(list),
                                           &tokens[1..])),
                            "." => {
                                // Read one more, cons to last
                                // element, begone
                                match maj_read_one(
                                    &mut state,
                                    &tokens[1..]) {
                                    Ok((obj, slice)) => {
                                        if let Some(e) = list.pop() {
                                            tokens = slice;
                                            if tokens.len() == 0 {
                                                return Err("Unexpected EOF when reading dotted pair");
                                            } else if tokens[0] != ")" {
                                                return Err("Invalid usage of dotted element");
                                            }
                                            
                                            let elt = Maj::cons(e, obj);
                                            list.push(elt);
                                            return Ok(
                                                (maj_parser_into_dotted_list(list),
                                                 &tokens[1..]));
                                        } else {
                                            return Err("Unexpected EOF while reading dotted element");
                                        }
                                    },
                                    Err(msg) => {
                                        return Err(msg);
                                    },
                                }
                            },
                            _ => {},
                        }
                    },
                    None => return Err("Unmatched parenthesis"),
                }
            }
        },
        ")" => Err("Unmatched close parenthesis"),
        "]" => Err("Unmatched close brackets"),
        "." => {
            // Best option for dotted stuff is here
            Err("Invalid cons pair notation")
        },
        // TODO: Handle subsequent for the quote and quasiquote
        // related!
        "'" => {
            if tokens.len() <= 1 {
                return Err("Unmatched quote");
            }

            match maj_read_one(&mut state, &tokens[1..]) {
                Ok((obj, slice)) => {
                    Ok((maj_parser_into_list(vec![Maj::quote(), obj]),
                        slice))
                },
                Err(msg) => Err(msg),
            }
        },
        "`" => {
            if tokens.len() <= 1 {
                return Err("Unmatched quasiquote");
            }

            match maj_read_one(&mut state, &tokens[1..]) {
                Ok((obj, slice)) => {
                    Ok((maj_parser_into_list(vec![Maj::quasiquote(), obj]),
                        slice))
                },
                Err(msg) => Err(msg),
            }
        },
        "," => {
            if tokens.len() <= 1 {
                return Err("Unmatched unquote");
            }

            match maj_read_one(&mut state, &tokens[1..]) {
                Ok((obj, slice)) => {
                    Ok((maj_parser_into_list(vec![Maj::unquote(), obj]),
                        slice))
                },
                Err(msg) => Err(msg),
            }
        },
        ",@" => {
            if tokens.len() <= 1 {
                return Err("Unmatched unquote-splice");
            }

            match maj_read_one(&mut state, &tokens[1..]) {
                Ok((obj, slice)) => {
                    Ok((maj_parser_into_list(
                        vec![Maj::unquote_splice(), obj]),
                        slice))
                },
                Err(msg) => Err(msg),
            }
        },
        _ => {
            // Symbols
            if let Some(obj) = maj_parse_character(first.as_ref()) {
                // Character
                Ok((obj, &tokens[1..]))
            } else if let Some(obj) = maj_parse_string(first.as_ref()) {
                // String
                Ok((obj, &tokens[1..]))
            } else if let Some(obj) = maj_parse_number(first.as_ref()) {
                // Number
                Ok((obj, &tokens[1..]))
            } else {
                let token: &str = first.as_ref();
                if (token.len() >= 2) && (&token[0..2] == "#\\") {
                    Err("Unknown character")
                } else {
                    // Ordinary symbol
                    Ok((Maj::symbol(&mut state, first), &tokens[1..]))
                }
            }
        }
    }
}

fn maj_parse_number(token: &str) -> Option<Gc<Maj>> {
    if let Some(pos) = maj_token_once_p(&token.to_uppercase(), 'J') {
        // Test for complex
        let real_token = &token[0..pos];
        let imag_token = &token[(pos+1)..];
        let real = maj_parse_number(real_token);
        let imag = maj_parse_number(imag_token);
        if real.is_some() && imag.is_some() {
            let (real, imag) = (real.unwrap(), imag.unwrap());
            if maj_errorp(real.clone()).to_bool() {
                return Some(real);
            } else if maj_errorp(imag.clone()).to_bool() {
                return Some(imag);
            } else if let Some(num) = imag.to_integer() {
                if num == 0 {
                    return Some(real);
                }
            }
            Some(Maj::complex(real, imag))
        } else {
            None
        }
    } else if let Some(_) = maj_token_once_p(token, '.') {
        // Test for float
        let float: f64 =
            match token.parse() {
                Ok(num) => num,
                Err(_)  => return None,
            };
        Some(Maj::float(float))
    } else if let Some(pos) = maj_token_once_p(token, '/') {
        // Test for fraction
        let numer_token = &token[0..pos];
        let denom_token = &token[(pos+1)..];
        let numer = maj_parse_number(numer_token);
        let denom = maj_parse_number(denom_token);

        if numer.is_some() && denom.is_some() {
            let numer = numer.unwrap().to_integer();
            let denom = denom.unwrap().to_integer();
            if numer.is_some() && denom.is_some() {
                use crate::axioms::utils::simplify_frac_raw;
                if denom.unwrap() == 0 {
                    return Some(maj_err(
                        Maj::string("Division by zero"),
                        Maj::nil()));
                }
                let (numer, denom) =
                    simplify_frac_raw(numer.unwrap(),
                                      denom.unwrap());
                Some(Maj::fraction(numer, denom))
            } else {
                None
            }
        } else {
            None
        }
    } else if maj_numeric_token_p(token) {
        // Test for integer
        let integer: i64 =
                match token.parse() {
                    Ok(num) => num,
                    Err(_)  => return None,
                };
        Some(Maj::integer(integer))
    } else {
        None
    }
}

fn maj_numeric_token_p(token: &str) -> bool {
    if token.len() == 0 {
        return false;
    }
    let token = if token.chars().nth(0).unwrap() == '-' {
        &token[1..]
    } else { token };
    token.chars().all(|x| x.is_digit(10))
}

fn maj_token_once_p(token: &str, ch: char) -> Option<usize> {
    let mut occurp = 0;
    let mut num    = 0;
    for (i, c) in token.chars().enumerate() {
        if c == ch {
            occurp = i;
            num += 1;
        }
    }
    if num == 1 {
        Some(occurp)
    } else {
        None
    }
}

fn maj_parse_string(token: &str) -> Option<Gc<Maj>> {
    if (token.len() < 2)
        || (token.chars().nth(0).unwrap() != '"')
        || (token.chars().last().unwrap() != '"') {
            None
        } else {
            let mut buffer = String::new();
            let mut ignore_next = false;
            for i in 1..(token.len()-1) {
                if ignore_next {
                    ignore_next = false;
                } else {
                    match token.chars().nth(i).unwrap() {
                        '\\' => {
                            match token.chars().nth(i+1).unwrap() {
                                'n' => {
                                    buffer.push('\n');
                                    ignore_next = true;
                                },
                                't' => {
                                    buffer.push('\t');
                                    ignore_next = true;
                                },
                                c => {
                                    buffer.push(c);
                                    ignore_next = true;
                                },
                            };
                        },
                        c    => buffer.push(c),
                    }
                }
            }
            Some(Maj::string(buffer.as_ref()))
        }
}

fn maj_parse_character(token: &str) -> Option<Gc<Maj>> {
    if (token.len() <= 2) || (&token[0..2] != "#\\") {
        None
    } else {
        let chr = &token[2..];
        Some(Maj::character(match chr {
            "␇" | "bel" => '\x07',
            "newline" => '\n',
            "tab" => '\t',
            // Those should not be needed!
            "space"    => ' ',
            "lparen"   => '(',
            "rparen"   => ')',
            "lbracket" => '[',
            "rbracket" => ']',
            c => {
                if c.len() == 1 {
                    c.chars().nth(0).unwrap()
                } else {
                    return None;
                }
            },
        }))
    }
}

pub fn maj_parse(
    mut state: &mut MajState, tokens: Vec<String>
) -> Result<Gc<Maj>, &str> {
    let mut list = Vec::new();
    let mut tokens = &tokens[..];
    while tokens.len() > 0 {
        match maj_read_one(&mut state, tokens) {
            Ok((expr, slice)) => {
                tokens = slice;
                list.push(expr);
            },
            Err(msg) => return Err(msg),
        }
    }
    Ok(maj_parser_into_list(list))
}

fn maj_parser_into_list(list: Vec<Gc<Maj>>) -> Gc<Maj> {
    if list.len() == 0 {
        Maj::nil()
    } else {
        let mut expr = Maj::nil();
        for elt in list.iter().rev() {
            expr = Maj::cons(elt.clone(), expr);
        }
        expr
    }
}

fn maj_parser_into_dotted_list(list: Vec<Gc<Maj>>) -> Gc<Maj> {
    match list.len() {
        0 => Maj::nil(),
        _ => {
            let mut list = list.clone();
            let mut expr = list.pop().unwrap();
            for elt in list.iter().rev() {
                expr = Maj::cons(elt.clone(), expr);
            }
            expr
        }
    }
}

fn maj_parser_into_vector(elts: Vec<Gc<Maj>>) -> Gc<Maj> {
    if elts.len() == 0 {
        Maj::nil()
    } else {
        let mut expr = Maj::nil();
        for elt in elts.iter().rev() {
            expr = Maj::cons(elt.clone(), expr);
        }
        Maj::cons(Maj::vector_sym(), expr)
    }
}
