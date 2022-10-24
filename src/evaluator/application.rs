use gc::Gc;
use crate::{
    maj_list,
    core::{ Maj, MajState },
    axioms::{
        predicates::*,
        primitives::*
    },
};

use crate::axioms::utils::{
    STACK_RED_ZONE,
    STACK_PER_RECURSION
};

use super::maj_eval;

pub fn maj_apply(
    mut state: &mut MajState,
    fun: Gc<Maj>,
    args: Gc<Maj>,
    env: Gc<Maj>
) -> Gc<Maj> {
    // primitives
    if maj_primitivep(fun.clone()).to_bool() {
        let name = maj_car(maj_cdr(maj_cdr(fun)));
        apply_primitive(&mut state, name, args, env)
    }

    // closure
    else if maj_closurep(fun.clone()).to_bool() {
        maj_apply_closure(&mut state, fun, args, env)
    }

    // macro
    else if maj_macrop(fun.clone()).to_bool() {
        let (expr, can_evaluate) =
            expand_macro(&mut state, fun, args,
                         env.clone());
        if can_evaluate {
            stacker::maybe_grow(
                STACK_RED_ZONE,
                STACK_PER_RECURSION,
                || maj_eval(&mut state, expr, env))
        } else {
            // Error
            expr
        }
    }

    // otherwise, fail
    else {
        maj_err(
            Maj::string("Cannot apply {} to args {}"),
            maj_list!(fun, args))
    }
}

fn maj_apply_closure(
    mut state: &mut MajState,
    fun: Gc<Maj>,
    args: Gc<Maj>,
    lexenv: Gc<Maj>
) -> Gc<Maj> {
    use crate::core::environment::maj_env_union;
    let length = maj_length(fun.clone()).to_integer().unwrap();
    if length != 5 {
        return maj_err(
            Maj::string("Invalid syntax: {}"),
            maj_list!(fun));
    }
    // (lit closure <env> lambda-list . body)
    let env = maj_car(maj_cdr(maj_cdr(fun.clone())));
    let lambda_list =
        maj_car(maj_cdr(maj_cdr(maj_cdr(fun.clone()))));
    let body =
        maj_car(maj_cdr(maj_cdr(maj_cdr(maj_cdr(fun.clone())))));

    if maj_nilp(args.clone()).to_bool() &&
        !maj_nilp(lambda_list.clone()).to_bool() {
            return maj_err(
                Maj::string(
                    "Cannot curry function without arguments"),
                Maj::nil());
        }

    if maj_consp(lambda_list.clone()).to_bool() &&
    maj_proper_list_p(lambda_list.clone()).to_bool() {
        let ll_len   = maj_length(lambda_list.clone())
            .to_integer();
        match ll_len {
            Some(ll_len) => {
                let args_len = maj_length(args.clone())
                    .to_integer()
                    .unwrap();
                if args_len > ll_len {
                    return maj_err(Maj::string(
                        "Too many arguments in function call"),
                                   Maj::nil());
                }
            },
            // Dotted list
            None => {},
        }
    }

    let (uargs, extenv) = maj_bind(lambda_list, args, env.clone());

    if maj_errorp(extenv.clone()).to_bool() {
        env
    } else if !maj_nilp(uargs.clone()).to_bool() {
        maj_list!(Maj::lit(),
                  Maj::closure(),
                  extenv,
                  uargs,
                  body)
    } else {
        // Implicit `do`
        let body = Maj::cons(Maj::do_sym(), body);
        // Unite extended environment with lexical environment
        // (on evaluation only)
        let extenv = maj_env_union(extenv, lexenv);
        maj_eval(&mut state, body, extenv)
    }
}

fn maj_bind(
    lambda_list: Gc<Maj>,
    args: Gc<Maj>,
    env: Gc<Maj>
) -> (Gc<Maj>, Gc<Maj>) {
    use crate::core::environment::maj_env_push;

    // -1. lambda_list is nil.
    if maj_nilp(lambda_list.clone()).to_bool() {
        // -1.1. If (not (nilp args)), then error
        if !maj_nilp(args.clone()).to_bool() {
            return
                (Maj::nil(),
                 maj_err(
                     Maj::string("Arguments {} exceeded lambda-list"),
                     maj_list!(args.clone())));
        }
        // -1.X. Otherwise, return (lambda_list, env).
        //      Everything is already bound.
        else {
            return (lambda_list, env);
        }
    }
    // 0. args is nil.
    // 0.1. Return (lambda_list, env) as well.
    //      This does currying.
    else if maj_nilp(args.clone()).to_bool() {
        return (lambda_list, env);
    }

    match &*lambda_list.clone() {
        // 1. lambda_list is a cons.
        Maj::Cons { car, cdr } => {
            let extenv =
                if maj_consp(car.clone()).to_bool() {
                    // If car is a cons: Do destructuring.
                    let newenv =
                        maj_destructuring_bind(car.clone(),
                                               maj_car(args.clone()),
                                               env.clone());
                    if maj_errorp(newenv.clone()).to_bool() {
                        return (Maj::nil(), newenv);
                    }
                    newenv
                } else {
                    // If car is not a cons: Normal binding.
                    // Bind car to (car args) in the lexenv.
                    maj_env_push(env.clone(),
                                 car.clone(),
                                 maj_car(args.clone()))
                };

            // Recur with extended environment, cdr as
            // lambda_list, (cdr args) as args
            maj_bind(cdr.clone(), maj_cdr(args), extenv)
        },
        // 2. lambda_list is a symbol.
        Maj::Sym(_) => {
            // TODO: if (last args) is (&), we need to curry
            // while binding (butlast args) to the symbol.
            // Then return the symbol itself as lambda-list.
            // The binding incurs in
            // (append (lookup ,symbol) (butlast args)).

            // If (last args) is not (&),
            // Both leave no unbound syms in lambda-list.
            (Maj::nil(),
             maj_env_push(
                 env.clone(),
                 lambda_list,
                 // 2.1. If (not (nilp args)) then bind args
                 if !maj_nilp(args.clone()).to_bool() {
                     args
                 }
                 // 2.2. If (nilp args) then bind nil
                 else {
                     Maj::nil()
                 }))
        },
        // X. Otherwise, error. Only cons and symbols allowed.
        _ => (Maj::nil(),
              maj_err(
                   Maj::string(
                      "Lambda list can only have symbols or conses"),
                  Maj::nil()))
    }
}

fn maj_destructuring_bind(list: Gc<Maj>,
                          args: Gc<Maj>,
                          env: Gc<Maj>) -> Gc<Maj> {
    use crate::core::environment::maj_env_push;

    let list_nilp = maj_nilp(list.clone()).to_bool();
    let args_nilp = maj_nilp(args.clone()).to_bool();

    // Do not allow leftover args
    if list_nilp && !args_nilp {
        return maj_err(
            Maj::string("Arguments exceed destructuring pattern"),
            Maj::nil());
    }

    // Leftover symbols, though, should bind to nil.
    if !list_nilp && args_nilp {
            return maj_bind_rec_nil(list, env);
    }

    // When both are nil, we are done
    if list_nilp && args_nilp {
        return env;
    }

    // When list is a symbol and args is not, bind list
    // to args and return
    if !maj_consp(list.clone()).to_bool() {
        return maj_env_push(env, list, args);
    }

    // When args is a non-nil symbol, fail immediately!
    if !maj_consp(args.clone()).to_bool() {
        return maj_err(
            Maj::string("Cannot destructure atomic value {}"),
            maj_list!(maj_car(args.clone())));
    }

    // Otherwise, there is destructuring to do.
    let sym = maj_car(list.clone());
    let arg = maj_car(args.clone());

    let extenv =
        if maj_consp(sym.clone()).to_bool() {
            // If sym is a cons, recursively destructure
            // it with arg as new environment.
            maj_destructuring_bind(sym, arg, env)
        } else {
            // Otherwise, do an ad-hoc binding to
            // generate the extended environment.
            maj_env_push(env, sym, arg)
        };

    // Proceed recursively with new environment
    maj_destructuring_bind(maj_cdr(list),
                           maj_cdr(args),
                           extenv)
}

fn maj_bind_rec_nil(lambda_list: Gc<Maj>, env: Gc<Maj>) -> Gc<Maj> {
    use crate::core::environment::maj_env_push;
    if maj_nilp(lambda_list.clone()).to_bool() {
        return Maj::nil();
    }

    match &*lambda_list.clone() {
        Maj::Sym(_) => {
            maj_env_push(env.clone(),
                         lambda_list.clone(),
                         Maj::nil())
        },
        Maj::Cons { car, cdr } => {
            let extenv = maj_bind_rec_nil(car.clone(), env);
            maj_bind_rec_nil(cdr.clone(), extenv)
        },
        _ => panic!("Never try to recursively bind a list with more than symbols and conses"),
    }
}

pub fn expand_macro(
    mut state: &mut MajState,
    mac: Gc<Maj>,
    args: Gc<Maj>,
    env: Gc<Maj>
) -> (Gc<Maj>, bool) {
    if maj_macrop(mac.clone()).to_bool() {
        // (lit macro <closure>)
        let closure = maj_car(maj_cdr(maj_cdr(mac)));
        let result = maj_apply(&mut state,
                               closure,
                               args.clone(),
                               env);
        if maj_closurep(result.clone()).to_bool() {
            (maj_err(
                Maj::string(
                    "Error expanding macro expression for {}"),
                maj_list!(args)),
             false)
        } else if maj_errorp(result.clone()).to_bool() {
            (result, false)
        } else {
            (result, true)
        }
    } else {
        (Maj::cons(mac, args), false)
    }
}

pub fn apply_primitive(mut state: &mut MajState,
                       prim: Gc<Maj>,
                       args: Gc<Maj>,
                       env: Gc<Maj>) -> Gc<Maj> {
    use crate::printing::maj_format;
    use crate::axioms::MajPrimArgs;
    let primitive = state.find_primitive(prim.clone());
    match primitive {
        Some((function, arity)) => {
            let argl = maj_length(args.clone())
                .to_integer()
                .unwrap();
            match *arity {
                MajPrimArgs::None => {
                    if argl != 0 {
                        maj_err(
                            Maj::string("{} requires no arguments"),
                            maj_list!(prim))
                    } else {
                        function(&mut state, Maj::nil(), env)
                    }
                },
                MajPrimArgs::Required(n) => {
                    let n = n as i64;
                    if argl < n {
                        // Curry on argl < n && argl != 0.
                        // Delegate to closures
                        curry_primitive(&mut state, prim, n, argl, args, env, false)
                    } else if argl > n {
                        // Fail on argl > n
                        maj_err(
                            Maj::string("Too many arguments for {}"),
                            maj_list!(prim))
                    } else {
                        // Apply on argl = n
                        function(&mut state, args, env)
                    }
                },
                MajPrimArgs::Variadic(n) => {
                    let n = n as i64;
                    // Check for & to account for in argl.
                    let last = maj_last(args.clone());
                    let force_curry = maj_eq(last, Maj::ampersand())
                        .to_bool();
                    let targl = if force_curry { argl - 1 } else { argl };

                    // Normal and variadic currying, respectively
                    if (targl < n && !force_curry) ||
                        (targl > n && force_curry) {
                            // Delegate currying to closures
                            curry_primitive(&mut state, prim, n, targl, args, env, true)
                        } else {
                            function(&mut state, args, env)
                        }
                },
            }
        },
        _ => panic!("Primitive \"{}\" does not exist",
                    maj_format(&state, prim)),
    }
}

fn gen_arglist(mut state: &mut MajState, n: i64, variadicp: bool) -> Gc<Maj> {
    let mut list = if variadicp {
        Maj::symbol(&mut state, "rest")
    } else {
        Maj::nil()
    };

    for _ in 0..n {
        list = Maj::cons(maj_gensym(&mut state),
                         list.clone());
    }
    list
}

fn wrap_primitive(prim: Gc<Maj>,
                  env: Gc<Maj>,
                  arglist: Gc<Maj>) -> Gc<Maj> {
    // (lit closure <env> <arglist> ((<prim> . <arglist>)))
    maj_list!(
        Maj::lit(),
        Maj::closure(),
        env,
        arglist.clone(),
        maj_list!(
            maj_cons(prim, arglist)))
}

fn curry_primitive(mut state: &mut MajState,
                   prim: Gc<Maj>,
                   num_args: i64,
                   num_params: i64,
                   args: Gc<Maj>,
                   env: Gc<Maj>,
                   variadicp: bool) -> Gc<Maj> {
    if num_params == 0 {
        return maj_err(
            Maj::string(
                "Cannot curry function without arguments"),
            Maj::nil());
    }

    // Currying a primitive function involves generating
    // a wrapper closure, then performing simple application.
    let arglist = gen_arglist(&mut state, num_args, variadicp);
    let wrapped = wrap_primitive(prim, env.clone(), arglist);
    maj_apply(&mut state, wrapped, args, env)
}
