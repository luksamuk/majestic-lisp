use gc::Gc;
use crate::core::{ Maj, MajState };
use crate::{ maj_list, maj_dotted_list };
use crate::evaluator::evaluation::maj_eval;
use crate::axioms::predicates::maj_errorp;

#[inline]
fn maj_define_ulps(mut state: &mut MajState) {
    let ulps_sym = Maj::symbol(&mut state, "*ulps*");
    state.push(ulps_sym, Maj::integer(3));
}

#[inline]
fn maj_define_standard_streams(mut state: &mut MajState) {
    let stdin_sym  = Maj::symbol(&mut state, "*stdin*");
    let stdout_sym = Maj::symbol(&mut state, "*stdout*");
    let stderr_sym = Maj::symbol(&mut state, "*stderr*");
    let stdin      = state.make_stream_stdin();
    let stdout     = state.make_stream_stdout();
    let stderr     = state.make_stream_stderr();
    state.push(stdin_sym, stdin);
    state.push(stdout_sym, stdout);
    state.push(stderr_sym, stderr);
}

fn maj_put_constants(mut state: &mut MajState) {
    maj_define_ulps(&mut state);
    maj_define_standard_streams(&mut state);
}

#[inline]
fn bootstrap_defmac(mut state: &mut MajState) -> Gc<Maj> {
    maj_list!(
        Maj::symbol(&mut state, "def"),
        Maj::symbol(&mut state, "defmac"),
        maj_list!(
            Maj::mac(),
            maj_dotted_list!(
                Maj::symbol(&mut state, "label"),
                Maj::symbol(&mut state, "lambda-list"),
                Maj::symbol(&mut state, "body")),
            maj_list!(
                Maj::quasiquote(),
                maj_list!(
                    Maj::symbol(&mut state, "def"),
                    maj_list!(
                        Maj::unquote(),
                        Maj::symbol(&mut state, "label")),
                    maj_list!(
                        Maj::mac(),
                        maj_list!(
                            Maj::unquote(),
                            Maj::symbol(&mut state,
                                        "lambda-list")),
                        maj_list!(
                            Maj::unquote_splice(),
                            Maj::symbol(&mut state,
                                        "body")))))))
}

#[inline]
fn bootstrap_defn(mut state: &mut MajState) -> Gc<Maj> {
    maj_list!(
        Maj::symbol(&mut state, "defmac"),
        Maj::symbol(&mut state, "defn"),
        maj_dotted_list!(
            Maj::symbol(&mut state, "label"),
            Maj::symbol(&mut state, "lambda-list"),
            Maj::symbol(&mut state, "body")),
        maj_list!(
            Maj::quasiquote(),
            maj_list!(
                Maj::symbol(&mut state, "def"),
                maj_list!(
                    Maj::unquote(),
                    Maj::symbol(&mut state, "label")),
                maj_list!(
                    Maj::fn_sym(),
                    maj_list!(
                        Maj::unquote(),
                        Maj::symbol(&mut state,
                                    "lambda-list")),
                    maj_list!(
                        Maj::unquote_splice(),
                        Maj::symbol(&mut state,
                                    "body"))))))
}

#[inline]
fn bootstrap_let(mut state: &mut MajState) -> Gc<Maj> {
    let sepfn_sym  = Maj::symbol(&mut state, "sepfn");
    let recur_sym  = Maj::symbol(&mut state, "recur");
    let syms_sym = Maj::symbol(&mut state, "syms");
    let vals_sym = Maj::symbol(&mut state, "vals");
    let body_sym = Maj::symbol(&mut state, "body");
    let args_sym = Maj::symbol(&mut state, "args");
    let pairs_sym = Maj::symbol(&mut state, "pairs");
    let cons_sym = Maj::symbol(&mut state, "cons");

    maj_list!(
        Maj::symbol(&mut state, "defmac"),
        Maj::symbol(&mut state, "let"),
        maj_dotted_list!(
            args_sym.clone(), body_sym.clone()),
        maj_list!(
            maj_list!(
                Maj::fn_sym(),
                maj_list!(sepfn_sym.clone()),
                maj_list!(
                    maj_list!(
                        Maj::fn_sym(),
                        maj_list!(
                            maj_list!(syms_sym.clone(),
                                      vals_sym.clone())),
                        maj_list!(
                            Maj::quasiquote(),
                            maj_list!(
                                maj_list!(
                                    Maj::fn_sym(),
                                    maj_list!(
                                        Maj::unquote(),
                                        syms_sym.clone()),
                                    maj_list!(
                                        Maj::unquote_splice(),
                                        body_sym)),
                                maj_list!(
                                    Maj::unquote_splice(),
                                    vals_sym.clone())))),
                    maj_list!(
                        sepfn_sym.clone(),
                        args_sym,
                        Maj::nil(),
                        Maj::nil(),
                        sepfn_sym))),
            maj_list!(
                Maj::fn_sym(),
                maj_list!(
                    pairs_sym.clone(),
                    syms_sym.clone(),
                    vals_sym.clone(),
                    recur_sym.clone()),
                maj_list!(
                    Maj::symbol(&mut state, "if"),
                    maj_list!(
                        Maj::symbol(&mut state, "nilp"),
                        pairs_sym.clone()),
                    maj_list!(
                        Maj::symbol(&mut state, "list"),
                        syms_sym.clone(),
                        vals_sym.clone()),
                    maj_list!(
                        recur_sym.clone(),
                        maj_list!(
                            Maj::symbol(&mut state, "cdr"),
                            pairs_sym.clone()),
                        maj_list!(
                            cons_sym.clone(),
                            maj_list!(
                                Maj::symbol(&mut state, "caar"),
                                pairs_sym.clone()),
                            syms_sym),
                        maj_list!(
                            cons_sym,
                            maj_list!(
                                Maj::symbol(&mut state, "car"),
                                maj_list!(
                                    Maj::symbol(&mut state, "cdar"),
                                    pairs_sym)),
                            vals_sym),
                        recur_sym)))))
}

#[inline]
fn bootstrap_letstar(mut state: &mut MajState) -> Gc<Maj> {
    let letstar_sym = Maj::symbol(&mut state, "let*");
    let if_sym      = Maj::symbol(&mut state, "if");
    let clauses_sym = Maj::symbol(&mut state, "clauses");
    let body_sym    = Maj::symbol(&mut state, "body");
    let nilp_sym    = Maj::symbol(&mut state, "nilp");
    let cdr_sym     = Maj::symbol(&mut state, "cdr");
    let cons_sym    = Maj::symbol(&mut state, "cons");

    maj_list!(
        Maj::symbol(&mut state, "defmac"),
        letstar_sym.clone(),
        maj_dotted_list!(
            clauses_sym.clone(),
            body_sym.clone()),
        maj_list!(
            if_sym.clone(),
            maj_list!(nilp_sym.clone(),
                      clauses_sym.clone()),
            maj_list!(
                cons_sym.clone(),
                maj_list!(
                    Maj::quote(),
                    Maj::do_sym()),
                body_sym.clone()),
            maj_list!(
                Maj::quasiquote(),
                maj_list!(
                    Maj::symbol(&mut state, "let"),
                    maj_list!(
                        maj_list!(
                            Maj::unquote(),
                            maj_list!(
                                Maj::symbol(&mut state, "car"),
                                clauses_sym.clone()))),
                    maj_list!(
                        Maj::unquote(),
                        maj_list!(
                            if_sym,
                            maj_list!(
                                nilp_sym,
                                maj_list!(
                                    cdr_sym.clone(),
                                    clauses_sym.clone())),
                            maj_list!(
                                cons_sym.clone(),
                                maj_list!(
                                    Maj::quote(),
                                    Maj::do_sym()),
                                body_sym.clone()),
                            maj_list!(
                                Maj::quasiquote(),
                                maj_list!(
                                    letstar_sym,
                                    maj_list!(
                                        Maj::unquote(),
                                        maj_list!(
                                            cdr_sym,
                                            clauses_sym)),
                                    maj_list!(
                                        Maj::unquote_splice(),
                                        body_sym)))))))))
}

#[inline]
fn bootstrap_letfn(mut state: &mut MajState) -> Gc<Maj> {
    let sepfn_sym  = Maj::symbol(&mut state, "sepfn");
    let recur_sym  = Maj::symbol(&mut state, "recur");
    let syms_sym = Maj::symbol(&mut state, "syms");
    let vals_sym = Maj::symbol(&mut state, "vals");
    let body_sym = Maj::symbol(&mut state, "body");
    let defs_sym = Maj::symbol(&mut state, "defs");
    let pairs_sym = Maj::symbol(&mut state, "pairs");
    let cons_sym = Maj::symbol(&mut state, "cons");

    maj_list!(
        Maj::symbol(&mut state, "defmac"),
        Maj::symbol(&mut state, "letfn"),
        maj_dotted_list!(
            defs_sym.clone(), body_sym.clone()),
        maj_list!(
            maj_list!(
                Maj::fn_sym(),
                maj_list!(sepfn_sym.clone()),
                maj_list!(
                    maj_list!(
                        Maj::fn_sym(),
                        maj_list!(
                            maj_list!(syms_sym.clone(),
                                      vals_sym.clone())),
                        maj_list!(
                            Maj::quasiquote(),
                            maj_list!(
                                maj_list!(
                                    Maj::fn_sym(),
                                    maj_list!(
                                        Maj::unquote(),
                                        syms_sym.clone()),
                                    maj_list!(
                                        Maj::unquote_splice(),
                                        body_sym)),
                                maj_list!(
                                    Maj::unquote_splice(),
                                    vals_sym.clone())))),
                    maj_list!(
                        sepfn_sym.clone(),
                        defs_sym,
                        Maj::nil(),
                        Maj::nil(),
                        sepfn_sym))),
            maj_list!(
                Maj::fn_sym(),
                maj_list!(
                    pairs_sym.clone(),
                    syms_sym.clone(),
                    vals_sym.clone(),
                    recur_sym.clone()),
                maj_list!(
                    Maj::symbol(&mut state, "if"),
                    maj_list!(
                        Maj::symbol(&mut state, "nilp"),
                        pairs_sym.clone()),
                    maj_list!(
                        Maj::symbol(&mut state, "list"),
                        syms_sym.clone(),
                        vals_sym.clone()),
                    maj_list!(
                        recur_sym.clone(),
                        maj_list!(
                            Maj::symbol(&mut state, "cdr"),
                            pairs_sym.clone()),
                        maj_list!(
                            cons_sym.clone(),
                            maj_list!(
                                Maj::symbol(&mut state, "caar"),
                                pairs_sym.clone()),
                            syms_sym),
                        maj_list!(
                            cons_sym.clone(),
                            maj_list!(
                                cons_sym,
                                maj_list!(
                                    Maj::quote(),
                                    Maj::fn_sym()),
                                maj_list!(
                                    Maj::symbol(&mut state, "cdar"),
                                    pairs_sym)),
                            vals_sym),
                        recur_sym)))))
}

#[inline]
fn bootstrap_letfnstar(mut state: &mut MajState) -> Gc<Maj> {
    let letfnstar_sym = Maj::symbol(&mut state, "letfn*");
    let if_sym        = Maj::symbol(&mut state, "if");
    let clauses_sym   = Maj::symbol(&mut state, "clauses");
    let body_sym      = Maj::symbol(&mut state, "body");
    let nilp_sym      = Maj::symbol(&mut state, "nilp");
    let cdr_sym       = Maj::symbol(&mut state, "cdr");
    let cons_sym    = Maj::symbol(&mut state, "cons");

    maj_list!(
        Maj::symbol(&mut state, "defmac"),
        letfnstar_sym.clone(),
        maj_dotted_list!(
            clauses_sym.clone(),
            body_sym.clone()),
        maj_list!(
            if_sym.clone(),
            maj_list!(nilp_sym.clone(),
                      clauses_sym.clone()),
            maj_list!(
                cons_sym.clone(),
                maj_list!(
                    Maj::quote(),
                    Maj::do_sym()),
                body_sym.clone()),
            maj_list!(
                Maj::quasiquote(),
                maj_list!(
                    Maj::symbol(&mut state, "letfn"),
                    maj_list!(
                        maj_list!(
                            Maj::unquote(),
                            maj_list!(
                                Maj::symbol(&mut state, "car"),
                                clauses_sym.clone()))),
                    maj_list!(
                        Maj::unquote(),
                        maj_list!(
                            if_sym,
                            maj_list!(
                                nilp_sym,
                                maj_list!(
                                    cdr_sym.clone(),
                                    clauses_sym.clone())),
                            maj_list!(
                                cons_sym.clone(),
                                maj_list!(
                                    Maj::quote(),
                                    Maj::do_sym()),
                                body_sym.clone()),
                            maj_list!(
                                Maj::quasiquote(),
                                maj_list!(
                                    letfnstar_sym,
                                    maj_list!(
                                        Maj::unquote(),
                                        maj_list!(
                                            cdr_sym,
                                            clauses_sym)),
                                    maj_list!(
                                        Maj::unquote_splice(),
                                        body_sym)))))))))
}

#[inline]
fn bootstrap_when(mut state: &mut MajState) -> Gc<Maj> {
    let pred_sym = Maj::symbol(&mut state, "pred");
    let body_sym = Maj::symbol(&mut state, "body");
    maj_list!(
        Maj::symbol(&mut state, "defmac"),
        Maj::symbol(&mut state, "when"),
        maj_dotted_list!(
            pred_sym.clone(), body_sym.clone()),
        maj_list!(
            Maj::quasiquote(),
            maj_list!(
                Maj::symbol(&mut state, "if"),
                maj_list!(Maj::unquote(), pred_sym),
                maj_list!(
                    Maj::do_sym(),
                        maj_list!(
                            Maj::unquote_splice(),
                            body_sym)),
                Maj::nil())))
}

#[inline]
fn bootstrap_unless(mut state: &mut MajState) -> Gc<Maj> {
    let pred_sym = Maj::symbol(&mut state, "pred");
    let body_sym = Maj::symbol(&mut state, "body");
    maj_list!(
        Maj::symbol(&mut state, "defmac"),
        Maj::symbol(&mut state, "unless"),
        maj_dotted_list!(
            pred_sym.clone(), body_sym.clone()),
        maj_list!(
            Maj::quasiquote(),
            maj_list!(
                Maj::symbol(&mut state, "if"),
                maj_list!(
                    Maj::symbol(&mut state, "not"),
                    maj_list!(Maj::unquote(), pred_sym)),
                maj_list!(
                    Maj::do_sym(),
                    maj_list!(
                        Maj::unquote_splice(),
                        body_sym)),
                Maj::nil())))
}

#[inline]
fn bootstrap_cond(mut state: &mut MajState) -> Gc<Maj> {
    let if_sym      = Maj::symbol(&mut state, "if");
    let nilp_sym    = Maj::symbol(&mut state, "nilp");
    let clauses_sym = Maj::symbol(&mut state, "clauses");
    let cond_sym    = Maj::symbol(&mut state, "cond");

    maj_list!(
        Maj::symbol(&mut state, "defmac"),
        cond_sym.clone(),
        clauses_sym.clone(),
        maj_list!(
            if_sym.clone(),
            maj_list!(nilp_sym.clone(), clauses_sym.clone()),
            Maj::nil(),
            maj_list!(
                Maj::quasiquote(),
                maj_list!(
                    if_sym.clone(),
                    maj_list!(
                        Maj::unquote(),
                        maj_list!(
                            Maj::symbol(&mut state, "caar"),
                            clauses_sym.clone())),
                    maj_list!(
                        Maj::do_sym(),
                        maj_list!(
                            Maj::unquote_splice(),
                            maj_list!(
                                Maj::symbol(&mut state, "cdar"),
                                clauses_sym.clone()))),
                    maj_list!(
                        Maj::unquote(),
                        maj_list!(
                            if_sym,
                            maj_list!(
                                nilp_sym,
                                maj_list!(
                                    Maj::symbol(&mut state, "cdr"),
                                    clauses_sym.clone())),
                            Maj::nil(),
                            maj_list!(
                                Maj::symbol(&mut state, "cons"),
                                maj_list!(
                                    Maj::quote(),
                                    cond_sym),
                                maj_list!(
                                    Maj::symbol(&mut state, "cdr"),
                                    clauses_sym))))))))
}

#[inline]
fn bootstrap_until(mut state: &mut MajState) -> Gc<Maj> {
    let pred_sym = Maj::symbol(&mut state, "pred");
    let body_sym = Maj::symbol(&mut state, "body");

    maj_list!(
        Maj::symbol(&mut state, "defmac"),
        Maj::symbol(&mut state, "until"),
        maj_dotted_list!(pred_sym.clone(),
                         body_sym.clone()),
        maj_list!(
            Maj::quasiquote(),
            maj_list!(
                Maj::symbol(&mut state, "while"),
                maj_list!(
                    Maj::symbol(&mut state, "not"),
                    maj_list!(Maj::unquote(),
                              pred_sym)),
                maj_list!(
                    Maj::unquote_splice(),
                    body_sym))))
}

fn bootstrap_with_open_stream(mut state: &mut MajState) -> Gc<Maj> {
    let sym_sym = Maj::symbol(&mut state, "sym");
    let dir_sym = Maj::symbol(&mut state, "dir");
    let file_sym = Maj::symbol(&mut state, "file");
    let body_sym = Maj::symbol(&mut state, "body");

    maj_list!(
        Maj::symbol(&mut state, "defmac"),
        Maj::symbol(&mut state, "with-open-stream"),
        maj_dotted_list!(
            maj_list!(sym_sym.clone(),
                      dir_sym.clone(),
                      file_sym.clone()),
            body_sym.clone()),
        maj_list!(
            Maj::quasiquote(),
            maj_list!(
                Maj::symbol(&mut state, "let"),
                maj_list!(
                    maj_list!(
                        maj_list!(
                            Maj::unquote(),
                            sym_sym.clone()),
                        maj_list!(
                            Maj::symbol(&mut state, "open-stream"),
                            maj_list!(Maj::unquote(), dir_sym),
                            maj_list!(Maj::unquote(), file_sym)))),
                maj_list!(
                    Maj::symbol(&mut state, "unwind-protect"),
                    maj_list!(
                        Maj::do_sym(),
                        maj_list!(Maj::unquote_splice(), body_sym)),
                    maj_list!(
                        Maj::symbol(&mut state, "close-stream"),
                        maj_list!(Maj::unquote(), sym_sym))))))
}

fn bootstrap_repeat(mut state: &mut MajState) -> Gc<Maj> {
    let n       = Maj::symbol(&mut state, "n");
    let body    = Maj::symbol(&mut state, "body");
    let let_sym = Maj::symbol(&mut state, "let");
    let it      = Maj::symbol(&mut state, "it");
    let res     = Maj::symbol(&mut state, "res");
    let gensym  = Maj::symbol(&mut state, "gensym");
    let set     = Maj::symbol(&mut state, "set");

    maj_list!(
        Maj::symbol(&mut state, "defmac"),
        Maj::symbol(&mut state, "repeat"),
        maj_dotted_list!(n.clone(), body.clone()),
        maj_list!(
            let_sym.clone(),
            maj_list!(
                maj_list!(it.clone(),
                          maj_list!(gensym.clone())),
                maj_list!(res.clone(),
                          maj_list!(gensym.clone()))),
            maj_list!(
                Maj::quasiquote(),
                maj_list!(
                    let_sym,
                    maj_list!(
                        maj_list!(
                            maj_list!(Maj::unquote(),
                                      it.clone()),
                            maj_list!(Maj::unquote(),
                                      n)),
                        maj_list!(
                            maj_list!(Maj::unquote(),
                                      res.clone()),
                            Maj::nil())),
                    maj_list!(
                        Maj::symbol(&mut state, "while"),
                        maj_list!(
                            Maj::symbol(&mut state, ">"),
                            maj_list!(Maj::unquote(),
                                      it.clone()),
                            Maj::integer(0)),
                        maj_list!(
                            set.clone(),
                            maj_list!(
                                Maj::unquote(),
                                res.clone()),
                            maj_list!(
                                Maj::do_sym(),
                                maj_list!(
                                    Maj::unquote_splice(),
                                    body))),
                        maj_list!(
                            set,
                            maj_list!(
                                Maj::unquote(),
                                it.clone()),
                            maj_list!(
                                Maj::symbol(&mut state, "1-"),
                                maj_list!(
                                    Maj::unquote(),
                                    it)))),
                    maj_list!(Maj::unquote(), res)))))
                                
}

#[inline]
fn bootstrap_car_cdr(mut state: &mut MajState) -> Vec<Gc<Maj>> {
    let def  = Maj::symbol(&mut state, "def");
    let defn = Maj::symbol(&mut state, "defn");
    let car  = Maj::symbol(&mut state, "car");
    let cdr  = Maj::symbol(&mut state, "cdr");
    let caar = Maj::symbol(&mut state, "caar");
    let cdar = Maj::symbol(&mut state, "cdar");
    let cadr = Maj::symbol(&mut state, "cadr");
    let cddr = Maj::symbol(&mut state, "cddr");
    let x    = Maj::symbol(&mut state, "x");

    vec![
        // (defn caar (x) (car (car x)))
        maj_list!(
            defn.clone(),
            caar.clone(),
            maj_list!(x.clone()),
            maj_list!(
                car.clone(),
                maj_list!(car.clone(),
                          x.clone()))),

        // (defn cadr (x) (car (cdr x)))
        maj_list!(
            defn.clone(),
            cadr.clone(),
            maj_list!(x.clone()),
            maj_list!(
                car.clone(),
                maj_list!(cdr.clone(),
                          x.clone()))),

        // (defn cdar (x) (cdr (car x)))
        maj_list!(
            defn.clone(),
            cdar.clone(),
            maj_list!(x.clone()),
            maj_list!(
                cdr.clone(),
                maj_list!(car.clone(),
                          x.clone()))),

        // (defn cddr (x) (cdr (cdr x)))
        maj_list!(
            defn.clone(),
            cddr.clone(),
            maj_list!(x.clone()),
            maj_list!(
                cdr.clone(),
                maj_list!(cdr.clone(),
                          x.clone()))),

        // (def first car)
        maj_list!(
            def.clone(),
            Maj::symbol(&mut state, "first"),
            car.clone()),

        // (def rest cdr)
        maj_list!(
            def.clone(),
            Maj::symbol(&mut state, "rest"),
            cdr.clone()),

        // (def first-of-first caar)
        maj_list!(
            def.clone(),
            Maj::symbol(&mut state,
                        "first-of-first"),
            caar.clone()),

        // (def second cadr)
        maj_list!(
            def.clone(),
            Maj::symbol(&mut state, "second"),
            cadr.clone()),

        // (def rest-of-first cdar)
        maj_list!(
            def.clone(),
            Maj::symbol(&mut state,
                        "rest-of-first"),
            cdar.clone()),

        // (def rest-of-rest cddr)
        maj_list!(
            def.clone(),
            Maj::symbol(&mut state,
                        "rest-of-rest"),
            cddr.clone()),


        // (defn third (x) (car (cddr x)))
        maj_list!(
            defn.clone(),
            Maj::symbol(&mut state, "third"),
            maj_list!(x.clone()),
            maj_list!(
                car.clone(),
                maj_list!(
                    cddr.clone(),
                    x.clone()))),

        // (defn fourth (x) (cadr (cddr x)))
        maj_list!(
            defn.clone(),
            Maj::symbol(&mut state, "fourth"),
            maj_list!(x.clone()),
            maj_list!(
                cadr.clone(),
                maj_list!(
                    cddr.clone(),
                    x.clone()))),
    ]
}

#[inline]
fn bootstrap_map(mut state: &mut MajState) -> Gc<Maj> {
    let map_sym = Maj::symbol(&mut state, "map");
    let x_sym = Maj::symbol(&mut state, "x");
    let xs_sym = Maj::symbol(&mut state, "xs");
    let f_sym = Maj::symbol(&mut state, "f");
    maj_list!(
        Maj::symbol(&mut state, "defn"),
        map_sym.clone(),
        maj_list!(
            f_sym.clone(),
            maj_dotted_list!(x_sym.clone(),
                             xs_sym.clone())),
        maj_list!(
            Maj::symbol(&mut state, "unless"),
            maj_list!(
                Maj::symbol(&mut state, "nilp"),
                x_sym.clone()),
            maj_list!(
                Maj::symbol(&mut state, "cons"),
                maj_list!(f_sym.clone(), x_sym),
                maj_list!(map_sym, f_sym, xs_sym))))
}

#[inline]
fn bootstrap_mapc(mut state: &mut MajState) -> Gc<Maj> {
    let mapc_sym = Maj::symbol(&mut state, "mapc");
    let x_sym = Maj::symbol(&mut state, "x");
    let xs_sym = Maj::symbol(&mut state, "xs");
    let f_sym = Maj::symbol(&mut state, "f");

    maj_list!(
        Maj::symbol(&mut state, "defn"),
        mapc_sym.clone(),
        maj_list!(
            f_sym.clone(),
            maj_dotted_list!(x_sym.clone(),
                             xs_sym.clone())),
        maj_list!(
            Maj::symbol(&mut state, "unless"),
            maj_list!(
                Maj::symbol(&mut state, "nilp"),
                x_sym.clone()),
            maj_list!(f_sym.clone(), x_sym),
            maj_list!(mapc_sym, f_sym, xs_sym)))
}

#[inline]
fn bootstrap_vectorequal(mut state: &mut MajState) -> Gc<Maj> {
    let when         = Maj::symbol(&mut state, "when");
    let set          = Maj::symbol(&mut state, "set");
    let vec_type     = Maj::symbol(&mut state, "vec-type");
    let vec_length   = Maj::symbol(&mut state, "vec-length");
    let vec_at       = Maj::symbol(&mut state, "vec-at");
    let len          = Maj::symbol(&mut state, "len");
    let i            = Maj::symbol(&mut state, "i");
    let continue_sym = Maj::symbol(&mut state, "continue");
    let va           = Maj::symbol(&mut state, "va");
    let vb           = Maj::symbol(&mut state, "vb");

    maj_list!(
        Maj::symbol(&mut state, "defn"),
        Maj::symbol(&mut state, "vector="),
        maj_list!(va.clone(), vb.clone()),
        maj_list!(
            when.clone(),
            maj_list!(Maj::symbol(&mut state, "eq"),
                      maj_list!(vec_type.clone(), va.clone()),
                      maj_list!(vec_type.clone(), vb.clone())),
            maj_list!(
                Maj::symbol(&mut state, "let*"),
                maj_list!(
                    maj_list!(len.clone(),
                              maj_list!(vec_length.clone(),
                                        va.clone())),
                    maj_list!(i.clone(), Maj::integer(0)),
                    maj_list!(continue_sym.clone(), Maj::t())),
                maj_list!(
                    when.clone(),
                    maj_list!(
                        Maj::symbol(&mut state, "="),
                        len.clone(),
                        maj_list!(vec_length.clone(),
                                  vb.clone())),
                    maj_list!(
                        Maj::symbol(&mut state, "while"),
                        maj_list!(
                            Maj::symbol(&mut state, "and"),
                            maj_list!(
                                Maj::symbol(&mut state, "<"),
                                i.clone(),
                                len),
                            continue_sym.clone()),
                        maj_list!(
                            Maj::symbol(&mut state, "unless"),
                            maj_list!(
                                Maj::symbol(&mut state, "equal"),
                                maj_list!(vec_at.clone(),
                                          i.clone(),
                                          va.clone()),
                                maj_list!(vec_at.clone(),
                                          i.clone(),
                                          vb.clone())),
                            maj_list!(set.clone(),
                                      continue_sym.clone(),
                                      Maj::nil())),
                        maj_list!(
                            set,
                            i.clone(),
                            maj_list!(
                                Maj::symbol(&mut state, "1+"),
                                i))),
                    continue_sym))))
}

#[inline]
fn bootstrap_equal(mut state: &mut MajState) -> Gc<Maj> {
    let and     = Maj::symbol(&mut state, "and");
    let equal   = Maj::symbol(&mut state, "equal");
    let numberp = Maj::symbol(&mut state, "numberp");
    let vectorp = Maj::symbol(&mut state, "vectorp");
    let symbolp = Maj::symbol(&mut state, "symbolp");
    let consp   = Maj::symbol(&mut state, "consp");
    let atomp   = Maj::symbol(&mut state, "atomp");
    let car     = Maj::symbol(&mut state, "car");
    let cdr     = Maj::symbol(&mut state, "cdr");
    let x       = Maj::symbol(&mut state, "x");
    let y       = Maj::symbol(&mut state, "y");
    
    maj_list!(
        Maj::symbol(&mut state, "defn"),
        equal.clone(),
        maj_list!(x.clone(), y.clone()),
        maj_list!(
            Maj::symbol(&mut state, "cond"),
            maj_list!(
                maj_list!(and.clone(),
                          maj_list!(numberp.clone(),
                                    x.clone()),
                          maj_list!(numberp, y.clone())),
                maj_list!(Maj::symbol(&mut state, "="),
                          x.clone(), y.clone())),
            maj_list!(
                maj_list!(and.clone(),
                          maj_list!(vectorp.clone(),
                                    x.clone()),
                          maj_list!(vectorp, y.clone())),
                maj_list!(Maj::symbol(&mut state, "vector="),
                          x.clone(), y.clone())),
            maj_list!(
                maj_list!(and.clone(),
                          maj_list!(symbolp.clone(),
                                    x.clone()),
                          maj_list!(symbolp, y.clone())),
                maj_list!(Maj::symbol(&mut state, "eq"),
                          x.clone(), y.clone())),
            maj_list!(
                maj_list!(and.clone(),
                          maj_list!(consp.clone(),
                                    x.clone()),
                          maj_list!(consp, y.clone())),
                maj_list!(
                    Maj::symbol(&mut state, "when"),
                    maj_list!(equal.clone(),
                              maj_list!(car.clone(), x.clone()),
                              maj_list!(car, y.clone())),
                    maj_list!(equal,
                              maj_list!(cdr.clone(), x.clone()),
                              maj_list!(cdr, y.clone())))),
            maj_list!(
                maj_list!(and,
                          maj_list!(atomp.clone(), x.clone()),
                          maj_list!(atomp, y.clone())),
                maj_list!(Maj::symbol(&mut state, "id"), x, y)),
            maj_list!(Maj::t(), Maj::nil())))
}

#[inline]
fn bootstrap_assp(mut state: &mut MajState) -> Gc<Maj> {
    let proc_sym = Maj::symbol(&mut state, "proc");
    let x_sym    = Maj::symbol(&mut state, "x");
    let xs_sym   = Maj::symbol(&mut state, "xs");
    let key_sym  = Maj::symbol(&mut state, "key");
    let rest_sym = Maj::symbol(&mut state, "rest");
    let assp_sym = Maj::symbol(&mut state, "assp");

    maj_list!(
        Maj::symbol(&mut state, "defn"), assp_sym.clone(),
        maj_list!(proc_sym.clone(),
                  maj_dotted_list!(x_sym.clone(),
                                   xs_sym.clone())),
        maj_list!(
            Maj::symbol(&mut state, "unless"),
            maj_list!(Maj::symbol(&mut state, "nilp"),
                      x_sym.clone()),
            maj_list!(
                Maj::symbol(&mut state, "let"),
                maj_list!(
                    maj_list!(
                        maj_dotted_list!(key_sym.clone(),
                                         rest_sym.clone()),
                        x_sym.clone())),
                maj_list!(
                    Maj::symbol(&mut state, "or"),
                    maj_list!(
                        Maj::symbol(&mut state, "and"),
                        maj_list!(proc_sym.clone(),
                                  key_sym),
                        x_sym),
                    maj_list!(assp_sym, proc_sym, xs_sym)))))
}

#[inline]
fn bootstrap_assoc(mut state: &mut MajState) -> Gc<Maj> {
    let sym_sym   = Maj::symbol(&mut state, "sym");
    let alist_sym = Maj::symbol(&mut state, "alist");
    
    maj_list!(
        Maj::symbol(&mut state, "defn"),
        Maj::symbol(&mut state, "assoc"),
        maj_list!(sym_sym.clone(), alist_sym.clone()),
        maj_list!(
            Maj::symbol(&mut state, "assp"),
            maj_list!(
                Maj::symbol(&mut state, "equal"),
                sym_sym),
            alist_sym))
}

fn bootstrap_one_plusless(mut state: &mut MajState) -> Vec<Gc<Maj>> {
    let defn_sym = Maj::symbol(&mut state, "defn");
    let x_sym    = Maj::symbol(&mut state, "x");
    let plus_sym = Maj::symbol(&mut state, "+");
    vec![
        maj_list!(defn_sym.clone(),
                  Maj::symbol(&mut state, "1+"),
                  maj_list!(x_sym.clone()),
                  maj_list!(plus_sym.clone(),
                            Maj::integer(1),
                            x_sym.clone())),
        maj_list!(defn_sym,
                  Maj::symbol(&mut state, "1-"),
                  maj_list!(x_sym.clone()),
                  maj_list!(plus_sym,
                            Maj::integer(-1),
                            x_sym)),
    ]
}

fn bootstrap_member(mut state: &mut MajState) -> Gc<Maj> {
    let member = Maj::symbol(&mut state, "member");
    let elt    = Maj::symbol(&mut state, "elt");
    let lst    = Maj::symbol(&mut state, "lst");
    let equal  = Maj::symbol(&mut state, "equal");
    let x      = Maj::symbol(&mut state, "x");
    let rest   = Maj::symbol(&mut state, "rest");
    
    maj_list!(
        Maj::symbol(&mut state, "defn"),
        member.clone(),
        maj_list!(elt.clone(), lst.clone()),
        maj_list!(
            Maj::symbol(&mut state, "unless"),
            maj_list!(
                Maj::symbol(&mut state, "nilp"),
                lst.clone()),
            maj_list!(
                Maj::symbol(&mut state, "let"),
                maj_list!(
                    maj_list!(
                        maj_dotted_list!(x.clone(),
                                         rest.clone()),
                        lst.clone())),
                maj_list!(
                    Maj::symbol(&mut state, "or"),
                    maj_list!(
                        Maj::symbol(&mut state, "and"),
                        maj_list!(equal, elt.clone(), x),
                        lst),
                    maj_list!(member, elt, rest)))))
}

pub fn maj_gen_bootstrap(mut state: &mut MajState) {
    use crate::printing::maj_format;

    let mut expressions: Vec<Gc<Maj>> = vec![];

    expressions.append(&mut vec![
        bootstrap_defmac(&mut state),
        bootstrap_defn(&mut state),
        bootstrap_let(&mut state),
        bootstrap_letstar(&mut state),
        bootstrap_letfn(&mut state),
        bootstrap_letfnstar(&mut state),
        bootstrap_when(&mut state),
        bootstrap_unless(&mut state),
        bootstrap_until(&mut state),
        bootstrap_with_open_stream(&mut state),
        bootstrap_repeat(&mut state),
        bootstrap_map(&mut state),
        bootstrap_mapc(&mut state),
        bootstrap_vectorequal(&mut state),
        bootstrap_cond(&mut state),
        bootstrap_equal(&mut state),
        bootstrap_assp(&mut state),
        bootstrap_assoc(&mut state),
        bootstrap_member(&mut state),
    ]);
    expressions.append(&mut bootstrap_car_cdr(&mut state));
    expressions.append(&mut bootstrap_one_plusless(&mut state));
    for expression in expressions.iter() {
        let e = maj_eval(&mut state,
                         expression.clone(),
                         Maj::nil());

        if maj_errorp(e.clone()).to_bool() {
            panic!("Bootstrap error:\n{}\nOn eval:\n{}",
                   maj_format(&state, e),
                   maj_format(&state, expression.clone()));
        }
    }

    maj_put_constants(&mut state);
}
