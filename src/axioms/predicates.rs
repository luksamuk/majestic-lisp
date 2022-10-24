use gc::Gc;
use crate::maj_list;
use crate::core::Maj;
use crate::core::MajState;
use super::primitives::{
    maj_car,
    maj_cdr,
    maj_err
};

pub fn maj_symbolp(x: Gc<Maj>) -> Gc<Maj> {
    if let Maj::Sym(_) = *x {
        return Maj::t();
    }
    Maj::nil()
}

pub fn maj_eq(x: Gc<Maj>, y: Gc<Maj>) -> Gc<Maj> {
    if let Maj::Sym(idx) = *x {
        if let Maj::Sym(idy) = *y {
            if idx == idy {
                return Maj::t();
            }
        }
    }
    Maj::nil()
}

pub fn maj_nilp(x: Gc<Maj>) -> Gc<Maj> {
    maj_eq(x, Maj::nil())
}

pub fn maj_consp(x: Gc<Maj>) -> Gc<Maj> {
    if let Maj::Cons { car: _, cdr: _ } = *x {
        return Maj::t();
    }
    Maj::nil()
}

pub fn maj_atomp(x: Gc<Maj>) -> Gc<Maj> {
    if let Maj::Cons { car: _, cdr: _ } = *x {
        return Maj::nil()
    }
    Maj::t()
}

pub fn maj_charp(x: Gc<Maj>) -> Gc<Maj> {
    if let Maj::Char(_) = *x {
        return Maj::t();
    }
    Maj::nil()
}

pub fn maj_char_equals(x: Gc<Maj>, y: Gc<Maj>) -> Gc<Maj> {
    if let Maj::Char(xc) = *x.clone() {
        if let Maj::Char(yc) = *y.clone() {
            if xc == yc {
                Maj::t()
            } else {
                Maj::nil()
            }
        } else {
            maj_err(Maj::string("{} is not a character"),
                    maj_list!(y))
        }
    } else {
        maj_err(Maj::string("{} is not a character"),
                maj_list!(x))
    }
}

pub fn maj_streamp(x: Gc<Maj>) -> Gc<Maj> {
    if let Maj::Stream(_) = *x {
        return Maj::t();
    }
    Maj::nil()
}

pub fn maj_numberp(x: Gc<Maj>) -> Gc<Maj> {
    if let Maj::Number(_) = &*x {
        return Maj::t();
    }
    Maj::nil()
}

pub fn maj_integerp(x: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajNumber;
    if let Maj::Number(num) = &*x {
        if let MajNumber::Integer(_) = num {
            return Maj::t();
        }
    }
    Maj::nil()
}

pub fn maj_floatp(x: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajNumber;
    if let Maj::Number(num) = &*x {
        if let MajNumber::Float(_) = num {
            return Maj::t();
        }
    }
    Maj::nil()
}

pub fn maj_fractionp(x: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajNumber;
    if let Maj::Number(num) = &*x {
        if let MajNumber::Fraction(_, _) = num {
            return Maj::t();
        }
    }
    Maj::nil()
}

pub fn maj_complexp(x: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajNumber;
    if let Maj::Number(num) = &*x {
        if let MajNumber::Complex {
            real: _,
            imag: _
        } = num {
            return Maj::t();
        }
    }
    Maj::nil()
}

pub fn maj_vectorp(x: Gc<Maj>) -> Gc<Maj> {
    if let Maj::Vector(_) = &*x {
        Maj::t()
    } else {
        Maj::nil()
    }
}

pub fn maj_id(x: Gc<Maj>, y: Gc<Maj>) -> Gc<Maj> {
    match *x {
        Maj::Sym(_) => {
            if let Maj::Sym(_) = *y {
                return maj_eq(x, y);
            }
        },
        Maj::Char(chrx) => {
            if let Maj::Char(chry) = *y {
                if chrx == chry {
                    return Maj::t();
                }
            }
        },
        _ => {
            let x_ptr = Gc::into_raw(x);
            let y_ptr = Gc::into_raw(y);
            if x_ptr == y_ptr {
                return Maj::t();
            }
        }
    }
    Maj::nil()
}

pub fn maj_proper_list_p(x: Gc<Maj>) -> Gc<Maj> {
    let is_cons = maj_consp(x.clone()).to_bool();
    let is_nil  = maj_nilp(x.clone()).to_bool();

    if !is_cons && !is_nil {
        return Maj::nil();
    }

    let mut itr = x.clone();
    while !maj_nilp(itr.clone()).to_bool()  {
        if let Maj::Cons { car: _, cdr } = &*itr.clone() {
            itr = cdr.clone();
        } else {
            return Maj::nil();
        }
    }
    Maj::t()
}

pub fn maj_stringp(x: Gc<Maj>) -> Gc<Maj> {
    use crate::core::types::MajVector;
    if let Maj::Vector(v) = &*x {
        if let MajVector::Char(_) = &v {
            return Maj::t();
        }
    }
    Maj::nil()
}

pub fn maj_literalp(x: Gc<Maj>) -> Gc<Maj> {
    if maj_proper_list_p(x.clone()).to_bool() &&
        maj_eq(maj_car(x.clone()), Maj::lit()).to_bool() {
            return Maj::t();
        }
    Maj::nil()
}

pub fn maj_primitivep(x: Gc<Maj>) -> Gc<Maj> {
    if maj_literalp(x.clone()).to_bool() {
        let sym = maj_car(maj_cdr(x));
        if maj_eq(sym, Maj::prim()).to_bool() {
            return Maj::t();
        }
    }
    Maj::nil()
}

pub fn maj_closurep(x: Gc<Maj>) -> Gc<Maj> {
    if maj_literalp(x.clone()).to_bool() {
        let sym = maj_car(maj_cdr(x));
        if maj_eq(sym, Maj::closure()).to_bool() {
            return Maj::t();
        }
    }
    Maj::nil()
}

pub fn maj_functionp(x: Gc<Maj>) -> Gc<Maj> {
    if maj_primitivep(x.clone()).to_bool()
        || maj_closurep(x).to_bool()
    {
        return Maj::t();
    }
    Maj::nil()
}

pub fn maj_macrop(x: Gc<Maj>) -> Gc<Maj> {
    if maj_literalp(x.clone()).to_bool() {
        let sym = maj_car(maj_cdr(x));
        if maj_eq(sym, Maj::macro_sym()).to_bool() {
            return Maj::t();
        }
    }
    Maj::nil()
}

pub fn maj_errorp(x: Gc<Maj>) -> Gc<Maj> {
    if maj_proper_list_p(x.clone()).to_bool() {
        let sym = maj_car(maj_cdr(x));
        if maj_eq(sym, Maj::error()).to_bool() {
            return Maj::t();
        }
    }
    Maj::nil()
}

pub fn maj_zerop(
    mut state: &mut MajState,
    env: Gc<Maj>,
    x: Gc<Maj>
) -> Gc<Maj> {
    use crate::axioms::primitives::maj_arithm_eq;
    maj_arithm_eq(&mut state, env,
                  Maj::integer(0), x, Maj::nil())
}

pub fn maj_gen_predicates(state: &mut MajState) {
    use crate::axioms::{ MajPrimFn, MajPrimArgs };
    use crate::maj_destructure_args;

    let predicates: Vec<(&str, MajPrimArgs, MajPrimFn)> = vec![
        ("symbolp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_symbolp(first)
        }),
        ("eq", MajPrimArgs::Required(2), |_, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_eq(first, second)
        }),
        ("nilp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_nilp(first)
        }),
        ("consp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_consp(first)
        }),
        ("atomp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_atomp(first)
        }),
        ("charp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_charp(first)
        }),
        ("char=", MajPrimArgs::Required(2), |_, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_char_equals(first, second)
        }),
        ("streamp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_streamp(first)
        }),
        ("numberp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_numberp(first)
        }),
        ("integerp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_integerp(first)
        }),
        ("floatp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_floatp(first)
        }),
        ("fractionp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_fractionp(first)
        }),
        ("complexp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_complexp(first)
        }),
        ("vectorp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_vectorp(first)
        }),
        ("id", MajPrimArgs::Required(2), |_, args, _| {
            maj_destructure_args!(args, first, rest, second);
            maj_id(first, second)
        }),
        ("proper-list-p", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_proper_list_p(first)
        }),
        ("stringp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_stringp(first)
        }),
        ("literalp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_literalp(first)
        }),
        ("primitivep", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_primitivep(first)
        }),
        ("closurep", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_closurep(first)
        }),
        ("functionp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_functionp(first)
        }),
        ("macrop", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_macrop(first)
        }),
        ("errorp", MajPrimArgs::Required(1), |_, args, _| {
            maj_destructure_args!(args, first);
            maj_errorp(first)
        }),
        ("zerop", MajPrimArgs::Required(1),
         |mut state, args, env| {
             maj_destructure_args!(args, first);
             maj_zerop(&mut state, env, first)
         }),
    ];

    for predicate in predicates.iter() {
        let (name, arity, f) = predicate;
        state.register_primitive(name, *arity, *f);
    }
}
