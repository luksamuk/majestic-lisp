use super::Maj;
use gc::Gc;

use crate::axioms::predicates::{
    maj_nilp,
    maj_eq,
    maj_proper_list_p,
    maj_symbolp
};

pub fn maj_env_push(env: Gc<Maj>, sym: Gc<Maj>, val: Gc<Maj>) -> Gc<Maj> {
    let is_env = maj_proper_list_p(env.clone()).to_bool();
    let is_sym = maj_symbolp(sym.clone()).to_bool();
    
    if !is_env || !is_sym {
        Maj::nil()
    } else {
        Maj::cons(
            Maj::cons(sym, val),
            env)
    }
}

pub fn maj_env_assoc(env: Gc<Maj>, sym: Gc<Maj>) -> Gc<Maj> {
    use crate::axioms::primitives::maj_err;
    use crate::maj_list;
    let mut itr = env.clone();
    while !maj_nilp(itr.clone()).to_bool() {
        if let Maj::Cons { car: entry, cdr } = &*itr.clone() {
            if let Maj::Cons {
                car: symbol,
                cdr: _
            } = &*entry.clone() {
                if maj_eq(symbol.clone(), sym.clone()).to_bool() {
                    return entry.clone();
                }
            } else {
                panic!("All entries on an environment must be pairs");
            }
            itr = cdr.clone();
        } else {
            panic!("Environment is not an alist");
        }
    }
    maj_err(
        Maj::string("{} is unbound"),
        maj_list!(sym))
}

pub fn maj_env_lookup(env: Gc<Maj>, sym: Gc<Maj>) -> Gc<Maj> {
    use crate::axioms::predicates::maj_errorp;
    use crate::axioms::primitives::maj_cdr;
    let result = maj_env_assoc(env, sym);
    if !maj_errorp(result.clone()).to_bool() {
        maj_cdr(result)
    } else {
        result
    }
}

pub fn maj_env_union(env1: Gc<Maj>, env2: Gc<Maj>) -> Gc<Maj> {
    use crate::axioms::{
        primitives::{ maj_car, maj_cdr },
        predicates::maj_errorp
    };
    let is_env_env1 = maj_proper_list_p(env1.clone()).to_bool();
    let is_env_env2 = maj_proper_list_p(env2.clone()).to_bool();
    if !is_env_env1 || !is_env_env2 {
        panic!("Attempted union of improper lists");
    }
    
    // Uniting two envs involves creating a new env with mixed bindings.
    // It must basically be env2 with env1 bindings substituting wherever
    // a substitution is needed.
    let mut iter = env2.clone();
    // 0. Reverse env2 (because of the way it works)
    let mut env2_bindings = vec![];
    while !maj_nilp(iter.clone()).to_bool() {
        env2_bindings.push(maj_car(iter.clone()));
        iter = maj_cdr(iter);
    }
    
    // 1. traverse for each bind2 on env2.
    let mut newenv = Maj::nil();
    for bind2 in env2_bindings.iter().rev() {
        // 2. if (sym in bind2) is defined in (bind1 in env1), collect bind1.
        let sym = maj_car(bind2.clone());
        let bind1 = maj_env_assoc(env1.clone(), sym);
        newenv = if !maj_errorp(bind1.clone()).to_bool() {
            Maj::cons(bind1, newenv)
        } else {
            //    2.5. otherwise collect bind2.
            Maj::cons(bind2.clone(), newenv)
        };
    }

    // 3. Add bindings on env1 that were not added
    iter = env1.clone();
    while !maj_nilp(iter.clone()).to_bool() {
        let binding = maj_car(iter.clone());
        let sym = maj_car(binding.clone());
        if maj_errorp(maj_env_assoc(newenv.clone(), sym.clone())).to_bool() {
            newenv = Maj::cons(binding, newenv);
        }
        iter = maj_cdr(iter);
    }

    newenv
}
