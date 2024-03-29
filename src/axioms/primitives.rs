//use std::fs::File;
use gc::Gc;
use crate::core::{
    Maj,
    MajState
};
use super::predicates::{
    maj_eq,
    maj_nilp,
    maj_consp,
    maj_errorp,
    maj_stringp,
    maj_numberp,
    maj_vectorp,
    maj_proper_list_p
};
use crate::{ maj_list, maj_destructure_args };
use super::MajRawSym;
use num_traits::FromPrimitive;
use crate::axioms::utils::{ simplify_frac, simplify_frac_coerce };
use crate::core::types::MajVectorType;

pub fn maj_cons(x: Gc<Maj>, y: Gc<Maj>) -> Gc<Maj> {
    if maj_errorp(x.clone()).to_bool() {
        x
    } else if maj_errorp(y.clone()).to_bool() {
        y
    } else {
        Maj::cons(x, y)
    }
}

pub fn maj_car(x: Gc<Maj>) -> Gc<Maj> {
    match &*x.clone() {
        Maj::Sym(_) => {
            if maj_nilp(x.clone()).to_bool() {
                return Maj::nil();
            }
        },
        Maj::Cons { car, cdr: _ } => {
            return car.clone();
        },
        _ => {}
    }
    maj_err(Maj::string("{} is not a cons cell"),
            maj_list!(x))
}

pub fn maj_cdr(x: Gc<Maj>) -> Gc<Maj> {
    match &*x.clone() {
        Maj::Sym(_) => {
            if maj_nilp(x.clone()).to_bool() {
                return Maj::nil();
            }
        },
        Maj::Cons { car: _, cdr } => {
            return cdr.clone();
        },
        _ => {}
    }
    maj_err(Maj::string("{} is not a cons cell"),
            maj_list!(x))
}

pub fn maj_copy(x: Gc<Maj>) -> Gc<Maj> {
    match &*x.clone() {
        Maj::Cons { car, cdr } => {
            Maj::cons(car.clone(), cdr.clone())
        }
        _ => maj_err(
            Maj::string("{} is not a cons cell"),
            maj_list!(x))
    }
}

pub fn maj_length(x: Gc<Maj>) -> Gc<Maj> {
    if maj_nilp(x.clone()).to_bool() {
        Maj::integer(0)
    } else if !maj_consp(x.clone()).to_bool() {
        maj_err(
            Maj::string("{} is not a proper list"),
            maj_list!(x.clone()))
    } else {
        let mut itr    = x.clone();
        let mut length = 0;
        while maj_consp(itr.clone()).to_bool() {
            length += 1;
            itr = maj_cdr(itr);
        }
        Maj::integer(length)
    }
}

fn maj_depth_helper(x: Gc<Maj>) -> i64 {
    use std::cmp;
    if !maj_consp(x.clone()).to_bool() {
        0
    } else {
        1 + cmp::max(maj_depth_helper(maj_car(x.clone())),
                     maj_depth_helper(maj_cdr(x)))
    }
}

pub fn maj_depth(x: Gc<Maj>) -> Gc<Maj> {
    use crate::axioms::predicates::maj_atomp;
    if maj_nilp(x.clone()).to_bool() {
        Maj::integer(0)
    } else if maj_atomp(x.clone()).to_bool() {
        maj_err(Maj::string(
            "{} is an atom"), maj_list!(x))
    } else {
        Maj::integer(maj_depth_helper(x))
    }
}

pub fn maj_type(mut state: &mut MajState, x: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajNumber;
    Maj::symbol(
        &mut state,
        match &*x {
            Maj::Sym(_)              =>"symbol",
            Maj::Cons {
                car: _, cdr: _
            }                        => "cons",
            Maj::Stream(_)           => "stream",
            Maj::Char(_)             => "char",
            Maj::Number(num) => {
                match num.clone() {
                    MajNumber::Integer(_)     => "integer",
                    MajNumber::Float(_)       => "float",
                    MajNumber::Fraction(_, _) => "fraction",
                    MajNumber::Complex {
                        real: _, imag: _
                    }                         => "complex",
                }
            },
            Maj::Vector(_)            => "vector",
        })
}

pub fn maj_intern(mut state: &mut MajState, x: Gc<Maj>) -> Gc<Maj> {
    match x.clone().stringify() {
        Some(string) => {
            if string == "" {
                Maj::nil()
            } else {
                Maj::symbol(&mut state, &string)
            }
        },
        None => {
            maj_err(Maj::string("{} is not a string"),
                    maj_list!(x))
        },
    }
}

pub fn maj_name(state: &MajState, x: Gc<Maj>) -> Gc<Maj> {
    use crate::printing::maj_format;
    if let Maj::Sym(_) = &*x.clone() {
        Maj::string(&maj_format(&state, x))
    } else {
        maj_err(Maj::string("{} is not a symbol"),
                maj_list!(x.clone()))
    }
}

pub fn maj_get_environment(
    mut state: &mut MajState,
    type_sym: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    if maj_eq(type_sym.clone(),
              Maj::symbol(&mut state, "lexical"))
        .to_bool()
    {
        env
    } else if maj_eq(type_sym.clone(),
                     Maj::symbol(&mut state, "global"))
        .to_bool()
    {
        state.get_global_env()
    } else {
        maj_err(
            Maj::string("Unknown environment type {}"),
            maj_list!(type_sym))
    }
}

pub fn maj_coin() -> Gc<Maj> {
    use rand::random;

    if random() {
        Maj::t()
    } else {
        Maj::nil()
    }
}

pub fn maj_sys(com: Gc<Maj>, args: Gc<Maj>) -> Gc<Maj> {
    use std::process::Command;

    if !maj_stringp(com.clone()).to_bool() {
        return maj_err(
            Maj::string("{} is not a string"),
            maj_list!(com));
    }

    let mut comm = Command::new(com.stringify().unwrap());

    let mut itr = args.clone();
    while !maj_nilp(itr.clone()).to_bool() {
        if let Maj::Cons { car, cdr } = &*itr.clone() {
            if !maj_stringp(car.clone()).to_bool() {
                return maj_err(
                    Maj::string("{} is not a string"),
                    maj_list!(car.clone()));
            } else {
                let _ = comm.arg(car.clone()
                                 .stringify()
                                 .unwrap());
                itr = cdr.clone();
            }
        } else {};
    }

    let result = match comm.status() {
        Ok(res) => res.code(),
        Err(_)  => None,
    };

    match result {
        Some(code) => Maj::integer(code as i64),
        None => maj_err(
            Maj::string("Error executing command {} {}"),
            maj_list!(com, args)),
    }
}

pub fn maj_format_prim(
    state: &MajState,
    fmt: Gc<Maj>,
    rest: Gc<Maj>
) -> Gc<Maj> {
    if let Some(rfmt) = fmt.stringify() {
        let mut buffer = String::new();
        let mut expect_closebracket = false;
        let mut gulp_next = false;
        let mut curr_arg = rest.clone();
        for c in rfmt.chars() {
            if gulp_next {
                buffer.push(c);
                gulp_next = false;
            } else if expect_closebracket {
                if c == '}' {
                    use crate::printing::maj_format;
                    expect_closebracket = false;
                    if maj_nilp(curr_arg.clone()).to_bool() {
                        return maj_err(
                            Maj::string("Missing arguments on format"),
                            Maj::nil());
                    }
                    let cafirst = maj_car(curr_arg.clone());
                    let mut formatted =
                        if let Maj::Char(c) = &*cafirst.clone() {
                            String::from(format!("{}", c))
                        } else {
                            maj_format(&state, cafirst)
                        };
                    let len = formatted.len();
                    if (len >= 2)
                        && (formatted.chars().nth(0).unwrap() == '"')
                        && (formatted.chars().last().unwrap() == '"') {
                            formatted =
                                String::from(&formatted[1..(len-1)]);
                        }
                    buffer.push_str(formatted.as_str());
                    curr_arg = maj_cdr(curr_arg);
                }
            } else {
                match c {
                    '{' => expect_closebracket = true,
                    '\\' => gulp_next = true,
                    '}' => {
                        return maj_err(
                            Maj::string(
                                "Unmatched closing curly brace in {}"),
                            maj_list!(fmt));
                    },
                    _ => buffer.push(c),
                }
            }
        }
        if expect_closebracket {
            return maj_err(
                Maj::string("Unmatched opening curly brace in {}"),
                maj_list!(fmt));
        }
        Maj::string(buffer.as_ref())
    } else {
        maj_err(Maj::string("{} is not a string"),
                maj_list!(fmt))
    }
}

pub fn maj_err(fmt: Gc<Maj>, rest: Gc<Maj>) -> Gc<Maj> {
    use crate::maj_dotted_list;

    if !maj_stringp(fmt.clone()).to_bool() {
        panic!("Cannot throw error: {} is not a string", fmt);
    } else if !maj_proper_list_p(rest.clone()).to_bool() {
        panic!("Cannot throw error: {} is not a proper list", rest);
    }

    maj_dotted_list!(Maj::lit(), Maj::error(), fmt, rest)
}

pub fn maj_warn(mut state: &mut MajState, fmt: Gc<Maj>,
                rest: Gc<Maj>, env: Gc<Maj>
) -> Gc<Maj> {
    let format = maj_format_prim(&state, fmt, rest);
    if maj_errorp(format.clone()).to_bool() {
        format
    } else {
        let stderr = Maj::symbol(&mut state, "*stderr*");
        let stderr = state.lookup(env, stderr);
        if maj_errorp(stderr.clone()).to_bool() {
            stderr
        } else {
            maj_write_string(
                &mut state, format, stderr.clone());
            maj_write_char(
                &mut state, Maj::character('\n'), stderr);
            Maj::nil()
        }
    }
}

#[inline]
pub fn maj_list(rest: Gc<Maj>) -> Gc<Maj> {
    rest
}

pub fn maj_append(rest: Gc<Maj>) -> Gc<Maj> {
    use crate::maj_dotted_list;
    use crate::axioms::predicates::maj_atomp;
    let car  = maj_car(rest.clone());
    let cdr  = maj_cdr(rest.clone());
    let cadr = maj_car(cdr.clone());
    let cddr = maj_cdr(cdr.clone());

    if maj_nilp(cdr.clone()).to_bool() {
        return car;
    }

    let mut itr = car.clone();
    let mut v = Vec::new();
    while !maj_nilp(itr.clone()).to_bool() {
        let cdar = maj_cdr(itr.clone());
        if maj_atomp(cdar.clone()).to_bool()
            && !maj_nilp(cdar.clone()).to_bool() {
                return maj_err(
                    Maj::string("Cannot append to dotted list"),
                    Maj::nil());
            }
        v.push(maj_car(itr.clone()));
        itr = cdar;
    }
    itr = cadr.clone();
    for obj in v.iter().rev() {
        itr = Maj::cons(obj.clone(), itr.clone());
    }
    maj_append(maj_dotted_list!(itr.clone(), cddr))
}

pub fn maj_last(x: Gc<Maj>) -> Gc<Maj> {
    if !maj_consp(x.clone()).to_bool() {
        return maj_err(
            Maj::string("{} is not a cons"),
            maj_list!(x));
    }

    let mut itr = x;
    loop {
        let cdr = maj_cdr(itr.clone());
        if !maj_consp(cdr.clone()).to_bool() {
            return itr;
        }
        itr = cdr;
    }
}

pub fn maj_reverse(x: Gc<Maj>) -> Gc<Maj> {
    use crate::axioms::predicates::maj_atomp;
    if !maj_consp(x.clone()).to_bool() {
        return maj_err(
            Maj::string("{} is not a cons"),
            maj_list!(x));
    }

    let mut itr = x.clone();
    let mut newlist = Maj::nil();
    while !maj_nilp(itr.clone()).to_bool() {
        newlist = Maj::cons(maj_car(itr.clone()), newlist);
        itr = maj_cdr(itr);
        let is_atom = maj_atomp(itr.clone()).to_bool();
        if is_atom && !maj_nilp(itr.clone()).to_bool() {
            return maj_err(
                Maj::string("Not a proper list: {}"),
                maj_list!(x));
        }
    }
    newlist
}

pub fn maj_nthcdr(n: Gc<Maj>, lst: Gc<Maj>) -> Gc<Maj> {
    use crate::axioms::predicates::maj_integerp;
    if maj_nilp(lst.clone()).to_bool() {
        return Maj::nil();
    }

    if !maj_integerp(n.clone()).to_bool() {
        return maj_err(Maj::string("{} is not an integer"),
                       maj_list!(n))
    }

    let mut num = n.to_integer().unwrap();
    if num < 0 {
        return maj_err(Maj::string("{} is not a valid index"),
                       maj_list!(n));
    }

    let mut iter = lst.clone();
    loop {
        if !maj_consp(iter.clone()).to_bool() &&
            !maj_nilp(iter.clone()).to_bool()
        {
            return maj_err(Maj::string("{} is not a list"),
                           maj_list!(iter));
        }
        if num <= 0 { break; }
        num -= 1;
        iter = maj_cdr(iter);
    }
    iter
}

pub fn maj_nth(n: Gc<Maj>, lst: Gc<Maj>) -> Gc<Maj> {
    let cons = maj_nthcdr(n, lst);
    if maj_errorp(cons.clone()).to_bool() {
        cons
    } else {
        maj_car(cons)
    }
}

pub fn maj_macroexpand_1(mut state: &mut MajState,
                         expr: Gc<Maj>,
                         env: Gc<Maj>) -> (Gc<Maj>, bool) {
    use crate::evaluator::application::expand_macro;

    if !maj_consp(expr.clone()).to_bool() {
        return (expr, true);
    }

    let mac = maj_car(expr.clone());
    let args = maj_cdr(expr.clone());

    let mac = state.lookup(env.clone(), mac);
    expand_macro(&mut state, mac, args, env)
}

#[inline]
pub fn maj_not(x: Gc<Maj>) -> Gc<Maj> {
    maj_nilp(x)
}

#[inline]
pub fn maj_gensym(mut state: &mut MajState) -> Gc<Maj> {
    Maj::gensym(&mut state)
}

pub fn maj_number_coerce(
    mut state: &mut MajState,
    subtype: Gc<Maj>,
    number: Gc<Maj>
) -> Gc<Maj> {
    use crate::printing::maj_format;
    use crate::axioms::predicates::maj_symbolp;

    if !maj_numberp(number.clone()).to_bool() {
        return maj_err(Maj::string("{} is not a number"),
                       maj_list!(number));
    }

    if !maj_symbolp(subtype.clone()).to_bool() {
        return maj_err(Maj::string("{} is not a symbol"),
                       maj_list!(subtype));
    }


    let number_type = maj_type(&mut state, number.clone());
    let number_type = number_type.to_raw_sym().unwrap();
    let number_type = FromPrimitive::from_u64(number_type)
        .unwrap();

    let coerce_type = subtype.to_raw_sym().unwrap();
    let coerce_type = FromPrimitive::from_u64(coerce_type);

    match number_type {
        MajRawSym::Integer => match coerce_type {
            Some(MajRawSym::Integer) => number,
            Some(MajRawSym::Float) => {
                Maj::float(number.to_forced_float().unwrap())
            },
            Some(MajRawSym::Fraction) => {
                Maj::fraction(number.to_integer().unwrap(), 1)
            },
            Some(MajRawSym::Complex) => {
                Maj::complex(number.clone(), Maj::float(0.0))
            },
            _ => maj_err(
                Maj::string("{} is not a number subtype"),
                maj_list!(subtype)),
        },
        MajRawSym::Float => match coerce_type {
            Some(MajRawSym::Integer) => {
                Maj::integer(number
                             .to_float()
                             .unwrap()
                             .trunc()
                             as i64)
            },
            Some(MajRawSym::Float) => number,
            Some(MajRawSym::Fraction) => {
                let mut buffer = maj_format(&state, number);
                let dot_index = buffer.find('.').unwrap();
                let denom_pow = (buffer.len() - dot_index - 1) as u32;
                let denom = 10_i64.pow(denom_pow);
                buffer.remove(dot_index);
                let frac =
                    Maj::fraction(buffer.parse().unwrap(), denom);
                simplify_frac(frac).unwrap()
            },
            Some(MajRawSym::Complex) => {
                Maj::complex(number.clone(), Maj::float(0.0))
            },
            _ => maj_err(
                Maj::string("{} is not a number subtype"),
                maj_list!(subtype)),
        },
        MajRawSym::Fraction => match coerce_type {
            Some(MajRawSym::Integer) => {
                Maj::integer(number.to_forced_float()
                             .unwrap()
                             .trunc()
                             as i64)
            },
            Some(MajRawSym::Float) => {
                Maj::float(number.to_forced_float()
                           .unwrap())
            },
            Some(MajRawSym::Fraction) => number,
            Some(MajRawSym::Complex) => {
                Maj::complex(number.clone(), Maj::float(0.0))
            },
            _ => maj_err(
                Maj::string("{} is not a number subtype"),
                maj_list!(subtype)),
        },
        MajRawSym::Complex => match coerce_type {
            Some(MajRawSym::Integer) |
            Some(MajRawSym::Float)   |
            Some(MajRawSym::Fraction) => {
                let realpart = maj_real_part(number);
                maj_number_coerce(&mut state, subtype, realpart)
            },
            Some(MajRawSym::Complex) => number,
            _ => maj_err(
                Maj::string("{} is not a number subtype"),
                maj_list!(subtype)),
        },
        _ => panic!("Symbol is not a number subtype"),
    }
}

pub fn maj_real_part(x: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajNumber;
    match &*x.clone() {
        Maj::Number(num) => {
            match &num.clone() {
                MajNumber::Complex {
                    real,
                    imag: _
                } => {
                    let real = &*real.clone();
                    Gc::new(Maj::Number(real.clone()))
                },
                _ => maj_err(Maj::string(
                    "{} is not a complex number"),
                    maj_list!(x)),
            }
        },
        _ => maj_err(Maj::string("{} is not a number"),
                     maj_list!(x)),
    }
}

pub fn maj_imag_part(x: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajNumber;
    match &*x.clone() {
        Maj::Number(num) => {
            match &num.clone() {
                MajNumber::Complex {
                    real: _,
                    imag
                } => {
                    let imag = &*imag.clone();
                    Gc::new(Maj::Number(imag.clone()))
                },
                _ => maj_err(Maj::string(
                    "{} is not a complex number"),
                    maj_list!(x)),
            }
        },
        _ => maj_err(Maj::string("{} is not a number"),
                     maj_list!(x)),
    }
}

pub fn maj_numer(x: Gc<Maj>) -> Gc<Maj> {
    if !maj_numberp(x.clone()).to_bool() {
        maj_err(Maj::string("{} is not a number"),
                maj_list!(x))
    } else {
        match x.clone().to_fraction() {
            Some((numer, _)) => Maj::integer(numer),
            None => maj_err(Maj::string("{} is not a fraction"),
                            maj_list!(x)),
        }
    }
}

pub fn maj_denom(x: Gc<Maj>) -> Gc<Maj> {
    if !maj_numberp(x.clone()).to_bool() {
        maj_err(Maj::string("{} is not a number"),
                maj_list!(x))
    } else {
        match x.clone().to_fraction() {
            Some((_, denom)) => Maj::integer(denom),
            None => maj_err(Maj::string("{} is not a fraction"),
                            maj_list!(x)),
        }
    }
}

pub fn maj_richest_number_type(
    mut state: &mut MajState,
    x: Gc<Maj>,
    y: Gc<Maj>
) -> Gc<Maj> {
    if !maj_numberp(x.clone()).to_bool() {
        return maj_err(Maj::string("{} is not a number"),
                       maj_list!(x));
    }
    if !maj_numberp(y.clone()).to_bool() {
        return maj_err(Maj::string("{} is not a number"),
                       maj_list!(y));
    }

    let x_type = maj_type(&mut state, x.clone());
    let x_type_sym = x_type.to_raw_sym().unwrap();
    let x_type_sym = FromPrimitive::from_u64(x_type_sym).unwrap();

    let y_type = maj_type(&mut state, y.clone());
    let y_type_sym = y_type.to_raw_sym().unwrap();
    let y_type_sym = FromPrimitive::from_u64(y_type_sym).unwrap();

    match x_type_sym {
        MajRawSym::Integer => {
            match y_type_sym {
                MajRawSym::Integer  |
                MajRawSym::Float    |
                MajRawSym::Fraction |
                MajRawSym::Complex  => y_type,
                _ => unimplemented!(
                    "Usage of unknown number subtype"),
            }
        },
        MajRawSym::Float => {
            match y_type_sym {
                MajRawSym::Integer  => x_type,
                MajRawSym::Float    |
                MajRawSym::Fraction |
                MajRawSym::Complex  => y_type,
                _ => unimplemented!(
                    "Usage of unknown number subtype"),
            }
        },
        MajRawSym::Fraction => {
            match y_type_sym {
                MajRawSym::Integer |
                MajRawSym::Float   |
                MajRawSym::Fraction => x_type,
                MajRawSym::Complex  => y_type,
                _ => unimplemented!(
                    "Usage of unknown number subtype"),
            }
        },
        MajRawSym::Complex => {
            match y_type_sym {
                MajRawSym::Integer  |
                MajRawSym::Float    |
                MajRawSym::Fraction |
                MajRawSym::Complex  => x_type,
                _ => unimplemented!(
                    "Usage of unknown number subtype"),
            }
        },
        _ => unimplemented!("Usage of unknown number subtype"),
    }
}

pub fn maj_rich_number_coerce(
    mut state: &mut MajState,
    x: Gc<Maj>,
    y: Gc<Maj>
) -> Gc<Maj> {
    let best_type = maj_richest_number_type(&mut state,
                                            x.clone(),
                                            y.clone());
    if maj_errorp(best_type.clone()).to_bool() {
        return best_type;
    }
    maj_list!(maj_number_coerce(&mut state,
                                best_type.clone(), x),
              maj_number_coerce(&mut state, best_type, y))
}

#[inline]
pub fn maj_iota(n: Gc<Maj>) -> Gc<Maj> {
    use crate::axioms::predicates::maj_integerp;
    let common_err = maj_err(
        Maj::string("iota expects a positive integer number"),
        Maj::nil());
    if !maj_integerp(n.clone()).to_bool() {
        common_err
    } else {
        let mut num = n.to_integer().unwrap();
        if num < 0 {
            common_err
        } else {
            let mut list = Maj::nil();
            while num > 0 {
                list = Maj::cons(Maj::integer(num - 1), list);
                num -= 1;
            }
            list
        }
    }
}

use crate::axioms::predicates::maj_zerop;

fn maj_arithm_coercion_helper(
    mut state: &mut MajState,
    x: Gc<Maj>,
    y: Gc<Maj>
) -> Result<(Gc<Maj>, Gc<Maj>, MajRawSym), Gc<Maj>> {
    if !maj_numberp(x.clone()).to_bool() {
        return Err(maj_err(
            Maj::string("{} is not a number"),
            maj_list!(x)));
    }
    if !maj_numberp(y.clone()).to_bool() {
        return Err(maj_err(
            Maj::string("{} is not a number"),
            maj_list!(y)));
    }
    // Perform rich coercion
    let coerced = maj_rich_number_coerce(&mut state, x, y);
    if maj_errorp(coerced.clone()).to_bool() {
        return Err(coerced);
    }
    let x = maj_car(coerced.clone());
    let y = maj_car(maj_cdr(coerced));
    let ntype = maj_type(&mut state, x.clone());

    Ok((x, y,
        FromPrimitive::from_u64(ntype.to_raw_sym()
                                .unwrap())
        .unwrap()))
}

#[derive(Debug)]
enum MajArithmFnType {
    Equals,
    Greater,
    Lesser,
    Plus,
    Minus,
    Multiply,
    Divide,
}

fn maj_arithm_dispatch(
    mut state: &mut MajState,
    env: Gc<Maj>,
    fntype: MajArithmFnType,
    x: Gc<Maj>,
    y: Gc<Maj>,
    rest: Gc<Maj>,
    accum: bool
) -> Gc<Maj> {
    let mut origy = y.clone();
    match maj_arithm_coercion_helper(&mut state, x, y.clone()) {
        Err(e) => e,
        Ok((x, y, ntype)) => {
            let result =
                match fntype {
                    MajArithmFnType::Equals => {
                        maj_arithm_internal_eq(
                            &mut state, env.clone(),
                            x, y, ntype)
                    },
                    MajArithmFnType::Greater => {
                        maj_arithm_internal_gl(
                            x, y, ntype, true)
                    },
                    MajArithmFnType::Lesser => {
                        maj_arithm_internal_gl(
                            x, y, ntype, false)
                    },
                    MajArithmFnType::Plus => {
                        maj_arithm_internal_sum(
                            &mut state, env.clone(),
                            x, y, ntype)
                    },
                    MajArithmFnType::Minus => {
                        maj_arithm_internal_subtract(
                            &mut state, env.clone(),
                            x, y, ntype)
                    },
                    MajArithmFnType::Multiply => {
                        maj_arithm_internal_multiply(
                            &mut state, env.clone(),
                            x, y, ntype)
                    },
                    MajArithmFnType::Divide => {
                        maj_arithm_internal_divide(
                            &mut state, env.clone(),
                            x, y, ntype)
                    },
                };

            let result = match result {
                Ok(b) => b,
                Err(e) => return e,
            };

            if accum {
                origy = result.clone();
            }

            if !maj_nilp(rest.clone()).to_bool() {
                maj_arithm_dispatch(
                    &mut state, env,
                    fntype,
                    origy,
                    maj_car(rest.clone()),
                    maj_cdr(rest),
                    accum)
            } else {
                result
            }
        }
    }
}

fn maj_arithm_internal_eq(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>,
    y: Gc<Maj>,
    ntype: MajRawSym
) -> Result<Gc<Maj>, Gc<Maj>> {
    match ntype {
        MajRawSym::Integer => {
            let x = x.to_integer().unwrap();
            let y = y.to_integer().unwrap();
            Ok(if x == y { Maj::t() } else { Maj::nil() })
        },
        MajRawSym::Fraction => {
            let (n1, d1) = x.to_fraction().unwrap();
            let (n2, d2) = y.to_fraction().unwrap();
            Ok(if (n1 == n2) && (d1 == d2) {
                Maj::t()
            } else {
                Maj::nil()
            })
        },
        MajRawSym::Complex => {
            // Comparação recursiva.
            // Ah... eu posso realmente comparar esses
            // números dessa forma?
            let r1 = maj_real_part(x.clone());
            let i1 = maj_imag_part(x);
            let r2 = maj_real_part(y.clone());
            let i2 = maj_imag_part(y);
            let res =
                maj_arithm_eq(&mut state, env.clone(),
                              r1, r2,
                              Maj::nil()).to_bool() &&
                maj_arithm_eq(&mut state, env.clone(),
                              i1, i2,
                              Maj::nil()).to_bool();
            Ok(if res { Maj::t() } else { Maj::nil() })
        },
        MajRawSym::Float => {
            let fres =
                maj_arithm_floateq(&mut state, env.clone(),
                                   x, y);
            if maj_errorp(fres.clone()).to_bool() {
                return Err(fres);
            }
            Ok(fres)
        },
        t => panic!("{:?} is not a number subtype", t),
    }
}

fn maj_arithm_internal_gl(
    x: Gc<Maj>,
    y: Gc<Maj>,
    ntype: MajRawSym,
    is_greater: bool
) -> Result<Gc<Maj>, Gc<Maj>> {
    match ntype {
        MajRawSym::Integer => {
            let x = x.to_integer().unwrap();
            let y = y.to_integer().unwrap();
            let res = if is_greater {
                x > y
            } else {
                x < y
            };
            Ok(if res { Maj::t() } else { Maj::nil() })
        },
        MajRawSym::Float => {
            let x = x.to_float().unwrap();
            let y = y.to_float().unwrap();
            let res = if is_greater {
                x > y
            } else {
                x < y
            };
            Ok(if res { Maj::t() } else { Maj::nil() })
        },
        MajRawSym::Fraction => {
            // Use common denominator test
            let (n1, d1) = x.to_fraction().unwrap();
            let (n2, d2) = y.to_fraction().unwrap();
            let res = if is_greater {
                (n1 * d2) > (n2 * d1)
            } else {
                (n1 * d2) < (n2 * d1)
            };
            Ok(if res { Maj::t() } else { Maj::nil() })
        },
        MajRawSym::Complex =>
            Err(maj_err(Maj::string(
                "The set of complex numbers can't be an ordered field"),
                        Maj::nil())),
        t => panic!("{:?} is not a number subtype", t),
    }
}

pub fn maj_arithm_eq(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>,
    y: Gc<Maj>,
    rest: Gc<Maj>
) -> Gc<Maj> {
    maj_arithm_dispatch(&mut state, env,
                        MajArithmFnType::Equals,
                        x, y, rest, false)
}

pub fn maj_arithm_floateq(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>,
    y: Gc<Maj>
) -> Gc<Maj> {
    use crate::axioms::predicates::maj_floatp;
    use core::f64;
    use float_cmp::approx_eq;
    if !maj_floatp(x.clone()).to_bool() {
        return maj_err(Maj::string("{} is not a float number"),
                       maj_list!(x));
    } else if !maj_floatp(y.clone()).to_bool() {
        return maj_err(Maj::string("{} is not a float number"),
                       maj_list!(y));
    }

    let ulps = Maj::symbol(&mut state, "*ulps*");
    let ulps = state.lookup(env, ulps);
    if maj_errorp(ulps.clone()).to_bool() {
        return ulps;
    }
    if let Some(u) = ulps.to_integer() {
        if u < 0 {
            maj_err(
                Maj::string("*ulps* cannot be smaller than 0"),
                Maj::nil())
        } else {
            let x = x.to_float().unwrap();
            let y = y.to_float().unwrap();
            if approx_eq!(f64, x, y, ulps = u) {
                Maj::t()
            } else {
                Maj::nil()
            }
        }
    } else {
        maj_err(Maj::string("*ulps* is not an integer"),
                Maj::nil())
    }
}

pub fn maj_arithm_greater(
    mut state: &mut MajState,
    x: Gc<Maj>,
    y: Gc<Maj>,
    rest: Gc<Maj>
) -> Gc<Maj> {
    maj_arithm_dispatch(&mut state, Maj::nil(),
                        MajArithmFnType::Greater,
                        x, y, rest, false)
}

pub fn maj_arithm_lesser(
    mut state: &mut MajState,
    x: Gc<Maj>,
    y: Gc<Maj>,
    rest: Gc<Maj>
) -> Gc<Maj> {
    maj_arithm_dispatch(&mut state, Maj::nil(),
                        MajArithmFnType::Lesser,
                        x, y, rest, false)
}

pub fn maj_arithm_geq(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>,
    y: Gc<Maj>,
    rest: Gc<Maj>
) -> Gc<Maj> {
    let result = maj_arithm_greater(&mut state,
                                    x.clone(), y.clone(),
                                    rest.clone());
    if maj_errorp(result.clone()).to_bool() {
        result
    } else if result.to_bool() {
        Maj::t()
    } else {
        maj_arithm_eq(&mut state, env, x, y, rest)
    }
}

pub fn maj_arithm_leq(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>,
    y: Gc<Maj>,
    rest: Gc<Maj>
) -> Gc<Maj> {
    let result = maj_arithm_lesser(&mut state,
                                   x.clone(), y.clone(),
                                   rest.clone());
    if maj_errorp(result.clone()).to_bool() {
        result
    } else if result.to_bool() {
        Maj::t()
    } else {
        maj_arithm_eq(&mut state, env, x, y, rest)
    }
}

fn maj_arithm_internal_sum(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>,
    y: Gc<Maj>,
    ntype: MajRawSym
) -> Result<Gc<Maj>, Gc<Maj>> {
    match ntype {
        MajRawSym::Integer => {
            let x = x.to_integer().unwrap();
            let y = y.to_integer().unwrap();
            Ok(Maj::integer(x + y))
        },
        MajRawSym::Float => {
            let x = x.to_float().unwrap();
            let y = y.to_float().unwrap();
            Ok(Maj::float(x + y))
        },
        MajRawSym::Fraction => {
            let (nx, dx) = x.to_fraction().unwrap();
            let (ny, dy) = y.to_fraction().unwrap();
            let dr = dx * dy;
            let nr = (nx * dy) + (ny * dx);
            simplify_frac_coerce(Maj::fraction(nr, dr))
        },
        MajRawSym::Complex => {
            let xr = maj_real_part(x.clone());
            let xi = maj_imag_part(x);
            let yr = maj_real_part(y.clone());
            let yi = maj_imag_part(y);
            let real = maj_arithm_plus(
                &mut state, env.clone(),
                maj_list!(xr, yr));
            let imag = maj_arithm_plus(
                &mut state, env, maj_list!(xi, yi));
            Ok(Maj::complex(real, imag))
        },
        t => panic!("{:?} is not a number subtype", t),
    }
}

fn maj_conjugate(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>
) -> Gc<Maj> {
    use crate::axioms::predicates::maj_complexp;
    if !maj_complexp(x.clone()).to_bool() {
        x
    } else {
        let realpart = maj_real_part(x.clone());
        let imagpart = maj_imag_part(x);
        let imagpart = maj_arithm_times(
            &mut state, env,
            maj_list!(imagpart, Maj::integer(-1)));
        if maj_errorp(imagpart.clone()).to_bool() {
            imagpart
        } else {
            Maj::complex(realpart, imagpart)
        }
    }
}

fn maj_arithm_internal_subtract(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>,
    y: Gc<Maj>,
    ntype: MajRawSym
) -> Result<Gc<Maj>, Gc<Maj>> {
    match ntype {
        MajRawSym::Integer => {
            let x = x.to_integer().unwrap();
            let y = y.to_integer().unwrap();
            Ok(Maj::integer(x - y))
        },
        MajRawSym::Float => {
            let x = x.to_float().unwrap();
            let y = y.to_float().unwrap();
            Ok(Maj::float(x - y))
        },
        MajRawSym::Fraction => {
            let (nx, dx) = x.to_fraction().unwrap();
            let (ny, dy) = y.to_fraction().unwrap();
            let dr = dx * dy;
            let nr = (nx * dy) - (ny * dx);
            simplify_frac_coerce(Maj::fraction(nr, dr))
        },
        MajRawSym::Complex => {
            let xr = maj_real_part(x.clone());
            let xi = maj_imag_part(x);
            let yr = maj_real_part(y.clone());
            let yi = maj_imag_part(y);
            let real = maj_arithm_minus(
                &mut state, env.clone(),
                maj_list!(xr, yr));
            let imag = maj_arithm_minus(
                &mut state, env, maj_list!(xi, yi));
            Ok(Maj::complex(real, imag))
        },
        t => panic!("{:?} is not a number subtype", t),
    }
}

fn maj_negate(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>
) -> Gc<Maj> {
    maj_arithm_times(
        &mut state, env,
        maj_list!(x, Maj::integer(-1)))
}

fn maj_arithm_internal_multiply(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>,
    y: Gc<Maj>,
    ntype: MajRawSym
) -> Result<Gc<Maj>, Gc<Maj>> {
    match ntype {
        MajRawSym::Integer => {
            let x = x.to_integer().unwrap();
            let y = y.to_integer().unwrap();
            Ok(Maj::integer(x * y))
        },
        MajRawSym::Float => {
            let x = x.to_float().unwrap();
            let y = y.to_float().unwrap();
            Ok(Maj::float(x * y))
        },
        MajRawSym::Fraction => {
            let (nx, dx) = x.to_fraction().unwrap();
            let (ny, dy) = y.to_fraction().unwrap();
            let nr = nx * ny;
            let dr = dx * dy;
            simplify_frac_coerce(Maj::fraction(nr, dr))
        },
        MajRawSym::Complex => {
            Ok(maj_arithm_internal_complex_mult(
                &mut state, env, x, y))
        },
        t => panic!("{:?} is not a number subtype", t),
    }
}

fn maj_arithm_internal_complex_mult(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>,
    y: Gc<Maj>
) -> Gc<Maj> {
    let xr = maj_real_part(x.clone());
    let xi = maj_imag_part(x);
    let yr = maj_real_part(y.clone());
    let yi = maj_imag_part(y);

    let firsts = maj_arithm_times(
        &mut state, env.clone(),
        maj_list!(xr.clone(), yr.clone()));
    let outers = Maj::complex(
        Maj::integer(0),
        maj_arithm_times(
            &mut state, env.clone(),
            maj_list!(xr.clone(), yi.clone())));
    let inners = Maj::complex(
        Maj::integer(0),
        maj_arithm_times(
            &mut state, env.clone(),
            maj_list!(xi.clone(), yr.clone())));
    let lasts = maj_arithm_times(
        &mut state, env.clone(),
        maj_list!(xi, yi));
    let lasts = maj_negate(&mut state, env.clone(), lasts);

    maj_arithm_plus(
        &mut state, env,
        maj_list!(firsts, outers, inners, lasts))
}

fn maj_signum(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>
) -> Gc<Maj> {
    if maj_zerop(&mut state, env, x.clone()).to_bool() {
        x
    } else if maj_arithm_lesser(
        &mut state,
        x.clone(),
        Maj::integer(0),
        Maj::nil()).to_bool() {
        Maj::integer(-1)
    } else {
        Maj::integer(1)
    }
}

fn maj_arithm_internal_divide(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>,
    y: Gc<Maj>,
    ntype: MajRawSym
) -> Result<Gc<Maj>, Gc<Maj>> {
    match ntype {
        MajRawSym::Integer => {
            let x = x.to_integer().unwrap();
            let y = y.to_integer().unwrap();
            if y == 0 {
                Err(maj_err(Maj::string("Division by zero"),
                            Maj::nil()))
            } else {
                Ok(if x % y == 0 {
                    Maj::integer(x / y)
                } else {
                    simplify_frac_coerce(Maj::fraction(x, y)).unwrap()
                })
            }
        },
        MajRawSym::Float => {
            if maj_zerop(&mut state, env, y.clone()).to_bool() {
                Err(maj_err(Maj::string("Division by zero"),
                            Maj::nil()))
            } else {
                let x = x.to_float().unwrap();
                let y = y.to_float().unwrap();
                Ok(Maj::float(x / y))
            }
        },
        MajRawSym::Fraction => {
            let (nx, dx) = x.to_fraction().unwrap();
            let (ny, dy) = y.to_fraction().unwrap();
            let nr = nx * dy;
            let dr = dx * ny;
            // Division by zero verified by this function
            simplify_frac_coerce(Maj::fraction(nr, dr))
        },
        MajRawSym::Complex => {
            return maj_arithm_internal_complex_div(
                &mut state, env, x, y);
        },
        t => panic!("{:?} is not a number subtype", t),
    }
}

fn maj_arithm_internal_complex_div(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>,
    y: Gc<Maj>
) -> Result<Gc<Maj>, Gc<Maj>> {
    let yconj = maj_arithm_plus(
        &mut state, env.clone(),
        maj_list!(y.clone()));

    let dividend = maj_arithm_times(
        &mut state, env.clone(),
        maj_list!(x, yconj.clone()));
    let divisor = maj_arithm_times(
        &mut state, env.clone(),
        maj_list!(y, yconj.clone()));

    let divisor = maj_real_part(divisor);
    if maj_zerop(&mut state,
                 env.clone(),
                 divisor.clone()
    ).to_bool() {
        return Err(maj_err(Maj::string("Division by zero"),
                           Maj::nil()));
    }

    let dividend_r = maj_real_part(dividend.clone());
    let dividend_i = maj_imag_part(dividend);

    let realpart = maj_arithm_divide(
        &mut state, env.clone(),
        maj_list!(dividend_r, divisor.clone()));
    let imagpart = maj_arithm_divide(
        &mut state, env.clone(),
        maj_list!(dividend_i, divisor.clone()));
    Ok(Maj::complex(realpart, imagpart))
}

fn maj_reciprocal(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>
) -> Gc<Maj> {
    if maj_zerop(&mut state, env.clone(), x.clone()).to_bool() {
        maj_err(Maj::string("Division by zero"),
                Maj::nil())
    } else {
        maj_arithm_divide(
            &mut state, env,
            maj_list!(Maj::fraction(1, 1), x.clone()))
    }
}

pub fn maj_arithm_plus(
    mut state: &mut MajState,
    env: Gc<Maj>,
    rest: Gc<Maj>
) -> Gc<Maj> {
    let len = maj_length(rest.clone());
    if maj_errorp(len.clone()).to_bool() {
        return len;
    }
    match len.to_integer().unwrap() {
        0 => Maj::integer(1),
        1 => {
            maj_destructure_args!(rest, x);
            if !maj_numberp(x.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(x))
            } else {
                maj_conjugate(&mut state, env, x)
            }
        },
        _ => {
            maj_destructure_args!(rest, x, r1, y, rest);
            if !maj_numberp(x.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(x))
            } else if !maj_numberp(y.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(y))
            } else {
                maj_arithm_dispatch(
                    &mut state, Maj::nil(),
                    MajArithmFnType::Plus,
                    x, y, rest, true)
            }
        },
    }
}

pub fn maj_arithm_minus(
    mut state: &mut MajState,
    env: Gc<Maj>,
    rest: Gc<Maj>
) -> Gc<Maj> {
    let len = maj_length(rest.clone());
    if maj_errorp(len.clone()).to_bool() {
        return len;
    }
    match len.to_integer().unwrap() {
        0 => Maj::integer(0),
        1 => {
            maj_destructure_args!(rest, x);
            if !maj_numberp(x.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(x))
            } else {
                maj_negate(&mut state, env, x)
            }
        },
        _ => {
            maj_destructure_args!(rest, x, r1, y, rest);
            if !maj_numberp(x.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(x))
            } else if !maj_numberp(y.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(y))
            } else {
                maj_arithm_dispatch(
                    &mut state, Maj::nil(),
                    MajArithmFnType::Minus,
                    x, y, rest, true)
            }
        },
    }
}

pub fn maj_arithm_times(
    mut state: &mut MajState,
    env: Gc<Maj>,
    rest: Gc<Maj>
) -> Gc<Maj> {
    let len = maj_length(rest.clone());
    if maj_errorp(len.clone()).to_bool() {
        return len;
    }
    match len.to_integer().unwrap() {
        0 => Maj::integer(1),
        1 => {
            maj_destructure_args!(rest, x);
            if !maj_numberp(x.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(x))
            } else {
                maj_signum(&mut state, env, x)
            }
        },
        _ => {
            maj_destructure_args!(rest, x, r1, y, rest);
            if !maj_numberp(x.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(x))
            } else if !maj_numberp(y.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(y))
            } else {
                maj_arithm_dispatch(
                    &mut state, Maj::nil(),
                    MajArithmFnType::Multiply,
                    x, y, rest, true)
            }
        },
    }
}

pub fn maj_arithm_divide(
    mut state: &mut MajState,
    env: Gc<Maj>,
    rest: Gc<Maj>
) -> Gc<Maj> {
    let len = maj_length(rest.clone());
    if maj_errorp(len.clone()).to_bool() {
        return len;
    }
    match len.to_integer().unwrap() {
        0 => Maj::integer(1),
        1 => {
            maj_destructure_args!(rest, x);
            if !maj_numberp(x.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(x))
            } else {
                maj_reciprocal(&mut state, env, x)
            }
        },
        _ => {
            maj_destructure_args!(rest, x, r1, y, rest);
            if !maj_numberp(x.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(x))
            } else if !maj_numberp(y.clone()).to_bool() {
                maj_err(Maj::string("{} is not a number"),
                        maj_list!(y))
            } else {
                maj_arithm_dispatch(
                    &mut state, Maj::nil(),
                    MajArithmFnType::Divide,
                    x, y, rest, true)
            }
        },
    }
}

use crate::core::types::{
    MajStreamType,
    MajStreamDirection
};

fn get_raw_stream(
    mut state: &mut MajState,
    stream: Gc<Maj>,
    expected_dir: MajStreamDirection
) -> Result<&std::fs::File, Gc<Maj>> {
    use std::io::Seek;
    if let Maj::Stream(mstream) = &*stream.clone() {
        if mstream.is_internal() {
            panic!("Never try to use *stdin* or *stdout* as ordinary streams!");
        }

        if maj_nilp(state.stat_stream(mstream.handle)).to_bool() {
            Err(maj_err(
                Maj::string("The stream {} is closed"),
                maj_list!(stream)))
        }
        else if mstream.direction == expected_dir {
            let eof_sym = Maj::symbol(&mut state, "eof");
            let file = state.borrow_stream(mstream.handle);
            let mut file = file.unwrap();
            if expected_dir == MajStreamDirection::In {
                // Check if eof.
                // If fails, position is unspecified and should
                // error out anyway. So we use unwrap.
                let length = file.stream_len().unwrap();
                let pos    = file.stream_position().unwrap();
                if pos >= length {
                    return Err(eof_sym)
                }
            }
            Ok(file)
        }
        else {
            Err(maj_err(
                Maj::string("{} is not an {} stream"),
                maj_list!(
                    stream,
                Maj::string(
                    if expected_dir ==
                        MajStreamDirection::In {
                            "input"
                        } else {
                            "output"
                        }))))
        }
    } else {
        Err(maj_err(
            Maj::string("{} is not a stream"),
            maj_list!(stream)))
    }
}

fn stdstreamp(x: Gc<Maj>) -> bool {
    if let Maj::Stream(mstream) = &*x.clone() {
        mstream.is_internal()
    } else {
        false
    }
}

fn stdstreamdirp(x: Gc<Maj>, dir: MajStreamDirection) -> bool {
    if let Maj::Stream(mstream) = &*x.clone() {
        if !mstream.is_internal() {
            panic!("Not a standard stream being tested");
        }
        dir == mstream.direction
    } else {
        false
    }    
}

fn stdstreamtype(x: Gc<Maj>) -> MajStreamType {
    if let Maj::Stream(mstream) = &*x.clone() {
        if !mstream.is_internal() {
            panic!("Not a standard stream being tested");
        }
        mstream.stype.clone()
    } else {
        panic!("Not a standard stream being tested");
    }
}

pub fn maj_open_stream(
    mut state: &mut MajState,
    dir: Gc<Maj>,
    path: Gc<Maj>
) -> Gc<Maj> {
    let path_opt = path.stringify();
    if path_opt.is_none() {
        return maj_err(
            Maj::string("{} is not a string"),
            maj_list!(path));
    }

    let direction =
        if maj_eq(Maj::symbol(&mut state, "in"),
                  dir.clone()).to_bool() {
            MajStreamDirection::In
        } else if maj_eq(Maj::symbol(&mut state, "out"),
                         dir.clone()).to_bool() {
            MajStreamDirection::Out
        } else {
            return maj_err(
                Maj::string("{} should be one of symbols 'in or 'out"),
                maj_list!(dir));
        };

    let stream =
        state.make_stream(&path_opt.unwrap(), direction);

    match stream {
        Some(obj) => obj,
        None => maj_err(
            Maj::string("Cannot open stream to path {}"),
            maj_list!(path)),
    }
}

pub fn maj_close_stream(
    state: &mut MajState,
    x: Gc<Maj>
) -> Gc<Maj> {
    state.close_stream(x)
}

pub fn maj_stat(mut state: &mut MajState, x: Gc<Maj>) -> Gc<Maj> {
    if let Maj::Stream(mstream) = &*x.clone() {
        let index  = mstream.handle;
        let result = state.stat_stream(index);

        if maj_errorp(result.clone()).to_bool() {
            result
        } else if maj_nilp(result.clone()).to_bool() {
            Maj::symbol(&mut state, "closed")
        } else {
            Maj::symbol(&mut state, "open")
        }
    } else {
        maj_err(
            Maj::string("Not a stream: {}"),
            maj_list!(x))
    }
}

pub fn maj_read_char(mut state: &mut MajState,
                     stream: Gc<Maj>
) -> Gc<Maj> {
    use std::io::Read;
    let mut buffer = [0; 1];
    if stdstreamp(stream.clone()) {
        if stdstreamdirp(stream.clone(), MajStreamDirection::In) {
            let peekopt = state.pop_stdin_peeked();
            match peekopt {
                Some(c) => {
                    return Maj::character(c);
                },
                None => {
                    use std::io;
                    match io::stdin().read(&mut buffer) {
                        Ok(_) => {
                            return Maj::character(buffer[0] as char);
                        },
                        Err(_) => {
                            return maj_err(
                                Maj::string(
                                    "Could not read from stream *stdin*"),
                                Maj::nil());
                        }
                    }
                },
            }
        } else {
            return maj_err(
                Maj::string("{} is not an input stream"),
                maj_list!(stream));
        }
    }

    match get_raw_stream(&mut state,
                         stream.clone(),
                         MajStreamDirection::In) {
        Ok(mut file) => {
            match file.read(&mut buffer) {
                Ok(_) => {
                    Maj::character(buffer[0] as char)
                },
                Err(_) => {
                    maj_err(
                        Maj::string(
                            "Could not read from stream {}"),
                        maj_list!(stream))
                }
            }
        },
        Err(expr) => expr,
    }
}

pub fn maj_peek_char(mut state: &mut MajState,
                     stream: Gc<Maj>
) -> Gc<Maj> {
    use std::io::{ Read, Seek, SeekFrom };
    let mut buffer = [0; 1];
    if stdstreamp(stream.clone()) {
        if stdstreamdirp(stream.clone(), MajStreamDirection::In) {
            let peekopt = state.pop_stdin_peeked();
            match peekopt {
                Some(c) => {
                    state.push_stdin_peeked(c);
                    return Maj::character(c);
                },
                None => {
                    use std::io;
                    match io::stdin().read(&mut buffer) {
                        Ok(_) => {
                            state.push_stdin_peeked(buffer[0] as char);
                            return Maj::character(buffer[0] as char);
                        },
                        Err(_) => {
                            return maj_err(
                                Maj::string(
                                    "Could not read from stream *stdin*"),
                                Maj::nil());
                        }
                    }
                },
            }
        } else {
            return maj_err(
                Maj::string("{} is not an input stream"),
                maj_list!(stream));
        }
    }

    match get_raw_stream(&mut state,
                         stream.clone(),
                         MajStreamDirection::In) {
        Ok(mut file) => {
            match file.read(&mut buffer) {
                Ok(_) => {
                    // Since a character was read,
                    // seek back to a point before it.
                    // If unable, this should fail anyway
                    file.seek(SeekFrom::Current(-1))
                        .unwrap();
                    Maj::character(buffer[0] as char)
                },
                Err(_) => {
                    maj_err(
                        Maj::string(
                            "Could not read from stream {}"),
                        maj_list!(stream))
                }
            }
        },
        Err(expr) => expr,
    }
}

pub fn maj_write_char(mut state: &mut MajState,
                      c: Gc<Maj>,
                      stream: Gc<Maj>
) -> Gc<Maj> {
    use std::io::Write;
    use crate::axioms::predicates::maj_charp;
    if !maj_charp(c.clone()).to_bool() {
        return maj_err(
            Maj::string("{} is not a character"),
            maj_list!(c));
    }
    let c = c.to_char().unwrap();

    if stdstreamp(stream.clone()) {
        if stdstreamdirp(stream.clone(), MajStreamDirection::Out) {
            match stdstreamtype(stream) {
                MajStreamType::Stdout =>
                    print!("{}", c),
                MajStreamType::Stderr =>
                    eprint!("{}", c),
                _ => panic!("write char to stream of wrong type"),
            };
            return Maj::nil();
        } else {
            return maj_err(
                Maj::string("{} is not an output stream"),
                maj_list!(stream));
        }
    }

    match get_raw_stream(&mut state,
                         stream.clone(),
                         MajStreamDirection::Out) {
        Ok(mut file) => {
            match write!(file, "{}", c) {
                Ok(_) => {
                    let _ = file.flush();
                    Maj::nil()
                },
                Err(_) => {
                    maj_err(
                        Maj::string(
                            "Could not write to stream {}"),
                        maj_list!(stream))
                }
            }
        },
        Err(expr) => expr,
    }
}

pub fn maj_write_string(mut state: &mut MajState,
                        strn: Gc<Maj>,
                        stream: Gc<Maj>
) -> Gc<Maj> {
    use std::io::Write;
    if !maj_stringp(strn.clone()).to_bool() {
        return maj_err(
            Maj::string("{} is not a string"),
            maj_list!(strn));
    }
    let strn = strn.stringify().unwrap();
    
    if stdstreamp(stream.clone()) {
        if stdstreamdirp(stream.clone(), MajStreamDirection::Out) {
            match stdstreamtype(stream) {
                MajStreamType::Stdout =>
                    print!("{}", strn),
                MajStreamType::Stderr =>
                    eprint!("{}", strn),
                _ => panic!("write string to stream of wrong type"),
            };
            return Maj::nil();
        } else {
            return maj_err(
                Maj::string("{} is not an output stream"),
                maj_list!(stream));
        }
    }

    match get_raw_stream(&mut state,
                         stream.clone(),
                         MajStreamDirection::Out) {
        Ok(mut file) => {
            match write!(file, "{}", strn) {
                Ok(_) => {
                    let _ = file.flush();
                    Maj::nil()
                },
                Err(_) => {
                    maj_err(
                        Maj::string(
                            "Could not write to stream {}"),
                        maj_list!(stream))
                }
            }
        },
        Err(expr) => expr,
    }
}

pub fn maj_write(mut state: &mut MajState,
                 x: Gc<Maj>,
                 stream: Gc<Maj>
) -> Gc<Maj> {
    use crate::printing::maj_format;
    let string = Maj::string(&maj_format(&state, x));
    maj_write_string(&mut state, string, stream)
}

pub fn maj_terpri(mut state: &mut MajState,
                  env: Gc<Maj>) -> Gc<Maj> {
    // Lookup dynamically bound stdout
    let stdout = Maj::symbol(&mut state, "*stdout*");
    let stdout = state.lookup(env.clone(), stdout);
    if maj_errorp(stdout.clone()).to_bool() {
        return stdout;
    }

    maj_write_char(&mut state, Maj::character('\n'),
                   stdout)
}

pub fn maj_display(mut state: &mut MajState,
                   x: Gc<Maj>,
                   env: Gc<Maj>
) -> Gc<Maj> {
    // Lookup dynamically bound stdout
    let stdout = Maj::symbol(&mut state, "*stdout*");
    let stdout = state.lookup(env.clone(), stdout);
    if maj_errorp(stdout.clone()).to_bool() {
        return stdout;
    }

    maj_write(&mut state, x, stdout)
}

pub fn maj_pretty_display(mut state: &mut MajState,
                          x: Gc<Maj>,
                          env: Gc<Maj>
) -> Gc<Maj> {
    use crate::printing::maj_pretty_format;
    // Lookup dynamically bound stdout
    let stdout = Maj::symbol(&mut state, "*stdout*");
    let stdout = state.lookup(env.clone(), stdout);
    if maj_errorp(stdout.clone()).to_bool() {
        return stdout;
    }

    let string = maj_pretty_format(&state, x);
    let string = Maj::string(&string);
    maj_write_string(&mut state, string, stdout)
}

pub fn maj_print(
    mut state: &mut MajState,
    fmt: Gc<Maj>,
    rest: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    // Lookup dynamically bound stdout
    let stdout = Maj::symbol(&mut state, "*stdout*");
    let stdout = state.lookup(env.clone(), stdout);
    if maj_errorp(stdout.clone()).to_bool() {
        return stdout;
    }

    let formatted = maj_format_prim(&state, fmt, rest);
    if maj_errorp(formatted.clone()).to_bool() {
        formatted
    } else {
        maj_write_string(&mut state, formatted,
                         stdout.clone());
        maj_write_char(&mut state, Maj::character('\n'),
                   stdout)
    }
}

pub fn maj_load(
    mut state: &mut MajState,
    env: Gc<Maj>,
    path: Gc<Maj>
) -> Gc<Maj> {
    use crate::reader::parser::maj_parse;
    use crate::reader::tokenizer::maj_tokenize_file;
    use crate::evaluator::maj_eval;
    match path.clone().stringify() {
        Some(pathstr) => {
            let mut buffer = String::new();
            match maj_tokenize_file(&pathstr, &mut buffer) {
                Ok(tokens) => {
                    match maj_parse(&mut state, tokens.clone()) {
                        Ok(expressions) => {
                            // TODO: Iterate over forms and yield errors
                            // depending on them
                            let results = maj_eval(&mut state, Maj::cons(
                                Maj::do_sym(), expressions), env);
                            if maj_errorp(results.clone()).to_bool() {
                                maj_err(
                                    Maj::string(
                                        "On evaluation of file {}: {}"),
                                    maj_list!(path, results))
                            } else {
                                results
                            }
                        },
                        Err(msg) => {
                            maj_err(
                                Maj::string("While parsing file {}: {}"),
                                maj_list!(path, Maj::string(msg)))
                        },
                    }
                },
                Err((line, msg)) => {
                    if line != 0 {
                        maj_err(
                            Maj::string("While reading file {}:{}: {}"),
                            maj_list!(path, Maj::integer(line as i64),
                                      Maj::string(msg)))
                    } else {
                        maj_err(
                            Maj::string("While reading file {}: {}"),
                            maj_list!(path, Maj::string(msg)))
                    }
                },
            }
        },
        None => maj_err(Maj::string("{} is not a string"),
                        maj_list!(path)),
    }
}

pub fn maj_vec_type(mut state: &mut MajState,
                    vec: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajVector;
    if !maj_vectorp(vec.clone()).to_bool() {
        return maj_err(
            Maj::string("{} is not a vector"),
            maj_list!(vec));
    }
    if let Maj::Vector(v) = &*vec {
        return Maj::symbol(&mut state, match v {
            MajVector::Integer(_) => "integer",
            MajVector::Float(_)   => "float",
            MajVector::Char(_)    => "char",
            MajVector::Any(_)     => "any",
        });
    }
    panic!("vec-type: Unknown vector type");
}

pub fn maj_vec_push(mut state: &mut MajState, x: Gc<Maj>, vec: Gc<Maj>) -> Gc<Maj> {
    let pos = maj_vec_length(vec.clone());
    if maj_errorp(pos.clone()).to_bool() {
        return pos;
    }

    maj_vec_insert(&mut state, pos, x, vec)
}

pub fn maj_vec_insert(mut state: &mut MajState,
                      pos: Gc<Maj>,
                      x: Gc<Maj>,
                      vec: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajVector;

    if !maj_vectorp(vec.clone()).to_bool() {
        return maj_err(
            Maj::string("{} is not a vector"),
            maj_list!(vec));
    }

    let xtype = maj_type(&mut state, x.clone());
    let vectype = maj_vec_type(&mut state, vec.clone());
    let any = Maj::symbol(&mut state, "any");

    let index =
        match pos.clone().to_integer() {
            Some(x) => {
                if x < 0 {
                    return maj_err(
                        Maj::string(
                            "Index {} is out of bounds in {}"),
                        maj_list!(pos, vec));
                }
                x as usize
            },
            None => {
                return maj_err(
                    Maj::string("{} is not an integer"),
                    maj_list!(pos));
            },
        };

    if !maj_eq(vectype.clone(), any).to_bool()
        && !maj_eq(xtype.clone(), vectype.clone()).to_bool() {
        return maj_err(
            Maj::string(
                "{} has type {}, which is incompatible with insertion on vector of type {}"),
            maj_list!(x, xtype, vectype));
    }

    if let Maj::Vector(v) = &*vec.clone() {
        match v {
            MajVector::Integer(v) => {
                let val = x.to_integer().unwrap();
                let len = v.borrow().len();
                if index > len {
                    maj_err(
                        Maj::string(
                            "Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    v.borrow_mut().insert(index, val);
                    vec
                }
            },
            MajVector::Float(v) => {
                let val = x.to_float().unwrap();
                let len = v.borrow().len();
                if index > len {
                    maj_err(
                        Maj::string(
                            "Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    v.borrow_mut().insert(index, val);
                    vec
                }
            },
            MajVector::Char(s) => {
                let val = x.to_char().unwrap();
                let len = s.borrow().chars().count();
                if index > len {
                    maj_err(
                        Maj::string(
                            "Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    s.borrow_mut().insert(index, val);
                    vec
                }
            },
            MajVector::Any(v) => {
                let len = v.borrow().len();
                if index > len {
                    maj_err(
                        Maj::string(
                            "Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    v.borrow_mut().insert(index, x.clone());
                    vec
                }
            }
        }
    } else {
        panic!("vec-insert: Not a vector");
    }
}

fn sym_to_vectype(mut state: &mut MajState,
                  vtype: Gc<Maj>) -> MajVectorType {
    if maj_eq(vtype.clone(),
              Maj::symbol(&mut state, "integer")).to_bool() {
        MajVectorType::Integer
    } else if maj_eq(
        vtype.clone(),
        Maj::symbol(&mut state, "float")).to_bool() {
        MajVectorType::Float
    } else if maj_eq(
        vtype.clone(),
        Maj::symbol(&mut state, "char")).to_bool() {
        MajVectorType::Char
    } else {
        MajVectorType::Any
    }
}

pub fn maj_vec_coerce(mut state: &mut MajState,
                      vtype: Gc<Maj>,
                      vec: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajVector;
    if !maj_vectorp(vec.clone()).to_bool() {
        return maj_err(
            Maj::string("{} is not a vector"),
            maj_list!(vec));
    }
    let intended_type = sym_to_vectype(&mut state, vtype.clone());
    let newvec = Maj::vector(intended_type);
    let is_any_intended = maj_eq(vtype.clone(),
                                 Maj::symbol(&mut state, "any"))
        .to_bool();

    if let Maj::Vector(v) = &*vec {
        match v {
            MajVector::Integer(v) => {
                for n in v.borrow().iter() {
                    let num = Maj::integer(*n);
                    if !is_any_intended {
                        let num = maj_number_coerce(
                            &mut state, vtype.clone(), num.clone());
                        if maj_errorp(num.clone()).to_bool() {
                            return num;
                        }
                    }
                    let result =
                        maj_vec_push(&mut state,
                                     num.clone(),
                                     newvec.clone());
                    if maj_errorp(result.clone()).to_bool() {
                        return result;
                    }
                }
            },
            MajVector::Float(v) => {
                for n in v.borrow().iter() {
                    let num = Maj::float(*n);
                    if !is_any_intended {
                        let num = maj_number_coerce(
                            &mut state, vtype.clone(), num.clone());
                        if maj_errorp(num.clone()).to_bool() {
                            return num;
                        }
                    }
                    let result =
                        maj_vec_push(&mut state,
                                     num.clone(),
                                     newvec.clone());
                    if maj_errorp(result.clone()).to_bool() {
                        return result;
                    }
                }
            },
            MajVector::Char(s) => {
                for c in s.borrow().chars() {
                    let result =
                        maj_vec_push(&mut state,
                                     Maj::character(c),
                                     newvec.clone());
                    if maj_errorp(result.clone()).to_bool() {
                        return result;
                    }
                }
            },
            MajVector::Any(v) => {
                for elt in v.borrow().iter() {
                    let result =
                        maj_vec_push(&mut state,
                                     elt.clone(),
                                     newvec.clone());
                    if maj_errorp(result.clone()).to_bool() {
                        return result;
                    }
                }
            }
        }
    } else {
        panic!("vec-coerce: Not a vector");
    }
    newvec
}

fn vectype_compatible(mut state: &mut MajState,
                      vtype: MajVectorType,
                      xtype: Gc<Maj>) -> bool {
    match vtype {
        MajVectorType::Any => true,
        MajVectorType::Integer =>
            maj_eq(xtype,
                   Maj::symbol(&mut state, "integer"))
            .to_bool(),
        MajVectorType::Float =>
            maj_eq(xtype,
                   Maj::symbol(&mut state, "float"))
            .to_bool(),
        MajVectorType::Char =>
            maj_eq(xtype,
                   Maj::symbol(&mut state, "char"))
            .to_bool(),
    }
}

fn best_vectype(mut state: &mut MajState,
                xtype: Gc<Maj>) -> MajVectorType {
    let integer = Maj::symbol(&mut state, "integer");
    let float   = Maj::symbol(&mut state, "float");
    let charsym = Maj::symbol(&mut state, "char");
    
    let sym =
        if maj_eq(xtype.clone(), integer).to_bool()
        || maj_eq(xtype.clone(), float).to_bool()
        || maj_eq(xtype.clone(), charsym).to_bool() {
            xtype
        } else {
            Maj::symbol(&mut state, "any")
        };
    sym_to_vectype(&mut state, sym)
}

pub fn maj_vector(mut state: &mut MajState, rest: Gc<Maj>) -> Gc<Maj> {
    if maj_nilp(rest.clone()).to_bool() {
        Maj::vector(MajVectorType::Any)
    } else {
        let mut iter = rest.clone();

        let first = maj_car(iter.clone());
        let fsttype = maj_type(&mut state, first);
        let mut vector = Maj::vector(
            best_vectype(&mut state, fsttype));

        while !maj_nilp(iter.clone()).to_bool() {
            let first = maj_car(iter.clone());
            let fsttype = maj_type(&mut state, first.clone());

            let vtype = maj_vec_type(&mut state, vector.clone());
            let vtype = sym_to_vectype(&mut state, vtype);

            if !vectype_compatible(
                &mut state, vtype, fsttype) {
                let any = Maj::symbol(&mut state, "any");
                vector = maj_vec_coerce(
                    &mut state, any, vector);
                if maj_errorp(vector.clone()).to_bool() {
                    return vector;
                }
            }
            let res =
                maj_vec_push(&mut state, first, vector.clone());
            if maj_errorp(res.clone()).to_bool() {
                return res;
            }
            iter = maj_cdr(iter);
        }
        vector
    }
}

pub fn maj_vec_length(vec: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajVector;
    if let Maj::Vector(v) = &*vec.clone() {
        Maj::integer(
            match v {
                MajVector::Integer(v) => v.borrow().len(),
                MajVector::Float(v) => v.borrow().len(),
                MajVector::Char(s) => s.borrow().chars().count(),
                MajVector::Any(v) => v.borrow().len(),
            } as i64)
    } else {
        maj_err(
            Maj::string("{} is not a vector"),
            maj_list!(vec))
    }
}

pub fn maj_vec_pop(vec: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajVector;
    if let Maj::Vector(vv) = &*vec.clone() {
        match vv {
            MajVector::Integer(v) => {
                match v.borrow_mut().pop() {
                    Some(val) => Maj::integer(val),
                    None => Maj::nil(),
                }
            },
            MajVector::Float(v) => {
                match v.borrow_mut().pop() {
                    Some(val) => Maj::float(val),
                    None => Maj::nil(),
                }
            },
            MajVector::Char(s) => {
                match s.borrow_mut().pop() {
                    Some(val) => Maj::character(val),
                    None => Maj::nil(),
                }
            },
            MajVector::Any(v) => {
                match v.borrow_mut().pop() {
                    Some(val) => val,
                    None => Maj::nil(),
                }
            }
        }
    } else {
        maj_err(
            Maj::string("{} is not a vector"),
            maj_list!(vec))
    }
}

pub fn maj_vec_deq(vec: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajVector;
    if let Maj::Vector(vv) = &*vec.clone() {
        match vv {
            MajVector::Integer(v) => {
                if v.borrow().is_empty() {
                    Maj::nil()
                } else {
                    Maj::integer(
                        v.borrow_mut().remove(0))
                }
            },
            MajVector::Float(v) => {
                if v.borrow().is_empty() {
                    Maj::nil()
                } else {
                    Maj::float(
                        v.borrow_mut().remove(0))
                }
            },
            MajVector::Char(s) => {
                if s.borrow().is_empty() {
                    Maj::nil()
                } else {
                    Maj::character(
                        s.borrow_mut().remove(0))
                }
            },
            MajVector::Any(v) => {
                if v.borrow().is_empty() {
                    Maj::nil()
                } else {
                    v.borrow_mut().remove(0)
                }
            }
        }
    } else {
        maj_err(
            Maj::string("{} is not a vector"),
            maj_list!(vec))
    }
}

pub fn maj_vec_at(x: Gc<Maj>, vec: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajVector;
    use crate::axioms::predicates::maj_integerp;

    if !maj_integerp(x.clone()).to_bool() {
        return maj_err(
            Maj::string("{} is not an integer"),
            maj_list!(x));
    }

    let index = x.to_integer().unwrap() as usize;

    if let Maj::Vector(vv) = &*vec.clone() {
        match vv {
            MajVector::Integer(v) => {
                match v.borrow().get(index) {
                    Some(val) => Maj::integer(*val),
                    None => maj_err(
                        Maj::string("Index {} is out of bounds in {}"),
                        maj_list!(x.clone(), vec)),
                }
            },
            MajVector::Float(v) => {
                match v.borrow().get(index) {
                    Some(val) => Maj::float(*val),
                    None => maj_err(
                        Maj::string("Index {} is out of bounds in {}"),
                        maj_list!(x.clone(), vec)),
                }
            },
            MajVector::Char(s) => {
                match s.borrow().chars().nth(index) {
                    Some(val) => Maj::character(val),
                    None => maj_err(
                        Maj::string("Index {} is out of bounds in {}"),
                        maj_list!(x.clone(), vec)),
                }
            },
            MajVector::Any(v) => {
                match v.borrow().get(index) {
                    Some(val) => val.clone(),
                    None => maj_err(
                        Maj::string("Index {} is out of bounds in {}"),
                        maj_list!(x.clone(), vec)),
                }
            }
        }
    } else {
        maj_err(
            Maj::string("{} is not a vector"),
            maj_list!(vec))
    }
}

fn replace_string_char(strn: &mut String, index: usize, rep: char) {
    let mut vec = strn.chars().collect::<Vec<_>>();
    vec[index] = rep;
    *strn = vec.iter().collect::<String>();
}

pub fn maj_vec_set(
    pos: Gc<Maj>, x: Gc<Maj>, vec: Gc<Maj>
) -> Gc<Maj> {
    use crate::core::types::MajVector;
    use crate::axioms::predicates::{
        maj_integerp,
        maj_floatp,
        maj_charp
    };
    let index =
        match pos.clone().to_integer() {
            Some(x) => {
                if x < 0 {
                    return maj_err(
                        Maj::string(
                            "Index {} is out of bounds in {}"),
                        maj_list!(pos, vec));
                }
                x as usize
            },
            None => {
                return maj_err(
                    Maj::string("{} is not an integer"),
                    maj_list!(pos));
            },
        };

    if let Maj::Vector(vv) = &*vec.clone() {
        match vv {
            MajVector::Integer(v) => {
                if !maj_integerp(x.clone()).to_bool() {
                    return maj_err(
                        Maj::string(
                            "{} is not type-compatible with vector {}"),
                        maj_list!(x.clone(), vec.clone()));
                }
                let x = x.to_integer().unwrap();
                let len = v.borrow().len();
                if index >= len {
                    maj_err(
                        Maj::string("Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    v.borrow_mut()[index] = x;
                    vec
                }
            },
            MajVector::Float(v) => {
                if!maj_floatp(x.clone()).to_bool() {
                    return maj_err(
                        Maj::string(
                            "{} is not type-compatible with vector {}"),
                        maj_list!(x.clone(), vec.clone()));
                }
                let x = x.to_float().unwrap();
                let len = v.borrow().len();
                if index >= len {
                    maj_err(
                        Maj::string("Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    v.borrow_mut()[index] = x;
                    vec
                }
            },
            MajVector::Char(s) => {
                if!maj_charp(x.clone()).to_bool() {
                    maj_err(
                        Maj::string(
                            "{} is not type-compatible with vector {}"),
                        maj_list!(x.clone(), vec.clone()));
                }
                let c = x.clone().to_char().unwrap();
                let len = s.borrow().len();
                if index >= len {
                    maj_err(
                        Maj::string("Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    replace_string_char(&mut s.borrow_mut(), index, c);
                    vec
                }
            },
            MajVector::Any(v) => {
                let len = v.borrow().len();
                if index >= len {
                    return maj_err(
                        Maj::string("Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    v.borrow_mut()[index] = x.clone();
                    vec
                }
            },
        }
    } else {
        maj_err(
            Maj::string("{} is not a vector"),
            maj_list!(vec))
    }
}

pub fn maj_vec_remove(pos: Gc<Maj>, vec: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajVector;
    let index =
        match pos.clone().to_integer() {
            Some(x) => {
                if x < 0 {
                    return maj_err(
                        Maj::string(
                            "Index {} is out of bounds in {}"),
                        maj_list!(pos, vec));
                }
                x as usize
            },
            None => {
                return maj_err(
                    Maj::string("{} is not an integer"),
                    maj_list!(pos));
            },
        };
    if let Maj::Vector(vv) = &*vec {
        match vv {
            MajVector::Integer(v) => {
                let len = v.borrow().len();
                if index >= len {
                    maj_err(
                        Maj::string(
                            "Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    let value = v.borrow_mut().remove(index);
                    Maj::integer(value)
                }
            },
            MajVector::Float(v) => {
                let len = v.borrow().len();
                if index >= len {
                    maj_err(
                        Maj::string(
                            "Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    let value = v.borrow_mut().remove(index);
                    Maj::float(value)
                }
            },
            MajVector::Char(s) => {
                let len = s.borrow().chars().count();
                if index >= len {
                    maj_err(
                        Maj::string(
                            "Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    let value = s.borrow_mut().remove(index);
                    Maj::character(value)
                }
            },
            MajVector::Any(v) => {
                let len = v.borrow().len();
                if index >= len {
                    maj_err(
                        Maj::string(
                            "Index {} is out of bounds in {}"),
                        maj_list!(pos, vec))
                } else {
                    v.borrow_mut().remove(index)
                }
            },
        }
    } else {
        maj_err(Maj::string("{} is not a vector"),
                maj_list!(vec))
    }
}

pub fn maj_gc() -> Gc<Maj> {
    use gc::force_collect;
    force_collect();
    Maj::nil()
}

fn maj_print_env(
    mut state: &mut MajState,
    args: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    use crate::printing::maj_format_env;
    let is_global =
        maj_eq(maj_car(args.clone()),
               Maj::symbol(&mut state, "global"))
        .to_bool();
    let is_lexical =
        maj_eq(maj_car(args.clone()),
               Maj::symbol(&mut state, "lexical"))
        .to_bool();

    if is_global {
        println!("{}", state);
        Maj::nil()
    } else if is_lexical {
        println!("{}", maj_format_env(&state, env));
        Maj::nil()
    } else {
        maj_err(Maj::string("Unknown environment type {}"),
                maj_list!(maj_car(args)))
    }
}

fn maj_stack_state() -> Gc<Maj> {
    match stacker::remaining_stack() {
        Some(n) => {
            println!("Remaining stack: {}B (roughly {}KB)",
                     n, n / 1024);
            use std::convert::TryInto;
            match n.try_into() {
                Ok(n) => Maj::integer(n),
                Err(_) => Maj::t()
            }
        },
        None => {
            println!("Couldn't query stack state");
            Maj::nil()
        }
    }
}

pub fn maj_gen_primitives(state: &mut MajState) {
    use crate::axioms::{ MajPrimFn, MajPrimArgs };
    let primitives: Vec<(&str, MajPrimArgs, MajPrimFn)> = vec![
        ("cons", MajPrimArgs::Required(2), |_, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_cons(first, second)
        }),
        ("car", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_car(first)
        }),
        ("cdr", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_cdr(first)
        }),
        ("copy", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_copy(first)
        }),
        ("length", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_length(first)
        }),
        ("depth", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_depth(first)
        }),
        ("type", MajPrimArgs::Required(1), |mut state, args, _| {
            maj_destructure_args!(args, first);
            maj_type(&mut state, first)
        }),
        ("intern", MajPrimArgs::Required(1), |mut state, args, _| {
            maj_destructure_args!(args, first);
            maj_intern(&mut state, first)
        }),
        ("name", MajPrimArgs::Required(1), |mut state, args, _| {
            maj_destructure_args!(args, first);
            maj_name(&mut state, first)
        }),
        ("get-environment", MajPrimArgs::Required(1),
         |mut state, args, env| {
            maj_destructure_args!(args, first);
            maj_get_environment(&mut state, first, env)
        }),
        ("coin", MajPrimArgs::None, |_, _, _| maj_coin()),
        ("sys", MajPrimArgs::Variadic(1), |_, args, _| {
            maj_destructure_args!(args, first, rest);
            maj_sys(first, rest)
        }),
        ("format", MajPrimArgs::Variadic(1), |state, args, _| {
            maj_destructure_args!(args, first, rest);
            maj_format_prim(&state, first, rest)
        }),
        ("err", MajPrimArgs::Variadic(1), |_, args, _| {
            maj_destructure_args!(args, first, rest);
            maj_err(first, rest)
        }),
        ("warn", MajPrimArgs::Variadic(1), |mut state, args, env| {
            maj_destructure_args!(args, first, rest);
            maj_warn(&mut state, first, rest, env)
        }),
        ("list", MajPrimArgs::Variadic(0),
         |_, args, _| maj_list(args)),
        ("append", MajPrimArgs::Variadic(0),
         |_, args, _| maj_append(args)),
        ("last", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_last(first)
        }),
        ("reverse", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_reverse(first)
        }),
        ("nth", MajPrimArgs::Required(2), |_, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_nth(first, second)
        }),
        ("nthcdr", MajPrimArgs::Required(2), |_, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_nthcdr(first, second)
        }),
        ("macroexpand-1", MajPrimArgs::Required(1),
         |mut state, args, env| {
             maj_destructure_args!(args, first);
             let (result, _) =
                 maj_macroexpand_1(&mut state, first, env);
             result
         }),
        ("not", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_not(first)
        }),
        ("gensym", MajPrimArgs::None,
         |mut state, _, _| maj_gensym(&mut state)),
        ("iota", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_iota(first)
         }),

        // Number functions
        ("number-coerce", MajPrimArgs::Required(2),
         |mut state, args, _| {
             maj_destructure_args!(args, first, rest, second);
             maj_number_coerce(&mut state, first, second)
         }),
        ("real-part", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_real_part(first)
        }),
        ("imag-part", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_imag_part(first)
        }),
        ("numer", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_numer(first)
        }),
        ("denom", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_denom(first)
        }),
        ("richest-number-type", MajPrimArgs::Required(2),
         |mut state, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_richest_number_type(&mut state, first, second)
        }),
        ("rich-number-coerce", MajPrimArgs::Required(2),
         |mut state, args, _| {
             maj_destructure_args!(args, first, rest, second);
             maj_rich_number_coerce(&mut state, first, second)
         }),
        ("=", MajPrimArgs::Variadic(2),
         |mut state, args, env| {
             maj_destructure_args!(args, first, r, second, rest);
             maj_arithm_eq(&mut state, env, first, second, rest)
         }),
        ("float=", MajPrimArgs::Required(2),
         |mut state, args, env| {
             maj_destructure_args!(args, first, r, second);
             maj_arithm_floateq(&mut state, env, first, second)
         }),
        (">", MajPrimArgs::Variadic(2),
         |mut state, args, _| {
             maj_destructure_args!(args, first, r, second, rest);
             maj_arithm_greater(&mut state, first, second, rest)
         }),
        ("<", MajPrimArgs::Variadic(2),
         |mut state, args, _| {
             maj_destructure_args!(args, first, r, second, rest);
             maj_arithm_lesser(&mut state, first, second, rest)
         }),
        (">=", MajPrimArgs::Variadic(2),
         |mut state, args, env| {
             maj_destructure_args!(args, first, r, second, rest);
             maj_arithm_geq(&mut state, env, first, second, rest)
         }),
        ("<=", MajPrimArgs::Variadic(2),
         |mut state, args, env| {
             maj_destructure_args!(args, first, r, second, rest);
             maj_arithm_leq(&mut state, env, first, second, rest)
         }),
        ("+", MajPrimArgs::Variadic(0),
         |mut state, args, env| {
             maj_arithm_plus(&mut state, env, args)
         }),
        ("-", MajPrimArgs::Variadic(0),
         |mut state, args, env| {
             maj_arithm_minus(&mut state, env, args)
         }),
        ("*", MajPrimArgs::Variadic(0),
         |mut state, args, env| {
             maj_arithm_times(&mut state, env, args)
         }),
        ("/", MajPrimArgs::Variadic(0),
         |mut state, args, env| {
             maj_arithm_divide(&mut state, env, args)
         }),

        // Stream functions
        ("open-stream", MajPrimArgs::Required(2), |mut state, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_open_stream(&mut state, first, second)
        }),
        ("close-stream", MajPrimArgs::Required(1), |mut state, args, _| {
            maj_destructure_args!(args, first);
            maj_close_stream(&mut state, first)
        }),
        ("stat", MajPrimArgs::Required(1), |mut state, args, _| {
            maj_destructure_args!(args, first);
            maj_stat(&mut state, first)
        }),
        ("read-char", MajPrimArgs::Required(1), |mut state, args, _| {
            maj_destructure_args!(args, first);
            maj_read_char(&mut state, first)
        }),
        ("peek-char", MajPrimArgs::Required(1), |mut state, args, _| {
            maj_destructure_args!(args, first);
            maj_peek_char(&mut state, first)
        }),
        ("write-char", MajPrimArgs::Required(2), |mut state, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_write_char(&mut state, first, second)
        }),
        ("write-string", MajPrimArgs::Required(2), |mut state, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_write_string(&mut state, first, second)
        }),
        ("write", MajPrimArgs::Required(2), |mut state, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_write(&mut state, first, second)
        }),
        ("terpri", MajPrimArgs::None, |mut state, _, env| {
            maj_terpri(&mut state, env)
        }),
        ("display", MajPrimArgs::Required(1), |mut state, args, env| {
            maj_destructure_args!(args, first);
            maj_display(&mut state, first, env)
        }),
        ("pretty-display", MajPrimArgs::Required(1),
         |mut state, args, env| {
             maj_destructure_args!(args, first);
             maj_pretty_display(&mut state, first, env)
         }),
        ("print", MajPrimArgs::Variadic(1), |mut state, args, env| {
            maj_destructure_args!(args, first, rest);
            maj_print(&mut state, first, rest, env)
        }),
        ("load", MajPrimArgs::Required(1), |mut state, args, env| {
            maj_destructure_args!(args, first);
            maj_load(&mut state, env, first)
        }),

        // Vector functions
        ("vec-type", MajPrimArgs::Required(1), |mut state, args, _| {
            maj_destructure_args!(args, first);
            maj_vec_type(&mut state, first)
        }),
        ("vec-push", MajPrimArgs::Required(2), |mut state, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_vec_push(&mut state, first, second)
        }),
        ("vec-insert", MajPrimArgs::Required(3), |mut state, args, _| {
            maj_destructure_args!(args, first, rest, second,
                                  sndrst, third);
            maj_vec_insert(&mut state, first, second, third)
        }),
        ("vec-coerce", MajPrimArgs::Required(2), |mut state, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_vec_coerce(&mut state, first, second)
        }),
        ("vector", MajPrimArgs::Variadic(0), |mut state, args, _| {
            maj_vector(&mut state, args)
        }),
        ("vec-length", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_vec_length(first)
        }),
        ("vec-pop", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_vec_pop(first)
        }),
        ("vec-deq", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_vec_deq(first)
        }),
        ("vec-at", MajPrimArgs::Required(2), |_, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_vec_at(first, second)
        }),
        ("vec-set", MajPrimArgs::Required(3), |_, args, _| {
            maj_destructure_args!(args, first, rest, second,
                                  snd_rest, third);
            maj_vec_set(first, second, third)
        }),
        ("vec-remove", MajPrimArgs::Required(2), |_, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_vec_remove(first, second)
        }),

        // Non-standard functions
        ("gc", MajPrimArgs::None, |_, _, _| maj_gc()),
        ("print-env", MajPrimArgs::Required(1), maj_print_env),
        ("stack-state", MajPrimArgs::None, |_, _, _| maj_stack_state()),
    ];

    for primitive in primitives.iter() {
        let (name, arity, f) = primitive;
        state.register_primitive(name, *arity, *f);
    }
}
