use gc::Gc;
use crate::maj_list;
use crate::core::{ Maj, MajState };
use crate::axioms::predicates::{
    maj_eq,
    maj_nilp,
    maj_symbolp,
    maj_errorp,
    maj_literalp,
    maj_numberp,
    maj_charp,
    maj_streamp,
    maj_stringp,
    maj_macrop
};
use crate::axioms::primitives::{
    maj_car,
    maj_cdr,
    maj_err,
    maj_length
};

use crate::axioms::utils::{
    STACK_RED_ZONE,
    STACK_PER_RECURSION
};

use super::maj_apply;

pub fn maj_eval(mut state: &mut MajState,
                expr: Gc<Maj>,
                env: Gc<Maj>) -> Gc<Maj> {
    // use crate::maj_format;
    // println!("EVAL: {}", maj_format(&state, expr.clone()));

    /* Errors */
    if maj_errorp(expr.clone()).to_bool() {
        expr
    }

    /* Special forms */
    // self-evaluating forms:
    // Literals, numbers, characters, streams
    else if maj_is_selfeval(expr.clone()) {
        expr
    }
    
    // variables
    else if maj_symbolp(expr.clone()).to_bool() {
        state.lookup(env, expr)
    }

    // quote
    else if maj_quotep(expr.clone()).to_bool() {
        maj_handle_quote(expr)
    }

    // quasiquote
    else if maj_quasiquotep(expr.clone()).to_bool() {
        maj_handle_quasiquote(&mut state, expr, env)
    }

    // macros
    else if maj_macp(expr.clone()).to_bool() {
        maj_handle_mac(expr, env)
    }

    // definitions
    else if maj_defp(&mut state, expr.clone()).to_bool() {
        maj_handle_definition(&mut state, expr, env)
    }

    // redefinitions
    else if maj_setp(&mut state, expr.clone()).to_bool() {
        maj_handle_redefinition(&mut state, expr, env)
    }

    // redefinitions for a cons cell
    else if maj_set_car_p(&mut state, expr.clone()).to_bool() {
        maj_handle_redefine_car(&mut state, expr, env)
    }
    else if maj_set_cdr_p(&mut state, expr.clone()).to_bool() {
        maj_handle_redefine_cdr(&mut state, expr, env)
    }

    // conditionals
    else if maj_ifp(&mut state, expr.clone()).to_bool() {
        maj_handle_if(&mut state, expr, env)
    }

    // closures
    else if maj_fnp(expr.clone()).to_bool() {
        maj_handle_fn(expr, env)
    }

    // do
    else if maj_dop(&mut state, expr.clone()).to_bool() {
        maj_handle_do(&mut state, expr, env)
    }

    // and
    else if maj_andp(&mut state, expr.clone()).to_bool() {
        maj_handle_and(&mut state, expr, env)
    }

    // or
    else if maj_orp(&mut state, expr.clone()).to_bool() {
        maj_handle_or(&mut state, expr, env)
    }

    // apply
    else if maj_applyp(expr.clone()).to_bool() {
        maj_handle_apply(&mut state, expr, env)
    }

    // while
    else if maj_whilep(&mut state, expr.clone()).to_bool() {
        maj_handle_while(&mut state, expr, env)
    }

    // letrec
    else if maj_letrecp(&mut state, expr.clone()).to_bool() {
        maj_handle_letrec(&mut state, expr, env)
    }

    // unwind-protect
    else if maj_unwind_protect_p(&mut state, expr.clone()).to_bool() {
        maj_handle_unwind_protect(&mut state, expr, env)
    }

    // application
    else {
        let fun = maj_eval(&mut state,
                           maj_car(expr.clone()),
                           env.clone());
        if maj_errorp(fun.clone()).to_bool() {
            return fun;
        }

        let args =
            if maj_macrop(fun.clone()).to_bool() {
                maj_cdr(expr)
            } else {
                maj_evlist(&mut state,
                           maj_cdr(expr),
                           env.clone())
            };
        
        if maj_errorp(args.clone()).to_bool() {
            return args;
        }

        stacker::maybe_grow(
            STACK_RED_ZONE,
            STACK_PER_RECURSION,
            || maj_apply(&mut state, fun, args, env))
    }
}

fn maj_evlist(mut state: &mut MajState,
              list: Gc<Maj>,
              env: Gc<Maj>) -> Gc<Maj> {
    let mut elts = Vec::new();
    let mut itr = list.clone();
    // Evaluate member by member
    while !maj_nilp(itr.clone()).to_bool() {
        let elt = maj_car(itr.clone());
        let elt = maj_eval(&mut state, elt, env.clone());
        if maj_errorp(elt.clone()).to_bool() {
            return elt;
        }
        elts.push(elt);
        itr = maj_cdr(itr);
    }
    elts.reverse();
    // Generate list of evaluated members backwards
    let mut new_list = Maj::nil();
    
    for elt in elts.iter() {
        new_list = Maj::cons(elt.clone(),
                             new_list.clone());
    }
    new_list
}

fn maj_handle_quote(expr: Gc<Maj>) -> Gc<Maj> {
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length != 2 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr));
    }

    maj_car(maj_cdr(expr))
}

fn maj_handle_quasiquote(mut state: &mut MajState, expr: Gc<Maj>, env: Gc<Maj>) -> Gc<Maj> {
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length != 2 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr));
    }
    maj_do_quasiquote(&mut state, maj_car(maj_cdr(expr)), env)
}

fn maj_do_quasiquote(
    mut state: &mut MajState,
    expr: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    use crate::axioms::predicates::maj_atomp;
    use crate::axioms::primitives::maj_append;
    
    if maj_atomp(expr.clone()).to_bool() {
        return expr;
    }

    let car  = maj_car(expr.clone());
    let cdr  = maj_cdr(expr.clone());
    let cadr = maj_car(cdr.clone());

    if maj_unquotep(expr.clone()).to_bool() ||
        maj_unquote_splice_p(expr.clone()).to_bool() {
            maj_eval(&mut state, cadr.clone(), env.clone())
        } else {
            let should_append = maj_unquote_splice_p(
                car.clone()
            ).to_bool();
            let car_qquote = maj_do_quasiquote(
                &mut state, car.clone(), env.clone());

            if maj_errorp(car_qquote.clone()).to_bool() {
                return car_qquote;
            }

            let rest = maj_do_quasiquote(
                &mut state,
                cdr.clone(),
                env.clone());
            if maj_errorp(rest.clone()).to_bool() {
                return rest;
            }

            if should_append {
                maj_append(maj_list!(car_qquote, rest))
            } else {
                Maj::cons(car_qquote, rest)
            }
        }
}

fn maj_handle_definition(
    mut state: &mut MajState,
    expr: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length != 3 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr));
    }

    let sym = maj_car(maj_cdr(expr.clone()));
    let val = maj_car(maj_cdr(maj_cdr(expr)));

    // Evaluate associated value before binding
    let val = maj_eval(&mut state, val, env);
    if maj_errorp(val.clone()).to_bool() {
        return val;
    }

    if maj_symbolp(sym.clone()).to_bool() {
        // Try finding thing on environment.
        // An error means it does not exist
        let element = state.assoc(Maj::nil(), sym.clone());
        if !maj_errorp(element.clone()).to_bool() {
            // If exists, attribute destructively
            let new_entry = Maj::cons(sym.clone(), val);
            unsafe {
                let raw_element = Gc::into_raw(element.clone());
                std::ptr::copy_nonoverlapping(
                    Gc::into_raw(new_entry.clone()),
                    raw_element as *mut Maj, 1);
            }
            sym
        } else {
            // Else, push new element to state
            state.push(sym, val)
        }
    } else {
        maj_err(
            Maj::string("{} is not a symbol"),
            maj_list!(sym))
    }
}

fn maj_handle_redefinition(
    mut state: &mut MajState,
    expr: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length != 3 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr));
    }

    let sym = maj_car(maj_cdr(expr.clone()));
    let val = maj_car(maj_cdr(maj_cdr(expr)));

    // Evaluate associated value before binding
    let val = maj_eval(&mut state, val, env.clone());
    if maj_errorp(val.clone()).to_bool() {
        return val;
    }

    if maj_symbolp(sym.clone()).to_bool() {
        // Try finding thing on both environments.
        let element = state.assoc(env, sym.clone());
        if maj_errorp(element.clone()).to_bool() {
            element
        } else {
            // When found, replace binding cons
            let new_binding = Maj::cons(sym.clone(), val);
            unsafe {
                let rawelement = Gc::into_raw(element.clone());
                std::ptr::copy_nonoverlapping(
                    Gc::into_raw(new_binding.clone()),
                    rawelement as *mut Maj, 1);
            }
            sym
        }
    } else {
        maj_err(
            Maj::string("{} is not a symbol"),
            maj_list!(sym))
    }
}

fn maj_handle_redefine_cxr_helper(
    mut state: &mut MajState,
    expr: Gc<Maj>,
    env: Gc<Maj>
) -> Result<(Gc<Maj>, Gc<Maj>), Gc<Maj>> {
    use crate::axioms::predicates::maj_consp;
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length != 3 {
        return Err(maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr)));
    }

    let value = maj_car(maj_cdr(maj_cdr(expr.clone())));
    let pair  = maj_car(maj_cdr(expr));

    let value = maj_eval(&mut state, value, env.clone());
    if maj_errorp(value.clone()).to_bool() {
        return Err(value);
    }
    
    let pair = maj_eval(&mut state, pair, env.clone());
    if maj_errorp(pair.clone()).to_bool() {
        return Err(pair);
    } else if !maj_consp(pair.clone()).to_bool() {
        return Err(maj_err(
            Maj::string("{} is not a cons cell"),
            maj_list!(pair)));
    }

    Ok((pair, value))
}

fn maj_handle_redefine_car(
    mut state: &mut MajState,
    expr: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    match maj_handle_redefine_cxr_helper(&mut state, expr, env) {
        Ok((pair, value)) => {
            let new_pair = Maj::cons(value, maj_cdr(pair.clone()));
            unsafe {
                let rawpair = Gc::into_raw(pair.clone());
                std::ptr::copy_nonoverlapping(
                    Gc::into_raw(new_pair.clone()),
                    rawpair as *mut Maj, 1);
            }
            pair
        },
        Err(error) => error,
    }
}

fn maj_handle_redefine_cdr(
    mut state: &mut MajState,
    expr: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    match maj_handle_redefine_cxr_helper(&mut state, expr, env) {
        Ok((pair, value)) => {
            let new_pair = Maj::cons(maj_car(pair.clone()), value);
            unsafe {
                let rawpair = Gc::into_raw(pair.clone());
                std::ptr::copy_nonoverlapping(
                    Gc::into_raw(new_pair.clone()),
                    rawpair as *mut Maj, 1);
            }
            pair
        },
        Err(error) => error,
    }
}

fn maj_handle_if(mut state: &mut MajState, expr: Gc<Maj>, env: Gc<Maj>) -> Gc<Maj> {
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length != 4 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr));
    }

    let pred   = maj_car(maj_cdr(expr.clone()));
    let conseq = maj_car(maj_cdr(maj_cdr(expr.clone())));
    let altern = maj_car(maj_cdr(maj_cdr(maj_cdr(expr))));
    
    let pred_result = maj_eval(&mut state, pred, env.clone());
    if maj_errorp(pred_result.clone()).to_bool() {
        pred_result
    } else {
        maj_eval(&mut state,
                 if maj_nilp(pred_result).to_bool() {
                     altern
                 } else {
                     conseq
                 },
                 env)
    }
}

fn maj_handle_fn(expr: Gc<Maj>, env: Gc<Maj>) -> Gc<Maj> {
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length < 3 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr));
    }

    // (fn lambda-list . body)
    let lambda_list = maj_car(maj_cdr(expr.clone()));
    let body = maj_cdr(maj_cdr(expr));

    if maj_errorp(lambda_list.clone()).to_bool() {
        lambda_list
    } else if maj_errorp(body.clone()).to_bool() {
        body
    } else {
        // (lit closure <env> <lambda-list> ((<body>)))
        maj_list!(Maj::lit(),
                  Maj::closure(),
                  env, lambda_list, body)
    }
}

fn maj_handle_mac(expr: Gc<Maj>, env: Gc<Maj>) -> Gc<Maj> {
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length < 3 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr));
    }

    // (mac lambda-list . body)
    let lambda_list = maj_car(maj_cdr(expr.clone()));
    let body = maj_cdr(maj_cdr(expr));

    // (lit macro (lit closure <env> lambda-list (body)))
    maj_list!(Maj::lit(),
              Maj::macro_sym(),
              maj_list!(
                  Maj::lit(),
                  Maj::closure(),
                  env, lambda_list, body))
}

fn maj_handle_do(mut state: &mut MajState, expr: Gc<Maj>, env: Gc<Maj>) -> Gc<Maj> {
    let mut itr    = maj_cdr(expr.clone());
    let mut result = Maj::nil();
    while !maj_nilp(itr.clone()).to_bool() {
        let expr = maj_car(itr.clone());
        result = maj_eval(&mut state, expr, env.clone());
        if maj_errorp(result.clone()).to_bool() {
            return result;
        }
        itr = maj_cdr(itr.clone());
    }
    result
}

fn maj_handle_apply(mut state: &mut MajState, expr: Gc<Maj>, env: Gc<Maj>) -> Gc<Maj> {
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length != 3 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr));
    }

    // (apply fun args) -> (eval (cons fun args))
    let func = maj_car(maj_cdr(expr.clone()));
    let args = maj_car(maj_cdr(maj_cdr(expr)));

    let func = maj_eval(&mut state, func, env.clone());
    let args = maj_eval(&mut state, args, env.clone());

    if maj_errorp(func.clone()).to_bool() {
        func
    } else if maj_macrop(func.clone()).to_bool()
        || maj_macp(func.clone()).to_bool()
    {
        maj_err(
            Maj::string("Macros cannot be applied"),
            Maj::nil())
    } else if maj_errorp(args.clone()).to_bool() {
        args
    } else {
        maj_eval(&mut state, Maj::cons(func, args), env)
    }
}

fn maj_handle_letrec(
    mut state: &mut MajState,
    expr: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    use crate::axioms::predicates::maj_consp;
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length < 3 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr));
    }

    // (letrec bindings . body)
    let bindings = maj_car(maj_cdr(expr.clone()));
    let body     = maj_cdr(maj_cdr(expr.clone()));

    // Implicit `do`
    let body = Maj::cons(Maj::do_sym(), body);

    let mut fnnames  = vec![];
    let mut fns      = vec![];
    let mut iter = bindings.clone();
    while !maj_nilp(iter.clone()).to_bool() {
        // (sym lambda-list . body)
        let clause = maj_car(iter.clone());
        if !maj_consp(clause.clone()).to_bool() {
            return maj_err(Maj::string(
                "Syntax error on letrec: {} is not a proper clause"),
                maj_list!(clause));
        }

        let sym = maj_car(clause.clone());
        if !maj_symbolp(sym.clone()).to_bool() {
            return maj_err(Maj::string(
                "Syntax error on letrec: {} is not a valid function name"),
                maj_list!(sym));
        }

        let function = Maj::cons(
            Maj::fn_sym(),
            maj_cdr(clause));

        fnnames.push(sym);
        fns.push(function);

        iter = maj_cdr(iter);
    }

    let mut closures = vec![];
    // iterate over fns, interpreting, putting results in closures
    for function in fns.iter() {
        let closure = maj_eval(
            &mut state,
            function.clone(),
            env.clone());
        if maj_errorp(closure.clone()).to_bool() {
            return closure;
        }
        closures.push(closure);
    }

    // iterate over fnnames and closures, creating an env.
    use crate::core::environment::maj_env_push;
    let mut new_env = Maj::nil();
    for i in 0..closures.len() {
        new_env = maj_env_push(new_env,
                               fnnames[i].clone(),
                               closures[i].clone());
    }

    // append new_env to env.
    use crate::core::environment::maj_env_union;
    new_env = maj_env_union(new_env, env.clone());

    // for each closure, inject this new environment
    for closure in closures.iter() {
        // (lit closure <env> <ll> . <body>)
        let clorest = maj_cdr(maj_cdr(closure.clone()));
        let clobody = maj_cdr(clorest.clone());
        
        // modify car of clobody
        unsafe {
            let rawclorest = Gc::into_raw(clorest.clone());
            let newcons = Maj::cons(new_env.clone(), clobody);
            std::ptr::copy_nonoverlapping(
                Gc::into_raw(newcons.clone()),
                rawclorest as *mut Maj, 1);
        }
    }
    maj_eval(&mut state, body, new_env)
}

fn maj_handle_while(
    mut state: &mut MajState,
    expr: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    // (while pred . body)
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length < 2 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr));
    }

    let pred = maj_car(maj_cdr(expr.clone()));
    let body = maj_cdr(maj_cdr(expr));
    let body = Maj::cons(Maj::do_sym(), body);

    let mut body_result = Maj::nil();
    loop {
        let pred_result = maj_eval(&mut state,
                                   pred.clone(),
                                   env.clone());
        if maj_errorp(pred_result.clone()).to_bool() {
            return pred_result;
        }
        if !pred_result.to_bool() {
            break;
        }
        body_result = maj_eval(&mut state,
                               body.clone(),
                               env.clone());
        if maj_errorp(body_result.clone()).to_bool() {
            return body_result;
        }
    }
    body_result
}

fn maj_handle_unwind_protect(
    mut state: &mut MajState,
    expr: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    // (unwind-protect expr cleanup)
    let length = maj_length(expr.clone())
        .to_integer().unwrap();
    if length != 3 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(expr));
    }

    let exp     = maj_car(maj_cdr(expr.clone()));
    let cleanup = maj_car(maj_cdr(maj_cdr(expr)));

    let result = maj_eval(&mut state, exp, env.clone());
    let _      = maj_eval(&mut state, cleanup, env);
    result
}

fn maj_handle_and(mut state: &mut MajState, expr: Gc<Maj>, env: Gc<Maj>) -> Gc<Maj> {
    let forms = maj_cdr(expr);
    let mut iter = forms;
    let mut result = Maj::t();
    while !maj_nilp(iter.clone()).to_bool() {
        let form = maj_car(iter.clone());
        result = maj_eval(&mut state, form, env.clone());
        if maj_nilp(result.clone()).to_bool() {
            return Maj::nil();
        }
        iter = maj_cdr(iter.clone());
    }
    result
}

fn maj_handle_or(mut state: &mut MajState, expr: Gc<Maj>, env: Gc<Maj>) -> Gc<Maj> {
    let forms = maj_cdr(expr);
    let mut iter = forms;
    let mut result = Maj::nil();
    while !maj_nilp(iter.clone()).to_bool() {
        let form = maj_car(iter.clone());
        result = maj_eval(&mut state, form, env.clone());
        if !maj_nilp(result.clone()).to_bool() {
            return result;
        }
        iter = maj_cdr(iter.clone());
    }
    result
}

fn maj_is_selfeval(x: Gc<Maj>) -> bool {
    maj_literalp(x.clone()).to_bool()
        || maj_nilp(x.clone()).to_bool()
        || maj_eq(x.clone(), Maj::t()).to_bool()
        || maj_eq(x.clone(), Maj::ampersand()).to_bool()
        || maj_eq(x.clone(), Maj::apply()).to_bool()
        || maj_numberp(x.clone()).to_bool()
        || maj_charp(x.clone()).to_bool()
        || maj_streamp(x.clone()).to_bool()
        || maj_stringp(x.clone()).to_bool()
}

pub fn maj_quotep(x: Gc<Maj>) -> Gc<Maj> {
    maj_eq(maj_car(x), Maj::quote())
}

pub fn maj_quasiquotep(x: Gc<Maj>) -> Gc<Maj> {
    maj_eq(maj_car(x), Maj::quasiquote())
}

pub fn maj_unquotep(x: Gc<Maj>) -> Gc<Maj> {
    maj_eq(maj_car(x), Maj::unquote())
}

pub fn maj_unquote_splice_p(x: Gc<Maj>) -> Gc<Maj> {
    maj_eq(maj_car(x), Maj::unquote_splice())
}

fn maj_defp(mut state: &mut MajState,
            x: Gc<Maj>) -> Gc<Maj> {
    let car = maj_car(x);
    maj_eq(car, Maj::symbol(&mut state, "def"))
}

fn maj_setp(mut state: &mut MajState,
            x: Gc<Maj>) -> Gc<Maj> {
    let car = maj_car(x);
    maj_eq(car, Maj::symbol(&mut state, "set"))
}

fn maj_set_car_p(mut state: &mut MajState,
            x: Gc<Maj>) -> Gc<Maj> {
    let car = maj_car(x);
    maj_eq(car, Maj::symbol(&mut state, "set-car"))
}

fn maj_set_cdr_p(mut state: &mut MajState,
            x: Gc<Maj>) -> Gc<Maj> {
    let car = maj_car(x);
    maj_eq(car, Maj::symbol(&mut state, "set-cdr"))
}

fn maj_dop(mut state: &mut MajState,
           x: Gc<Maj>) -> Gc<Maj> {
    let car = maj_car(x);
    maj_eq(car, Maj::symbol(&mut state, "do"))
}

fn maj_andp(mut state: &mut MajState,
            x: Gc<Maj>) -> Gc<Maj> {
    let car = maj_car(x);
    maj_eq(car, Maj::symbol(&mut state, "and"))
}

fn maj_orp(mut state: &mut MajState,
            x: Gc<Maj>) -> Gc<Maj> {
    let car = maj_car(x);
    maj_eq(car, Maj::symbol(&mut state, "or"))
}

fn maj_ifp(mut state: &mut MajState,
           x: Gc<Maj>) -> Gc<Maj> {
    let car = maj_car(x);
    maj_eq(car, Maj::symbol(&mut state, "if"))
}

#[inline]
fn maj_fnp(x: Gc<Maj>) -> Gc<Maj> {
    maj_eq(maj_car(x), Maj::fn_sym())
}

#[inline]
fn maj_applyp(x: Gc<Maj>) -> Gc<Maj> {
    maj_eq(maj_car(x), Maj::apply())
}

fn maj_whilep(mut state: &mut MajState,
              x: Gc<Maj>) -> Gc<Maj> {
    let car = maj_car(x);
    maj_eq(car, Maj::symbol(&mut state, "while"))
}

#[inline]
fn maj_macp(x: Gc<Maj>) -> Gc<Maj> {
    maj_eq(maj_car(x), Maj::mac())
}

#[inline]
fn maj_letrecp(mut state: &mut MajState,
               x: Gc<Maj>) -> Gc<Maj> {
    let car = maj_car(x);
    maj_eq(car, Maj::symbol(&mut state, "letrec"))
}

#[inline]
fn maj_unwind_protect_p(mut state: &mut MajState,
                        x: Gc<Maj>) -> Gc<Maj> {
    let car = maj_car(x);
    maj_eq(car, Maj::symbol(&mut state, "unwind-protect"))
}
