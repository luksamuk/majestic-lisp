use gc::{Finalize, Gc, GcCell, Trace};
use super::MajState;

#[derive(Debug, Trace, Finalize, Clone)]
pub enum Maj {
    Sym(u64),
    Cons {
        car: Gc<Maj>,
        cdr: Gc<Maj>
    },
    Char(char),
    Stream(MajStream),
    Number(MajNumber),
    Vector(MajVector)
}

impl Maj {
    pub fn symbol(state: &mut MajState, str: &str) -> Gc<Maj> {
        Gc::new(Maj::Sym(state.gen_symbol(str)))
    }
}

impl Maj {
    pub fn gensym(state: &mut MajState) -> Gc<Maj> {
        Gc::new(Maj::Sym(state.gen_random_symbol()))
    }
}

impl Maj {
    pub fn cons(car: Gc<Maj>, cdr: Gc<Maj>) -> Gc<Maj> {
        Gc::new(Maj::Cons { car, cdr })
    }
}

impl Maj {
    pub fn character(chr: char) -> Gc<Maj> {
        Gc::new(Maj::Char(chr))
    }
}

impl Maj {
    pub fn string(string: &str) -> Gc<Maj> {
        Gc::new(Maj::Vector(MajVector::Char(
            GcCell::new(String::from(string)))))
    }
}

impl Maj {
    pub fn nil() -> Gc<Maj> {
        Gc::new(Maj::Sym(0))
    }

    pub fn t() -> Gc<Maj> {
        Gc::new(Maj::Sym(1))
    }
}

impl Maj {
    pub fn to_bool(&self) -> bool {
        if let Maj::Sym(idx) = self {
            !(*idx == 0)
        } else {
            true
        }
    }
}

#[macro_export]
macro_rules! maj_list {
    ($x:expr) => (Maj::cons($x, Maj::nil()));

    ($x:expr, $($y:expr),+) => (
	    Maj::cons($x, maj_list!($($y),+))
    )
}

#[macro_export]
macro_rules! maj_dotted_list {
    ($x:expr, $y:expr) => (Maj::cons($x, $y));

    ($x:expr, $y:expr, $($z:expr),+) => (
	    Maj::cons($x, maj_dotted_list!($y, $($z),+))
    )
}

#[derive(Debug, Trace, Finalize, Clone)]
pub enum MajNumber {
    Integer(i64),
    Float(f64),
    Fraction(i64, i64),
    Complex {
        real: Gc<MajNumber>,
        imag: Gc<MajNumber>
    }
}

impl Maj {
    pub fn integer(num: i64) -> Gc<Maj> {
        Gc::new(Maj::Number(MajNumber::Integer(num)))
    }
}

impl Maj {
    pub fn float(num: f64) -> Gc<Maj> {
        Gc::new(Maj::Number(MajNumber::Float(num)))
    }
}

impl Maj {
    pub fn fraction(numer: i64, denom: i64) -> Gc<Maj> {
        Gc::new(Maj::Number(MajNumber::Fraction(numer, denom)))
    }
}

impl Maj {
    pub fn complex(r: Gc<Maj>, i: Gc<Maj>) -> Gc<Maj> {
        use crate::axioms::predicates::maj_complexp;

        let r_complexp = maj_complexp(r.clone()).to_bool();
        let i_complexp = maj_complexp(i.clone()).to_bool();

        if r_complexp || i_complexp {
            panic!("Complex cannot have complex parts");
        }

        if let Maj::Number(rc) = &*r.clone() {
            if let Maj::Number(ic) = &*i.clone() {
                return Gc::new(
                    Maj::Number(MajNumber::Complex {
                        real: Gc::new(rc.clone()),
                        imag: Gc::new(ic.clone())
                    }));
            } else {};
        } else {};

        panic!("Complex cannot have non-numeric parts");
    }
}

impl Maj {
    fn to_maj_number(x: &Maj) -> Option<MajNumber> {
        if let Maj::Number(num) = x {
            Some(num.clone())
        } else {
            None
        }
    }
}

impl Maj {
    pub fn to_integer(&self) -> Option<i64> {
        match Maj::to_maj_number(self) {
            Some(num) => {
                if let MajNumber::Integer(n) = num {
                    return Some(n);
                } else {};
            }
            None => {},
        };
        None
    }

    pub fn to_float(&self) -> Option<f64> {
        match Maj::to_maj_number(self) {
            Some(num) => {
                if let MajNumber::Float(n) = num {
                    return Some(n);
                } else {};
            },
            None => {},
        };
        None
    }
}

impl Maj {
    pub fn to_fraction(&self) -> Option<(i64, i64)> {
        match Maj::to_maj_number(self) {
            Some(num) => {
                if let MajNumber::Fraction(n, d) = num {
                    return Some((n, d));
                } else {};
            },
            None => {},
        };
        None
    }
}

impl MajNumber {
    pub fn into_float(&self) -> f64 {
        match self {
            MajNumber::Integer(n) => {
                *n as f64
            },
            MajNumber::Float(n) => {
                *n
            },
            MajNumber::Fraction(n, d) => {
                *n as f64 / *d as f64
            },
            MajNumber::Complex {
                real: _,
                imag: _
            } => {
                panic!("Cannot convert complex to float");
            }
        }
    }
}

impl Maj {
    pub fn to_forced_float(&self) -> Option<f64> {
        match Maj::to_maj_number(self) {
            Some(num) => Some(num.into_float()),
            None => None,
        }
    }
}

impl Maj {
    pub fn to_complex(&self) -> Option<(f64, f64)> {
        match Maj::to_maj_number(self) {
            Some(num) => {
                let num = num.clone();
                if let MajNumber::Complex {
                    real, imag
                } = &num {
                    let rreal = (*real.clone()).into_float();
                    let rimag = (*imag.clone()).into_float();
                    return Some((rreal, rimag));
                } else {};
            },
            None => {},
        }
        None
    }
}

fn maj_to_string(x: Gc<Maj>) -> Option<String> {
    if let Maj::Vector(vv) = &*x {
        if let MajVector::Char(s) = &vv {
            Some(s.borrow().clone())
        } else {
            None
        }
    } else {
        None
    }
}

impl Maj {
    pub fn stringify(&self) -> Option<String> {
        maj_to_string(Gc::new(self.clone()))
    }
}

impl Maj {
    pub fn to_char(&self) -> Option<char> {
        if let Maj::Char(c) = &*self {
            Some(*c)
        } else {
            None
        }
    }
}

impl Maj {
    pub fn to_raw_sym(&self) -> Option<u64> {
        match *self {
            Maj::Sym(n) => Some(n),
            _ => None
        }
    }
}

#[derive(Trace, Finalize, Debug, Clone, PartialEq)]
pub enum MajStreamDirection {
    In,
    Out
}

#[derive(Trace, Finalize, Debug, Clone, PartialEq)]
pub enum MajStreamType {
    File,
    Stdin,
    Stdout,
    Stderr
}

#[derive(Debug, Trace, Finalize, Clone)]
pub struct MajStream {
    pub direction: MajStreamDirection,
    pub handle:    usize,
    pub stype:     MajStreamType
}

impl MajStream {
    pub fn is_internal(&self) -> bool {
        self.stype != MajStreamType::File
    }
}

impl Maj {
    pub fn stream(state: &mut MajState,
                  file: &str,
                  dir: MajStreamDirection
    ) -> Option<Gc<Maj>> {
        state.make_stream(file, dir)
    }
}

#[derive(Debug, Trace, Finalize, Clone)]
pub enum MajVector {
    Integer(GcCell<Vec<i64>>),
    Float(GcCell<Vec<f64>>),
    Char(GcCell<String>),
    Any(GcCell<Vec<Gc<Maj>>>)
}

#[derive(Debug, PartialEq)]
pub enum MajVectorType {
    Integer,
    Float,
    Char,
    Any
}

impl Maj {
    pub fn vector(vtype: MajVectorType) -> Gc<Maj> {
        Gc::new(Maj::Vector(
            match vtype {
                MajVectorType::Integer => {
                    MajVector::Integer(
                        GcCell::new(Vec::new()))
                },
                MajVectorType::Float => {
                    MajVector::Float(
                        GcCell::new(Vec::new()))
                },
                MajVectorType::Char => {
                    MajVector::Char(
                        GcCell::new(String::new()))
                },
                MajVectorType::Any => {
                    MajVector::Any(
                        GcCell::new(Vec::new()))
                },
            }))
    }
}

impl Maj {
    pub fn vector_integer(vec: Vec<i64>) -> Gc<Maj> {
        Gc::new(Maj::Vector(
            MajVector::Integer(
                GcCell::new(vec.clone()))))
    }
    
    pub fn vector_float(vec: Vec<f64>) -> Gc<Maj> {
        Gc::new(Maj::Vector(
            MajVector::Float(
                GcCell::new(vec.clone()))))
    }

    pub fn vector_any(vec: Vec<Gc<Maj>>) -> Gc<Maj> {
        Gc::new(Maj::Vector(
            MajVector::Any(
                GcCell::new(vec.clone()))))
    }
}

use std::fmt;

impl fmt::Display for Maj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Maj::Sym(idx) => {
                write!(f, "~sym#{}", idx)
            },
            Maj::Cons { car, cdr } => {
                // Temporary cons cell display
                write!(f, "({} . {})", car, cdr)
            },
            Maj::Char(chr) =>
                write!(f, "~char##{}", *chr),
            Maj::Stream(_) => write!(f, "~stream"),
            Maj::Number(num) => write!(f, "{}", num),
            Maj::Vector(_) => write!(f, "~vector"),
        }
    }
}

impl Maj {
    pub fn symbol_name(&self, state: &MajState) -> String {
        if let Maj::Sym(idx) = self {
            state.symbol_name(idx)
        } else {
            // Cannot give a symbol name to something
            // that is not a symbol... but..
            format!("~maj#{:?}", self)
        }
    }
}

impl fmt::Display for MajNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MajNumber::Integer(num) => write!(f, "{}", num),
            MajNumber::Float(num) => {
                use crate::axioms::utils::format_raw_float;
                write!(f, "{}", format_raw_float(*num))
            },
            MajNumber::Fraction(numer, denom) => {
                write!(f, "{}/{}", numer, denom)
            },
            MajNumber::Complex { real, imag } => {
                write!(f, "{}J{}", real, imag)
            }
        }
    }
}

use crate::axioms::MajRawSym;
use crate::axioms::utils::sym_from_raw;

impl Maj {
    pub fn prim() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Prim)
    }

    pub fn lit() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Lit)
    }

    pub fn closure() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Closure)
    }

    pub fn error() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Error)
    }

    pub fn fn_sym() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Fn)
    }

    pub fn ampersand() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Ampersand)
    }

    pub fn apply() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Apply)
    }

    pub fn macro_sym() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Macro)
    }

    pub fn mac() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Mac)
    }

    pub fn quote() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Quote)
    }

    pub fn unquote() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Unquote)
    }

    pub fn unquote_splice() -> Gc<Maj> {
        sym_from_raw(MajRawSym::UnquoteSplice)
    }

    pub fn quasiquote() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Quasiquote)
    }

    pub fn do_sym() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Do)
    }

    pub fn vector_sym() -> Gc<Maj> {
        sym_from_raw(MajRawSym::Vector)
    }
}
