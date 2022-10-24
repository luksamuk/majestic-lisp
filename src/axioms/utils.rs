use gc::Gc;
use crate::core::Maj;
use crate::maj_list;
use crate::axioms::MajRawSym;

pub const STACK_RED_ZONE: usize      = 100 * 1024;      // 100KB
pub const STACK_PER_RECURSION: usize = 9 * 1024 * 1024; // 9MB

pub fn gcd(a: i64, b: i64) -> i64 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let ratio = a % b;
        a = b;
        b = ratio;
    }
    a
}

pub fn simplify_frac_raw(numer: i64, denom: i64) -> (i64, i64) {
    if denom == 0 {
        // Division by zero is illegal and, if it came
        // to this part of the program, we should panic
        panic!("Division by zero on fraction simplification");
    }
    let gcd = gcd(numer, denom);
    let (mut numer, mut denom) = (numer / gcd, denom / gcd);
    if (denom < 0) && (numer > 0) {
        numer *= -1;
        denom *= -1;
    }
    (numer, denom)
}

pub fn simplify_frac(x: Gc<Maj>) -> Result<Gc<Maj>, Gc<Maj>> {
    use crate::axioms::predicates::maj_fractionp;
    use crate::axioms::primitives::{
        maj_numer,
        maj_denom,
        maj_err,
    };
    if !maj_fractionp(x.clone()).to_bool() {
        Err(maj_err(Maj::string("{} is not a fraction"),
                    maj_list!(x)))
    } else {
        let numer = maj_numer(x.clone()).to_integer().unwrap();
        let denom = maj_denom(x.clone()).to_integer().unwrap();
        if denom == 0 {
            Err(maj_err(Maj::string("Division by zero"),
                        Maj::nil()))
        } else {
            let (numer, denom) = simplify_frac_raw(numer, denom);
            Ok(Maj::fraction(numer, denom))
        }
    }
}

pub fn simplify_frac_coerce(x: Gc<Maj>) -> Result<Gc<Maj>, Gc<Maj>> {
    use crate::axioms::primitives::{maj_numer, maj_denom };
    let frac = simplify_frac(x)?;
    Ok(if maj_denom(frac.clone()).to_integer().unwrap() == 1 {
        maj_numer(frac)
    } else {
        frac
    })
}

pub fn sym_from_raw(raw: MajRawSym) -> Gc<Maj> {
    Gc::new(Maj::Sym(raw as u64))
}

pub fn truncate_format(text: String, max: usize) -> String {
    if max < 4 || text.len() <= max {
        text
    } else {
        let slice = &text[..(max - 4)];
        let should_close =
            slice.chars().nth(0).unwrap() == '(';
        format!("{}{}{}",
                slice,
                "...",
                if should_close { ")" } else { "" })
    }
}

pub fn format_raw_float(num: f64) -> String {
    let mut buffer = format!("{}", num);
    if buffer.find('.').is_none() {
        buffer.push('.');
        buffer.push('0');
    }
    buffer
}
