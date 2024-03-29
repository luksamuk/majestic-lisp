use regex::Regex;
use crate::{ maj_list, maj_dotted_list };
use crate::printing::{ maj_format, maj_format_raw };
use crate::core::{ MajState, Maj };
use crate::axioms::predicates::maj_errorp;
use crate::evaluator::maj_eval;
use crate::reader::tokenizer::maj_tokenize;
use crate::reader::parser::maj_parse;
use crate::axioms::primitives::{ maj_car, maj_macroexpand_1 };

const RE_IN_STREAM:  &'static str =
    // #<stream (in) {0x...}>
    r"(?u)^#<stream \(in\) \{0x[0-9a-z]*}>$";
const RE_OUT_STREAM: &'static str =
    // #<stream (out) {0x...}>
    r"(?u)^#<stream \(out\) \{0x[0-9a-z]*}>$";
const RE_ENVIRONMENT: &'static str =
    // #<environment {0x...}>
    r"^(?u)#<environment \{0x[0-9a-z]*\}>$";
const RE_CLOSURE: &'static str =
    // #<function (fn (x...)) {0x...}>
    // #<function (fn nil) {0x...}>
    // #<function (fn (x... . blah)) {0x...}>
    r"(?u)^#<function \(fn (nil|\((\w+\s*)+(\. \w+)?\))\) \{0x[0-9a-z]*\}>$";
const RE_MACRO: &'static str =
    // #<macro (mac (x...)) {0x...}>
    // #<macro (mac nil) {0x...}>
    // #<macro (mac (x... . blah)) {0x...}>
    r"(?u)^#<macro \(mac (nil|\((\w+\s*)+(\. \w+)?\))\) \{0x[0-9a-z]*\}>$";
const RE_PRIMITIVE: &'static str =
    // #<function (prim fn-name (arity (required X)))>
    // #<function (prim fn-name (arity (required 0)))>
    // #<function (prim fn-name (arity (variadic X)))>
    // #<function (prim fn-name (arity (variadic 0)))>
    r"(?u)^#<function \(prim ((\w|\W)+) \(arity \((required|variadic) [0-9]*\)\)\)>$";

macro_rules! test_format {
    ($state:ident, $x:expr, $y:tt) => {
        let ex = $x;
        let fmt = maj_format(&$state, ex);
        //println!("fmt = {}", fmt);
        assert_eq!(fmt, $y);
    };
}

macro_rules! multi_test {
    ($state:ident; $(($x:expr, $y:tt);)+) => {
        $(test_format!($state, $x, $y);)*
    }
}

macro_rules! test_eval_ast {
    ($state:ident, $x:expr, $env:expr, $y:tt) => {
        let ex = $x;
        let result = maj_eval(&mut $state, ex, $env);
        let fmt = maj_format(&$state, result);
        assert_eq!(fmt, $y);
    };
}

macro_rules! multi_eval_ast_test {
    ($state:ident; $(($x:expr, $env:expr, $y:tt);)+) => {
        $(test_eval_ast!($state, $x, $env, $y);)*
    }
}

macro_rules! test_parser {
    ($state:ident, $expr:tt, $toks:expr, $y:tt) => {
        let tokens = maj_tokenize($expr).unwrap();
        let expected_tokens: Vec<&str> = $toks;
        assert_eq!(tokens.len(), expected_tokens.len());
        for n in 0..tokens.len() {
            assert_eq!(tokens[n], expected_tokens[n]);
        }
        let parsed = maj_parse(&mut $state, tokens).unwrap();
        let fmt    = maj_format_raw(&$state, parsed, false);
        assert_eq!(fmt, $y);
    }
}

macro_rules! multi_parser_test {
    ($state:ident; $(($expr:tt, $toks:expr, $y:tt);)+) => {
        $(test_parser!($state, $expr, $toks, $y);)*
    }
}

macro_rules! test_parser_fail {
    ($state:ident, $expr:tt) => {
        let tokens = maj_tokenize($expr).unwrap();
        assert!(maj_parse(&mut $state, tokens).is_err());
    }
}

macro_rules! multi_parser_fail_test {
    ($state:ident; $($expr:tt;)+) => {
        $(test_parser_fail!($state, $expr);)*
    }
}

macro_rules! test_eval {
    ($state:ident, $expr:tt, $y:tt) => {
        let tokens = maj_tokenize($expr).unwrap();
        let parsed = maj_parse(&mut $state, tokens).unwrap();
        let result = maj_eval(
            &mut $state,
            Maj::cons(Maj::do_sym(), parsed),
            Maj::nil());
        assert!(!maj_errorp(result.clone()).to_bool());
        let fmt    = maj_format_raw(&$state, result, false);
        assert_eq!(fmt, $y);
    }
}

macro_rules! multi_eval_test {
    ($state:ident; $(($expr:tt, $y:tt);)+) => {
        $(test_eval!($state, $expr, $y);)*
    }
}

macro_rules! test_eval_fail {
    ($state:ident, $expr:tt) => {
        let tokens = maj_tokenize($expr).unwrap();
        let parsed = maj_parse(&mut $state, tokens).unwrap();
        let result = maj_eval(
            &mut $state,
            Maj::cons(Maj::do_sym(), parsed),
            Maj::nil());
        assert!(maj_errorp(result).to_bool());
    }
}

macro_rules! multi_eval_fail_test {
    ($state:ident; $($expr:tt;)+) => {
        $(test_eval_fail!($state, $expr);)*
    }
}

macro_rules! test_macroexpand_1 {
    ($state:ident, $expr:tt, $y:tt) => {
        let tokens = maj_tokenize($expr).unwrap();
        let parsed = maj_parse(&mut $state, tokens).unwrap();
        let parsed = maj_car(parsed);
        let (expanded, worked) =
            maj_macroexpand_1(&mut $state, parsed, Maj::nil());
        if !worked {
            panic!("Failed expansion: {}\nExpected: {}",
                   maj_format_raw(&$state, expanded, false),
                   $y);
        }
        let fmt = maj_format_raw(&$state, expanded, false);
        assert_eq!(fmt, $y);
    }
}

macro_rules! multi_macroexpand_1_test {
    ($state:ident; $(($expr:tt, $y:tt);)+) => {
        $(test_macroexpand_1!($state, $expr, $y);)*
    }
}

macro_rules! test_regex {
    ($state:ident, $x:expr, $re:tt) => {
        let ex  = $x;
        let fmt = maj_format(&$state, ex);
        let re  = Regex::new($re).unwrap();
        assert!(re.is_match(&fmt));
    };
}

macro_rules! test_fail_regex {
    ($state:ident, $x:expr, $re:tt) => {
        let ex  = $x;
        let fmt = maj_format(&$state, ex);
        let re  = Regex::new($re).unwrap();
        assert!(!re.is_match(&fmt));
    };
}

macro_rules! multi_regex_test {
    ($state:ident; $($pair:expr;)+) => {
        $(
            let (x, y) = $pair;
            test_regex!($state, x, y);
        )*
    }
}

macro_rules! test_fail {
    ($x:expr) => {
        assert!(maj_errorp($x).to_bool());
    };
}

macro_rules! test_dont_fail {
    ($x:expr) => {
        assert!(!maj_errorp($x).to_bool());
    };
}

macro_rules! multi_fail_test {
    ($($x:expr;)+) => {
        $(test_fail!($x);)*
    }
}

macro_rules! test_boolean {
    ($x:expr, $v:literal) => {
        let res = $x;
        if maj_errorp(res.clone()).to_bool() {
            panic!("Error raised during assertion");
        }
        assert_eq!(res.to_bool(), $v);
    }
}

macro_rules! multi_boolean_test {
    ($(($x:expr, $v:literal);)+) => {
        $(test_boolean!($x, $v);)*
    }
}

#[test]
fn formatter_symbol() {
    let mut state = MajState::new();
    multi_test!(
        state;
        (Maj::symbol(&mut state, "foo"),   "foo");
        (Maj::symbol(&mut state, "bar"),   "bar");
        (Maj::symbol(&mut state, "baz"),   "baz");
        (Maj::symbol(&mut state, "t"),     "t");
        (Maj::t(),                         "t");
        (Maj::symbol(&mut state, "nil"),   "nil");
        (Maj::nil(),                       "nil");
        (Maj::symbol(&mut state, "&"),     "&");
        (Maj::symbol(&mut state, "apply"), "apply");
    );
}

#[test]
fn formatter_cons_cell() {
    let mut state = MajState::new();
    let foo = Maj::symbol(&mut state, "foo");
    let bar = Maj::symbol(&mut state, "bar");
    let baz = Maj::symbol(&mut state, "baz");

    multi_test!(
        state;
        (Maj::cons(foo.clone(), bar.clone()),
         "(foo . bar)");
        (Maj::cons(foo.clone(),
                   Maj::cons(bar.clone(), baz.clone())),
         "(foo bar . baz)");
    );
}

#[test]
fn formatter_character() {
    let state = MajState::new();
    multi_test!(
        state;
        (Maj::character('a'),    "#\\a");
        (Maj::character('\x07'), "#\\␇");
        (Maj::character('\t'),   "#\\tab");
        (Maj::character(' '),    "#\\space");
        (Maj::character('\n'),   "#\\newline");
    );
}

#[test]
#[ignore]
fn formatter_stream() {
    use crate::core::types::MajStreamDirection;
    let mut state = MajState::new();
    let stdin  = state.make_stream_stdin();
    let stdout = state.make_stream_stdout();
    let istream = Maj::stream(
        &mut state,
        "test-streams-in.txt",
        MajStreamDirection::In).unwrap();
    let ostream = Maj::stream(
        &mut state,
        "test-streams-out.txt",
        MajStreamDirection::Out).unwrap();

    multi_regex_test!(
        state;
        (stdin,           RE_IN_STREAM);
        (stdout,          RE_OUT_STREAM);
        (istream.clone(), RE_IN_STREAM);
        (ostream.clone(), RE_OUT_STREAM);
    );

    state.close_stream(istream);
    state.close_stream(ostream);
}

#[test]
fn formatter_number_integer() {
    let state = MajState::new();
    multi_test!(
        state;
        (Maj::integer(1), "1");
        (Maj::integer(0), "0");
        (Maj::integer(-1), "-1");
        (Maj::integer(50), "50");
        (Maj::integer(295), "295");
    );
}

#[test]
fn formatter_number_fraction() {
    let state = MajState::new();
    multi_test!(
        state;
        (Maj::fraction(2, 3),    "2/3");
        (Maj::fraction(3, 4),    "3/4");
        (Maj::fraction(5, 8),    "5/8");
        (Maj::fraction(99, 100), "99/100");
        (Maj::fraction(5, 2),    "5/2");
        (Maj::fraction(-10, 3),  "-10/3");

        // (Maj::fraction(2, 4),    "1/2");
        // (Maj::fraction(5, 1),    "5");
        // (Maj::fraction(4, -9),   "-4/9");
        // (Maj::fraction(-4, -9),  "4/9");
    );
}

#[test]
fn formatter_number_float() {
    let state = MajState::new();
    multi_test!(
        state;
        (Maj::float(2.0), "2.0");
        (Maj::float(0.5), "0.5");
        (Maj::float(20.2), "20.2");
        (Maj::float(-16.3), "-16.3");
        (Maj::float(-9.0), "-9.0");
        (Maj::float(-7.0), "-7.0");
    );
}

#[test]
fn formatter_number_complex() {
    let state = MajState::new();
    multi_test!(
        state;
        (Maj::complex(Maj::integer(0),
                      Maj::integer(5)),
         "0J5");
        (Maj::complex(Maj::integer(3),
                      Maj::integer(1)),
         "3J1");
        (Maj::complex(Maj::fraction(2, 3),
                      Maj::float(0.5)),
         "2/3J0.5");
        (Maj::complex(Maj::integer(-2),
                      Maj::integer(-1)),
         "-2J-1");
        (Maj::complex(Maj::float(35.0),
                      Maj::fraction(-2, 9)),
         "35.0J-2/9");
    );
}

#[test]
fn formatter_dotted_list() {
    let mut state = MajState::new();
    let foo  = Maj::symbol(&mut state, "foo");
    let bar  = Maj::symbol(&mut state, "bar");
    let baz  = Maj::symbol(&mut state, "baz");
    let quux = Maj::symbol(&mut state, "quux");

    multi_test!(
        state;
        (maj_dotted_list!(foo.clone(), bar.clone()),
         "(foo . bar)");
        (maj_dotted_list!(foo.clone(), bar.clone(),
                          baz.clone()),
         "(foo bar . baz)");
        (maj_dotted_list!(foo.clone(), bar.clone(),
                          baz.clone(), quux.clone()),
         "(foo bar baz . quux)");
    );
}

#[test]
fn formatter_proper_list() {
    let mut state = MajState::new();
    let foo  = Maj::symbol(&mut state, "foo");
    let bar  = Maj::symbol(&mut state, "bar");
    let baz  = Maj::symbol(&mut state, "baz");
    let quux = Maj::symbol(&mut state, "quux");

    multi_test!(
        state;
        (maj_list!(foo.clone()), "(foo)");
        (maj_list!(foo.clone(), bar.clone()),
         "(foo bar)");
        (maj_list!(foo.clone(), bar.clone(),
                   baz.clone()),
         "(foo bar baz)");
        (maj_list!(foo.clone(), bar.clone(),
                   baz.clone(), quux.clone()),
         "(foo bar baz quux)");
        (maj_list!(Maj::quote(), foo.clone()),
         "'foo");
        (maj_list!(Maj::quasiquote(),
                   maj_list!(
                       foo.clone(),
                       maj_list!(
                           Maj::unquote(),
                           bar.clone()),
                       maj_list!(
                           Maj::unquote_splice(),
                           baz.clone()))),
         "`(foo ,bar ,@baz)");
    );
}

#[test]
fn formatter_vector_integers() {
    let state = MajState::new();
    multi_test!(
        state;
        (Maj::vector_integer(vec![]), "[]");
        (Maj::vector_integer(vec![1, 2, 3, 4]),
         "[1 2 3 4]");
        (Maj::vector_integer(vec![-10, 2, 5, 93]),
         "[-10 2 5 93]");
    );
}

#[test]
fn formatter_vector_float() {
    let state = MajState::new();
    multi_test!(
        state;
        (Maj::vector_float(vec![]), "[]");
        (Maj::vector_float(vec![1.0, 2.0, 3.0, 4.0]),
         "[1.0 2.0 3.0 4.0]");
        (Maj::vector_float(vec![-10.0, 2.0, 5.0, 93.0]),
         "[-10.0 2.0 5.0 93.0]");
    );
}

#[test]
fn formatter_string() {
    let state = MajState::new();
    multi_test!(
        state;
        (Maj::string(""), "\"\"");
        (Maj::string("abc"), "\"abc\"");
    );
}

#[test]
fn formatter_vector_any() {
    let mut state = MajState::new();
    multi_test!(
        state;
        (Maj::vector_any(vec![]), "[]");
        (Maj::vector_any(vec![
            Maj::integer(1),
            Maj::fraction(2, 3),
            Maj::complex(Maj::integer(2),
                         Maj::integer(5)),
            Maj::integer(6),
            maj_list!(Maj::symbol(&mut state, "a"),
                      Maj::symbol(&mut state, "b"),
                      Maj::symbol(&mut state, "c"))]),
         "[1 2/3 2J5 6 (a b c)]");
        (Maj::vector_any(
            vec![Maj::vector_any(vec![]),
                 Maj::vector_any(vec![]),
                 Maj::vector_any(vec![])]),
         "[[] [] []]");
        (Maj::vector_any(
            vec![Maj::character('H'),
                 Maj::character('e'),
                 Maj::character('l'),
                 Maj::character('l'),
                 Maj::character('o')]),
         "[#\\H #\\e #\\l #\\l #\\o]");
    );
}

#[test]
fn formatter_closure() {
    let mut state = MajState::new();
    let x = Maj::symbol(&mut state, "x");
    let fst = Maj::symbol(&mut state, "fst");
    let snd = Maj::symbol(&mut state, "snd");
    let rest = Maj::symbol(&mut state, "rest");
    multi_regex_test!(
        state;
        // (lit closure nil (x) ((* x x)))
        // #<function (fn (x)) {0x...}>
        (maj_list!(Maj::lit(), Maj::closure(), Maj::nil(),
                   maj_list!(x.clone()),
                   maj_list!(
                       maj_list!(Maj::symbol(&mut state, "*"),
                                 x.clone(), x.clone()))),
         RE_CLOSURE);
        // (lit closure nil nil (2))
        // #<function (fn nil) {0x...}>
        (maj_list!(Maj::lit(), Maj::closure(), Maj::nil(),
                   Maj::nil(),
                   maj_list!(Maj::integer(2))),
         RE_CLOSURE);
        // (lit closure nil (x . rest) ((x)))
        // #<function (fn (x . rest)) {0x...}>
        (maj_list!(Maj::lit(), Maj::closure(), Maj::nil(),
                   maj_dotted_list!(x.clone(),
                                    rest.clone()),
                   maj_list!(maj_list!(x.clone()))),
         RE_CLOSURE);
        // (lit closure nil (fst snd . rest) ((fst)))
        // #<function (fn (fst snd . rest)) {0x...}>
        (maj_list!(Maj::lit(), Maj::closure(), Maj::nil(),
                   maj_dotted_list!(fst.clone(),
                                    snd.clone(),
                                    rest.clone()),
                   maj_list!(maj_list!(fst.clone()))),
         RE_CLOSURE);
    );
    test_fail_regex!(state,
                     // (lit 2)
                     maj_list!(Maj::lit(), Maj::integer(2)),
                     RE_CLOSURE);
}

#[test]
fn formatter_primitive() {
    let mut state = MajState::new();

    let prim_fetch = |mut state: &mut MajState, name| {
        let sym = Maj::symbol(&mut state, name);
        let result =
            maj_eval(&mut state, sym, Maj::nil());
        assert!(!maj_errorp(result.clone()).to_bool());
        result
    };
    
    multi_regex_test! {
        state;
        // (lit prim cons required 2) => (defn cons (a b) ...)
        (prim_fetch(&mut state, "cons"), RE_PRIMITIVE);
        // (lit prim coin nil nil) => (defn coin () ...)
        (prim_fetch(&mut state, "coin"), RE_PRIMITIVE);
        // (lit prim print variadic 1) => (defn print (fmt . rest) ...)
        (prim_fetch(&mut state, "print"), RE_PRIMITIVE);
        // (lit prim list variadic 0) => (defn list rest ...)
        (prim_fetch(&mut state, "list"), RE_PRIMITIVE);
    }
}

#[test]
fn formatter_macro() {
    let mut state = MajState::new();

    let x = Maj::symbol(&mut state, "x");
    let fst = Maj::symbol(&mut state, "fst");
    let snd = Maj::symbol(&mut state, "snd");
    let rest = Maj::symbol(&mut state, "rest");
    
    let macro_writer = |lambda_list, body| {
        // (lit macro (lit closure nil <lambda-list> <body>))
        // where <body> is also a list of many expressions
        maj_list!(Maj::lit(), Maj::macro_sym(),
                  maj_list!(
                      Maj::lit(),
                      Maj::closure(),
                      Maj::nil(),
                      lambda_list, body))
    };

    multi_regex_test!(
        state;
        // #<macro (mac (x)) {0x...}>
        // (mac (x) x)
        (macro_writer(
            maj_list!(x.clone()),
            maj_list!(x.clone())),
         RE_MACRO);
        // #<macro (mac nil) {0x...}>
        // (mac () x)
        (macro_writer(
            Maj::nil(),
            maj_list!(x.clone())),
         RE_MACRO);
        
        // #<macro (mac (x . rest)) {0x...}>
        // (mac (x . rest) (x rest))
        (macro_writer(
            maj_dotted_list!(x.clone(), rest.clone()),
            maj_list!(x.clone(), rest.clone())),
         RE_MACRO);
        
        // #<macro (mac (fst snd . rest)) {0x...}>
        // (mac (fst snd . rest) (fst snd rest))
        (macro_writer(
            maj_dotted_list!(fst.clone(), snd.clone(),
                             rest.clone()),
            maj_list!(fst.clone(), snd.clone(), rest.clone())),
         RE_MACRO);
    );
}

#[test]
fn formatter_error() {
    let state = MajState::new();

    let error_writer = |format, args| {
        maj_dotted_list!(
            Maj::lit(), Maj::error(),
            format, args)
    };
    
    multi_test!(
        state;
        (error_writer(Maj::string("Hello"),
                      Maj::nil()),
         "Hello");
        (error_writer(Maj::string("Hello, {}"),
                      maj_list!(
                          Maj::string("world"))),
         "Hello, world");
        (error_writer(Maj::string("{} + {} = {}"),
                      maj_list!(
                          Maj::integer(1),
                          Maj::integer(2),
                          Maj::integer(3))),
         "1 + 2 = 3");
        (error_writer(Maj::string("{} is not a list such as {}"),
                      maj_list!(
                          Maj::cons(Maj::integer(1),
                                    Maj::integer(2)),
                          maj_list!(Maj::integer(1),
                                    Maj::integer(2)))),
         "(1 . 2) is not a list such as (1 2)");
    );
}

#[test]
#[ignore]
fn formatter_prettyprinting() {
    unimplemented!();
}

#[test]
fn number_coercion() {
    use crate::axioms::primitives::maj_number_coerce;
    let mut state = MajState::new();
    let integer  = Maj::symbol(&mut state, "integer");
    let fraction = Maj::symbol(&mut state, "fraction");
    let float    = Maj::symbol(&mut state, "float");
    let complex  = Maj::symbol(&mut state, "complex");

    multi_test!(
        state;
        
        // Integer coercion
        (maj_number_coerce(&mut state, integer.clone(),
                           Maj::integer(5)),
         "5");
        (maj_number_coerce(&mut state, float.clone(),
                           Maj::integer(5)),
         "5.0");
        (maj_number_coerce(&mut state, fraction.clone(),
                           Maj::integer(5)),
         "5/1");
        (maj_number_coerce(&mut state, complex.clone(),
                           Maj::integer(5)),
         "5J0.0");

        // Float coercion
        (maj_number_coerce(&mut state, integer.clone(),
                           Maj::float(5.3)),
         "5");
        (maj_number_coerce(&mut state, float.clone(),
                           Maj::float(5.3)),
         "5.3");
        (maj_number_coerce(&mut state, fraction.clone(),
                           Maj::float(5.3)),
         "53/10");
        (maj_number_coerce(&mut state, complex.clone(),
                           Maj::float(5.3)),
         "5.3J0.0");

        // Fraction coercion
        (maj_number_coerce(&mut state, integer.clone(),
                           Maj::fraction(3, 2)),
         "1");
        (maj_number_coerce(&mut state, float.clone(),
                           Maj::fraction(3, 2)),
         "1.5");
        (maj_number_coerce(&mut state, fraction.clone(),
                           Maj::fraction(3, 2)),
         "3/2");
        (maj_number_coerce(&mut state, complex.clone(),
                           Maj::fraction(3, 2)),
         "3/2J0.0");

        // Complex coercion
        (maj_number_coerce(&mut state, integer.clone(),
                           Maj::complex(
                               Maj::integer(3),
                               Maj::integer(2))),
         "3");
        (maj_number_coerce(&mut state, float.clone(),
                           Maj::complex(
                               Maj::integer(3),
                               Maj::integer(2))),
         "3.0");
        (maj_number_coerce(&mut state, fraction.clone(),
                           Maj::complex(
                               Maj::integer(3),
                               Maj::integer(2))),
         "3/1");
        (maj_number_coerce(&mut state, complex.clone(),
                           Maj::complex(
                               Maj::integer(3),
                               Maj::integer(2))),
         "3J2");
    );
}

#[test]
fn predicates_symbolp() {
    use crate::axioms::predicates::maj_symbolp;
    multi_boolean_test!(
        (maj_symbolp(Maj::t()), true);
        (maj_symbolp(Maj::fraction(2, 3)), false);
    );
}

#[test]
fn predicates_eq() {
    use crate::axioms::predicates::maj_eq;
    multi_boolean_test!(
        (maj_eq(Maj::t(), Maj::t()), true);
        (maj_eq(Maj::nil(), Maj::nil()), true);
        (maj_eq(Maj::t(), Maj::nil()), false);
        (maj_eq(Maj::integer(20), Maj::t()), false);
    );
}

#[test]
fn predicates_nilp() {
    use crate::axioms::predicates::maj_nilp;
    multi_boolean_test!(
        (maj_nilp(Maj::nil()), true);
        (maj_nilp(Maj::integer(5)), false);
    );
}

#[test]
fn predicates_consp() {
    use crate::axioms::predicates::maj_consp;
    let mut state = MajState::new();
    multi_boolean_test!(
        (maj_consp(Maj::cons(Maj::integer(2),
                             Maj::integer(3))),
         true);
        (maj_consp(Maj::cons(Maj::symbol(&mut state, "a"),
                             Maj::symbol(&mut state, "b"))),
         true);
        (maj_consp(Maj::integer(10)), false);
    );
}

#[test]
fn predicates_atomp() {
    use crate::axioms::predicates::maj_atomp;
    let mut state = MajState::new();
    multi_boolean_test!(
        (maj_atomp(Maj::integer(1)), true);
        (maj_atomp(Maj::symbol(&mut state, "a")), true);
        (maj_atomp(maj_list!(Maj::integer(1),
                             Maj::integer(2),
                             Maj::integer(3))),
         false);
    );
}

#[test]
fn predicates_charp() {
    use crate::axioms::predicates::maj_charp;
    multi_boolean_test!(
        (maj_charp(Maj::character('a')), true);
        (maj_charp(Maj::t()), false);
    );
}

#[test]
fn predicates_char_equals() {
    use crate::axioms::predicates::maj_char_equals;
    let small_a = Maj::character('a');
    let big_a   = Maj::character('A');
    let bel     = Maj::character('\x07');
    multi_boolean_test!(
        (maj_char_equals(small_a.clone(), small_a.clone()),
         true);
        (maj_char_equals(small_a.clone(), big_a.clone()),
         false);
    );
    multi_fail_test!(
        maj_char_equals(small_a.clone(), Maj::t());
        maj_char_equals(Maj::nil(), bel);
    );
}

#[test]
fn predicates_streamp() {
    use crate::axioms::predicates::maj_streamp;
    use crate::core::types::MajStreamDirection;
    let mut state = MajState::new();
    let ostream = Maj::stream(
        &mut state,
        "test-streams-out.txt",
        MajStreamDirection::Out).unwrap();
    let stdout = state.make_stream_stdout();
    multi_boolean_test!(
        (maj_streamp(ostream.clone()), true);
        (maj_streamp(stdout), true);
        (maj_streamp(Maj::t()), false);
    );
    state.close_stream(ostream);
}

#[test]
fn predicates_numberp() {
    use crate::axioms::predicates::maj_numberp;
    multi_boolean_test!(
        (maj_numberp(Maj::integer(2)), true);
        (maj_numberp(Maj::float(0.5)), true);
        (maj_numberp(Maj::fraction(2, 3)), true);
        (maj_numberp(Maj::complex(Maj::integer(0),
                                  Maj::float(2.0))),
         true);
        (maj_numberp(Maj::t()), false);
    );
}

#[test]
fn predicates_integerp() {
    use crate::axioms::predicates::maj_integerp;
    multi_boolean_test!(
        (maj_integerp(Maj::integer(2)), true);
        (maj_integerp(Maj::fraction(2, 3)), false);
    );
}

#[test]
fn predicates_floatp() {
    use crate::axioms::predicates::maj_floatp;
    multi_boolean_test!(
        (maj_floatp(Maj::float(2.5)), true);
        (maj_floatp(Maj::float(2.0)), true);
        (maj_floatp(Maj::float(0.5)), true);
        (maj_floatp(Maj::integer(3)), false);
    );
}

#[test]
fn predicates_fractionp() {
    use crate::axioms::predicates::maj_fractionp;
    multi_boolean_test!(
        (maj_fractionp(Maj::fraction(2, 3)), true);
        (maj_fractionp(Maj::fraction(5, 8)), true);
        (maj_fractionp(Maj::integer(8)), false);
    );
}

#[test]
fn predicates_complexp() {
    use crate::axioms::predicates::maj_complexp;
    multi_boolean_test!(
        (maj_complexp(Maj::complex(Maj::float(0.5),
                                   Maj::integer(1))),
         true);
        (maj_complexp(Maj::complex(Maj::integer(-10),
                                   Maj::integer(-3))),
         true);
        (maj_complexp(Maj::integer(5)), false);
        // (maj_complexp(Maj::complex(Maj::integer(5),
        //                            Maj::integer(0))),
        //  false);
        (maj_complexp(Maj::complex(Maj::integer(5),
                                   Maj::float(0.0))),
         true);
    );
}

#[test]
fn predicates_vectorp() {
    use crate::axioms::predicates::maj_vectorp;
    multi_boolean_test!(
        (maj_vectorp(
            Maj::vector_integer(vec![1, 2, 3])), true);
        (maj_vectorp(Maj::t()), false);
    );
}

#[test]
fn predicates_id() {
    use crate::axioms::predicates::maj_id;
    let mut state = MajState::new();
    multi_boolean_test!(
        (maj_id(Maj::symbol(&mut state, "a"),
                Maj::symbol(&mut state, "a")),
         true);
        (maj_id(Maj::character('c'),
                Maj::character('c')),
         true);
        (maj_id(maj_list!(Maj::symbol(&mut state, "a"),
                          Maj::symbol(&mut state, "b")),
                maj_list!(Maj::symbol(&mut state, "a"),
                          Maj::symbol(&mut state, "b"))),
         false);
        (maj_id(Maj::integer(5), Maj::integer(5)),
         false);
    );
}

#[test]
fn predicates_proper_list_p() {
    use crate::axioms::predicates::maj_proper_list_p;
    let mut state = MajState::new();
    let a = Maj::symbol(&mut state, "a");
    let b = Maj::symbol(&mut state, "b");
    let c = Maj::symbol(&mut state, "c");
    let d = Maj::symbol(&mut state, "d");
    multi_boolean_test!(
        (maj_proper_list_p(
            maj_list!(a.clone(),
                      b.clone(),
                      c.clone(),
                      d.clone())),
         true);
        (maj_proper_list_p(
            maj_dotted_list!(a, b, c, d)),
         false);
    );
}

#[test]
fn predicates_stringp() {
    use crate::axioms::predicates::maj_stringp;
    use crate::axioms::primitives::maj_vector;
    let mut state = MajState::new();
    let a = Maj::character('a');
    let b = Maj::character('b');
    let c = Maj::character('c');
    let charlist = maj_list!(a.clone(), b.clone(), c.clone());
    let charvec = maj_vector(&mut state, charlist.clone());
    let charvec2 = Maj::vector_any(vec![a, b, c]);
    multi_boolean_test!(
        (maj_stringp(Maj::string("abc")), true);
        (maj_stringp(charvec), true);
        (maj_stringp(charlist), false);
        (maj_stringp(charvec2), false);
        (maj_stringp(Maj::t()), false);
    );  
}

#[test]
fn predicates_literalp() {
    use crate::axioms::predicates::maj_literalp;
    let mut state = MajState::new();
    let x = Maj::symbol(&mut state, "x");
    let a = Maj::symbol(&mut state, "a");
    let b = Maj::symbol(&mut state, "b");
    let c = Maj::symbol(&mut state, "c");
    multi_boolean_test!(
        (maj_literalp(
            // (lit closure nil (x) ((* x x)))
            maj_list!(Maj::lit(),
                      Maj::closure(),
                      Maj::nil(),
                      maj_list!(x.clone()),
                      maj_list!(maj_list!(
                          Maj::symbol(&mut state, "*"),
                          x.clone(), x)))),
         true);
        (maj_literalp(maj_list!(a, b, c)), false);
        (maj_literalp(maj_list!(
            Maj::lit(),
            Maj::symbol(&mut state, "blah"))),
         true);       
    );
}

#[test]
fn predicates_primitivep() {
    use crate::axioms::predicates::maj_primitivep;
    let mut state = MajState::new();
    let x = Maj::symbol(&mut state, "x");
    // Emulate internal definitions of primitivep and +
    // (lit prim primitivep required 1)
    let primitivep = maj_list!(
        Maj::lit(),
        Maj::prim(),
        Maj::symbol(&mut state, "primitivep"),
        Maj::symbol(&mut state, "required"),
        Maj::integer(1));
    // (lit prim + variadic 0)
    let plus = maj_list!(
        Maj::lit(),
        Maj::prim(),
        Maj::symbol(&mut state, "+"),
        Maj::symbol(&mut state, "variadic"),
        Maj::integer(0));
    // Emulate closure definition on top-level
    // (lit closure nil (x) ((+ x x)))
    let closure = maj_list!(
        Maj::lit(),
        Maj::closure(),
        Maj::nil(),
        maj_list!(x.clone()),
        maj_list!(maj_list!(
            Maj::symbol(&mut state, "+"),
            x.clone(), x)));
    multi_boolean_test!(
        (maj_primitivep(primitivep), true);
        (maj_primitivep(closure), false);
        (maj_primitivep(plus), true);
    );
}

#[test]
fn predicates_closurep() {
    use crate::axioms::predicates::maj_closurep;
    let mut state = MajState::new();
    let x = Maj::symbol(&mut state, "x");
    // Emulate closure definition on top-level
    // (lit closure nil (x) ((* x x)))
    let closure = maj_list!(
        Maj::lit(),
        Maj::closure(),
        Maj::nil(),
        maj_list!(x.clone()),
        maj_list!(maj_list!(
            Maj::symbol(&mut state, "*"),
            x.clone(), x)));
    // Emulate internal definitions of cons and +
    // (lit prim cons required 2)
    let cons = maj_list!(
        Maj::lit(),
        Maj::prim(),
        Maj::symbol(&mut state, "cons"),
        Maj::symbol(&mut state, "required"),
        Maj::integer(2));
    // (lit prim + variadic 0)
    let plus = maj_list!(
        Maj::lit(),
        Maj::prim(),
        Maj::symbol(&mut state, "+"),
        Maj::symbol(&mut state, "variadic"),
        Maj::integer(0));
    multi_boolean_test!(
        (maj_closurep(closure), true);
        (maj_closurep(
            maj_list!(Maj::lit(),
                      Maj::symbol(&mut state, "blah"))),
         false);
        (maj_closurep(cons), false);
        (maj_closurep(plus), false);
    );
}

#[test]
fn predicates_functionp() {
    use crate::axioms::predicates::maj_functionp;
    let mut state = MajState::new();
    let x = Maj::symbol(&mut state, "x");
    // Emulate closure definition on top-level
    // (lit closure nil (x) ((* x x)))
    let closure = maj_list!(
        Maj::lit(),
        Maj::closure(),
        Maj::nil(),
        maj_list!(x.clone()),
        maj_list!(maj_list!(
            Maj::symbol(&mut state, "*"),
            x.clone(), x)));
    // Emulate internal definitions of cons and +
    // (lit prim cons required 2)
    let cons = maj_list!(
        Maj::lit(),
        Maj::prim(),
        Maj::symbol(&mut state, "cons"),
        Maj::symbol(&mut state, "required"),
        Maj::integer(2));
    // (lit prim + variadic 0)
    let plus = maj_list!(
        Maj::lit(),
        Maj::prim(),
        Maj::symbol(&mut state, "+"),
        Maj::symbol(&mut state, "variadic"),
        Maj::integer(0));
    multi_boolean_test!(
        (maj_functionp(closure), true);
        (maj_functionp(plus), true);
        (maj_functionp(cons), true);
        (maj_functionp(maj_list!(
            Maj::lit(),
            Maj::symbol(&mut state, "blah"))),
         false);
    );
}

#[test]
fn predicates_macrop() {
    use crate::axioms::predicates::maj_macrop;
    let mut state = MajState::new();
    
    let x         = Maj::symbol(&mut state, "x");
    // Emulate definition of (mac (x) `(list ,x)) on top level
    // (lit macro (lit closure nil (x) ((quasiquote (list (unquote x))))))
    let themacro = maj_list!(
        Maj::lit(),
        Maj::symbol(&mut state, "macro"),
        maj_list!(
            Maj::lit(),
            Maj::closure(),
            Maj::nil(),
            maj_list!(x.clone()),
            maj_list!(
                maj_list!(
                    Maj::quasiquote(),
                    maj_list!(
                        Maj::symbol(&mut state, "list"),
                        maj_list!(
                            Maj::unquote(),
                            x.clone()))))));
    // Emulate internal definitions of cons and +
    // (lit prim cons required 2)
    let cons = maj_list!(
        Maj::lit(),
        Maj::prim(),
        Maj::symbol(&mut state, "cons"),
        Maj::symbol(&mut state, "required"),
        Maj::integer(2));
    // (lit prim + variadic 0)
    let plus = maj_list!(
        Maj::lit(),
        Maj::prim(),
        Maj::symbol(&mut state, "+"),
        Maj::symbol(&mut state, "variadic"),
        Maj::integer(0));
    multi_boolean_test!(
        (maj_macrop(themacro), true);
        (maj_macrop(maj_list!(Maj::lit(),
                              Maj::symbol(&mut state, "blah"))),
         false);
        (maj_macrop(cons), false);
        (maj_macrop(plus), false);
    );
}

#[test]
fn predicates_errorp() {
    use crate::axioms::predicates::maj_errorp;
    use crate::axioms::primitives::maj_err;
    multi_boolean_test!(
        (maj_errorp(maj_err(Maj::string("Some error"),
                            Maj::nil())),
         true);
        (maj_errorp(maj_err(Maj::string("Some other error"),
                            Maj::nil())),
         true);
        (maj_errorp(Maj::integer(2)), false);
    );
}

#[test]
fn predicates_zerop() {
    use crate::axioms::predicates::maj_zerop;
    let mut state = MajState::new();
    multi_boolean_test!(
        (maj_zerop(&mut state, Maj::nil(), Maj::float(0.0)), true);
        (maj_zerop(&mut state, Maj::nil(), Maj::fraction(0, 1)), true);
        (maj_zerop(&mut state, Maj::nil(), Maj::integer(0)), true);
        (maj_zerop(&mut state, Maj::nil(), Maj::integer(5)), false);
    );
    test_fail!(
        maj_zerop(&mut state, Maj::nil(), Maj::t()));
}

#[test]
fn primitives_cons() {
    use crate::axioms::primitives::{ maj_cons, maj_err };
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_cons(Maj::symbol(&mut state, "a"),
                  Maj::symbol(&mut state, "b")),
         "(a . b)");
        (maj_cons(Maj::integer(1), Maj::integer(2)),
         "(1 . 2)");
    );
    multi_fail_test!(
        maj_cons(maj_err(Maj::string("Some error"),
                         Maj::nil()),
                 Maj::t());
    );
}

#[test]
fn primitives_car() {
    use crate::axioms::primitives::{
        maj_cons
    };
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_car(Maj::cons(Maj::symbol(&mut state, "a"),
                           Maj::symbol(&mut state, "b"))),
         "a");
        (maj_car(maj_cons(Maj::symbol(&mut state, "c"),
                          Maj::symbol(&mut state, "d"))),
         "c");
        (maj_car(Maj::nil()), "nil");
    );
}

#[test]
fn primitives_cdr() {
    use crate::axioms::primitives::{
        maj_cdr, maj_cons
    };
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_cdr(Maj::cons(Maj::symbol(&mut state, "a"),
                           Maj::symbol(&mut state, "b"))),
         "b");
        (maj_cdr(maj_cons(Maj::symbol(&mut state, "c"),
                          Maj::symbol(&mut state, "d"))),
         "d");
        (maj_cdr(Maj::nil()), "nil");
    );
}

#[test]
fn primitives_copy() {
    use crate::axioms::{
        primitives::{
            maj_copy, maj_cons, maj_cdr
        },
        predicates::maj_id
    };
    let mut state = MajState::new();
    let x = maj_cons(Maj::symbol(&mut state, "a"),
                     Maj::symbol(&mut state, "b"));
    let y = maj_copy(x.clone());

    multi_test!(
        state;
        (x.clone(), "(a . b)");
        (y.clone(), "(a . b)");
    );
    multi_boolean_test!(
        (maj_id(x.clone(), y.clone()), false);
        (maj_id(maj_car(x.clone()), maj_car(y.clone())),
         true);
        (maj_id(maj_cdr(x.clone()), maj_cdr(y.clone())),
         true);
    );
    test_fail!(maj_copy(Maj::t()));
}

#[test]
fn primitives_length() {
    use crate::axioms::primitives::maj_length;
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_length(maj_list!(Maj::integer(1),
                              Maj::integer(2),
                              Maj::integer(3),
                              Maj::integer(4))),
         "4");
        (maj_length(maj_list!(Maj::symbol(&mut state, "a"),
                              Maj::symbol(&mut state, "b"),
                              Maj::symbol(&mut state, "c"))),
         "3");
        (maj_length(Maj::nil()), "0");
        (maj_length(maj_dotted_list!(Maj::integer(1),
                                     Maj::integer(2),
                                     Maj::integer(3))),
         "2");
    );
    test_fail!(maj_length(Maj::symbol(&mut state, "a")));
}

#[test]
fn primitives_depth() {
    use crate::axioms::primitives::maj_depth;
    let state = MajState::new();
    multi_test!(
        state;
        (maj_depth(Maj::nil()), "0");
        (maj_depth(maj_list!(Maj::nil())), "1");
        (maj_depth(maj_list!(maj_list!(Maj::integer(1)))),
         "2");
        (maj_depth(maj_list!(Maj::integer(1),
                             Maj::integer(2),
                             Maj::integer(3),
                             Maj::integer(4),
                             Maj::integer(5))),
         "5");
        (maj_depth(maj_list!(Maj::integer(1),
                             Maj::integer(2),
                             Maj::integer(3),
                             Maj::integer(4),
                             Maj::integer(5),
                             Maj::integer(6))),
         "6");
        (maj_depth(maj_dotted_list!(Maj::integer(1),
                                    Maj::integer(2),
                                    Maj::integer(3),
                                    Maj::integer(4),
                                    Maj::integer(5),
                                    Maj::integer(6))),
         "5");
    );
    test_fail!(maj_depth(Maj::integer(1)));
}

#[test]
fn primitives_type() {
    use crate::axioms::primitives::maj_type;
    let mut state = MajState::new();
    let a = Maj::symbol(&mut state, "a");
    let b = Maj::symbol(&mut state, "b");
    let c = Maj::symbol(&mut state, "c");
    let stdout = state.make_stream_stdout();
    multi_test!(
        state;
        (maj_type(&mut state, Maj::integer(1)),
         "integer");
        (maj_type(&mut state, Maj::float(2.0)),
         "float");
        (maj_type(&mut state, a.clone()),
         "symbol");
        (maj_type(&mut state, Maj::nil()),
         "symbol");
        (maj_type(&mut state, Maj::fraction(2, 3)),
         "fraction");
        (maj_type(&mut state, maj_list!(a, b, c)),
         "cons");
        (maj_type(&mut state, Maj::character('L')),
         "char");
        (maj_type(&mut state, stdout), "stream");
        (maj_type(&mut state, Maj::complex(Maj::integer(1),
                                           Maj::integer(5))),
         "complex");
        (maj_type(&mut state, Maj::vector_integer(vec![1, 2, 3])),
         "vector");
    );
}

#[test]
fn primitives_intern() {
    use crate::axioms::primitives::maj_intern;
    let mut state = MajState::new();
    let a = Maj::symbol(&mut state, "a");
    multi_test!(
        state;
        (maj_intern(&mut state, Maj::string("blah")),
         "blah");
        (maj_intern(&mut state, Maj::string("foo")),
         "foo");
        (maj_intern(&mut state, Maj::string("")),
         "nil");
    );
    test_fail!(maj_intern(&mut state, a));
}

#[test]
fn primitives_name() {
    use crate::axioms::primitives::maj_name;
    let mut state = MajState::new();
    let foo = Maj::symbol(&mut state, "foo");
    let thing = Maj::symbol(&mut state, "thing");
    multi_test!(
        state;
        (maj_name(&state, Maj::t()), "\"t\"");
        (maj_name(&state, foo), "\"foo\"");
        (maj_name(&state, thing), "\"thing\"");
        (maj_name(&state, Maj::nil()), "\"nil\"");
    );
    test_fail!(maj_name(&state, Maj::string("Blah")));
}

#[test]
#[ignore]
fn primitives_get_environment() {
    use crate::core::environment::maj_env_push;
    use crate::axioms::primitives::maj_get_environment;
    let mut state = MajState::new();
    let x = Maj::symbol(&mut state, "x");
    let y = Maj::symbol(&mut state, "y");
    let plus = Maj::symbol(&mut state, "+");
    let sum_test = Maj::symbol(&mut state, "sum-test");
    let global = Maj::symbol(&mut state, "global");
    let lexical = Maj::symbol(&mut state, "lexical");
    let blah = Maj::symbol(&mut state, "blah");
    let mut env = Maj::nil();
    env = maj_env_push(env, x.clone(), Maj::integer(5));
    env = maj_env_push(env, y.clone(), Maj::integer(6));
    multi_regex_test!(
        state;
        (maj_get_environment(&mut state, global, env.clone()),
         RE_ENVIRONMENT);
        (maj_get_environment(&mut state, lexical.clone(), env.clone()),
         RE_ENVIRONMENT);
    );
    test_fail!(maj_get_environment(&mut state, blah, env.clone()));
    test_format!(
        state,
        {
            let lexenv = maj_get_environment(
                &mut state, lexical, env.clone());
            let newenv = maj_env_push(
                env.clone(),
                Maj::symbol(&mut state, "env"),
                lexenv);
            state.push(sum_test.clone(),
                       maj_list!(
                           maj_list!(
                               Maj::quasiquote(),
                               maj_list!(
                                   Maj::lit(),
                                   Maj::closure(),
                                   newenv.clone(),
                                   Maj::nil(),
                                   maj_list!(
                                       maj_list!(
                                           plus, x, y))))));
            sum_test.clone() // TODO: Replace with lookup
        }, "sum-test"); // TODO: Return a closure here and check
    // TODO: Add application (sum-test)
}

#[test]
fn primitives_coin() {
    use crate::axioms::primitives::maj_coin;
    let state = MajState::new();
    // No better way to test this... just toss a coin
    // for a few times and check whether the output
    // is either t or nil.
    // In the future, we might want to test the distribution
    // of these values.
    let n = 100;
    for _ in 0..n {
        test_regex!(state, maj_coin(), r"^t|nil$");
    }
}

#[test]
fn primitives_sys() {
    use crate::axioms::primitives::maj_sys;
    let state = MajState::new();
    multi_test!(
        state;
        (maj_sys(Maj::string("true"), Maj::nil()),  "0");
        (maj_sys(Maj::string("false"), Maj::nil()), "1");
    );
}

#[test]
fn primitives_format() {
    use crate::axioms::primitives::{
        maj_format_prim,
        maj_type
    };
    let mut state = MajState::new();
    let x = Maj::fraction(1, 2);
    let xtype = maj_type(&mut state, x.clone());
    multi_test!(
        state;
        (maj_format_prim(&state, Maj::string("Hello world"),
                         Maj::nil()),
         "\"Hello world\"");
        (maj_format_prim(&state, Maj::string("The number five: {}"),
                         maj_list!(Maj::integer(5))),
         "\"The number five: 5\"");
        (maj_format_prim(
            &state, Maj::string("The floating point {} is nice"),
            maj_list!(Maj::float(2.0))),
         "\"The floating point 2.0 is nice\"");
        (maj_format_prim(
            &state, Maj::string("The number {} has a subtype {}"),
            maj_list!(x, xtype)),
         "\"The number 1/2 has a subtype fraction\"");
    );
    multi_fail_test!(
        maj_format_prim(&state, Maj::string("Hello {}"),
                        Maj::nil());
        maj_format_prim(&state, Maj::string("Hello {"),
                        maj_list!(Maj::string("World")));
        maj_format_prim(&state, Maj::string("Hello }"),
                        maj_list!(Maj::string("World")));
    );
}

#[test]
fn primitives_err() {
    use crate::axioms::primitives::maj_err;
    // Should any error creation fail here, the
    // function will panic.
    multi_fail_test!(
        maj_err(Maj::string("{} is not a number"),
                maj_list!(Maj::t()));
        maj_err(
            Maj::string("This is an error, numbers are {} and {}"),
            maj_list!(Maj::integer(2), Maj::integer(3)));
    );
}

#[test]
#[ignore]
fn primitives_warn() {
    unimplemented!();
}

#[test]
fn primitives_list() {
    use crate::axioms::primitives::maj_list;
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_list(maj_list!(Maj::integer(1),
                            Maj::integer(2),
                            Maj::integer(3))),
         "(1 2 3)");
        (maj_list(maj_list!(
            Maj::symbol(&mut state, "a"),
            Maj::integer(6),
            Maj::symbol(&mut state, "b"))),
         "(a 6 b)");
        (maj_list(Maj::nil()), "nil");
    );
}

#[test]
fn primitives_append() {
    use crate::axioms::primitives::maj_append;
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_append(maj_list!(
            maj_list!(Maj::integer(1),
                      Maj::integer(2),
                      Maj::integer(3)),
            maj_list!(Maj::integer(4),
                      Maj::integer(5),
                      Maj::integer(6)))),
         "(1 2 3 4 5 6)");
        (maj_append(maj_list!(
            maj_list!(
                Maj::symbol(&mut state, "a"),
                maj_list!(
                    Maj::symbol(&mut state, "b"),
                    Maj::symbol(&mut state, "c")),
                Maj::symbol(&mut state, "d"),
                maj_list!(
                    Maj::symbol(&mut state, "e"))),
            maj_list!(
                Maj::symbol(&mut state, "f"),
                maj_list!(
                    Maj::symbol(&mut state, "g"),
                    Maj::symbol(&mut state, "h"))))),
         "(a (b c) d (e) f (g h))");
        (maj_append(maj_list!(
            maj_list!(
                Maj::symbol(&mut state, "a"),
                Maj::symbol(&mut state, "b"),
                Maj::symbol(&mut state, "c")),
            Maj::symbol(&mut state, "d"))),
         "(a b c . d)");
    );
}

#[test]
fn primitives_last() {
    use crate::axioms::primitives::maj_last;
    let mut state = MajState::new();
    let a = Maj::symbol(&mut state, "a");
    let b = Maj::symbol(&mut state, "b");
    let c = Maj::symbol(&mut state, "c");
    let d = Maj::symbol(&mut state, "d");
    multi_test!(
        state;
        (maj_last(maj_list!(a.clone(),
                            b.clone(),
                            c.clone(),
                            d.clone())),
         "(d)");
        (maj_last(maj_dotted_list!(
            a.clone(), b, c, d)),
         "(c . d)");
    );
    test_fail!(maj_last(a));
}

#[test]
fn primitives_reverse() {
    use crate::axioms::primitives::maj_reverse;
    let mut state = MajState::new();
    let a = Maj::symbol(&mut state, "a");
    let b = Maj::symbol(&mut state, "b");
    let c = Maj::symbol(&mut state, "c");
    let d = Maj::symbol(&mut state, "d");
    test_format!(
        state,
        maj_reverse(maj_list!(
            a.clone(), b.clone(), c.clone(), d.clone())),
        "(d c b a)");
    multi_fail_test!(
        maj_reverse(a.clone());
        maj_reverse(maj_dotted_list!(a, b, c, d));
    );
}

#[test]
fn primitives_nthcdr() {
    use crate::axioms::primitives::maj_nthcdr;
    let state = MajState::new();
    let list = maj_list!(Maj::integer(1),
                         Maj::integer(2),
                         Maj::integer(3),
                         Maj::integer(4));
    let dlist = maj_dotted_list!(Maj::integer(1),
                                 Maj::integer(2),
                                 Maj::integer(3),
                                 Maj::integer(4));
    multi_test!(
        state;
        (maj_nthcdr(Maj::integer(0), list.clone()),
         "(1 2 3 4)");
        (maj_nthcdr(Maj::integer(1), list.clone()),
         "(2 3 4)");
        (maj_nthcdr(Maj::integer(3), list.clone()),
         "(4)");
    );
    multi_fail_test!(
        maj_nthcdr(Maj::integer(3), dlist);
        maj_nthcdr(Maj::integer(-1), list.clone());
        maj_nthcdr(Maj::fraction(3, 4), list.clone());
    );
    multi_test!(
        state;
        (maj_nthcdr(Maj::integer(4), list.clone()),
         "nil");
        (maj_nthcdr(Maj::integer(5), list), "nil");
        (maj_nthcdr(Maj::integer(50), Maj::nil()), "nil");
    );
}

#[test]
fn primitives_nth() {
    use crate::axioms::primitives::maj_nth;
    let state = MajState::new();
    let list = maj_list!(Maj::integer(1),
                         Maj::integer(2),
                         Maj::integer(3),
                         Maj::integer(4));
    let dlist = maj_dotted_list!(Maj::integer(1),
                                 Maj::integer(2),
                                 Maj::integer(3),
                                 Maj::integer(4));
    multi_test!(
        state;
        (maj_nth(Maj::integer(0), list.clone()),
         "1");
        (maj_nth(Maj::integer(3), list.clone()),
         "4");
    );
    multi_fail_test!(
        maj_nth(Maj::integer(3), dlist);
        maj_nth(Maj::integer(-1), list.clone());
        maj_nth(Maj::fraction(3, 4), list.clone());
    );
    multi_test!(
        state;
        (maj_nth(Maj::integer(4), list.clone()),
         "nil");
        (maj_nth(Maj::integer(5), list), "nil");
        (maj_nth(Maj::integer(50), Maj::nil()), "nil");
    );
}

#[test]
#[ignore]
fn primitives_macroexpand_1() {
    unimplemented!();
}

#[test]
#[ignore]
fn primitives_macroexpand() {
    unimplemented!();
}

#[test]
fn primitives_not() {
    use crate::axioms::primitives::maj_not;
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_not(Maj::integer(1)), "nil");
        (maj_not(maj_list!(Maj::symbol(&mut state, "a"),
                           Maj::symbol(&mut state, "b"),
                           Maj::symbol(&mut state, "c"))),
         "nil");
        (maj_not(Maj::t()), "nil");
        (maj_not(Maj::nil()), "t");
    );
}

#[test]
fn primitives_gensym() {
    use crate::axioms::primitives::maj_gensym;
    // Generate a few symbols and that's it.
    let mut state = MajState::new();
    let n = 100;
    for _ in 0..n {
        test_regex!(state, maj_gensym(&mut state), r"^:G[0-9]*$");
    }
}

#[test]
fn primitives_vector_vec_type() {
    use crate::axioms::primitives::{
        maj_vector,
        maj_vec_type
    };
    let mut state = MajState::new();
    let v1 = maj_vector(&mut state, Maj::nil());
    let v2 = maj_vector(&mut state, maj_list!(Maj::integer(1),
                                              Maj::integer(2),
                                              Maj::integer(3),
                                              Maj::integer(4)));
    let v3 = maj_vector(&mut state, maj_list!(Maj::float(1.0),
                                              Maj::float(3.0),
                                              Maj::float(2.5)));
    let v4 = maj_vector(&mut state, maj_list!(Maj::character('H'),
                                              Maj::character('e'),
                                              Maj::character('l'),
                                              Maj::character('l'),
                                              Maj::character('o')));
    let blah = Maj::symbol(&mut state, "blah");
    let v5 = maj_vector(&mut state, maj_list!(Maj::float(1.0),
                                              Maj::integer(3),
                                              Maj::fraction(2, 5),
                                              Maj::complex(
                                                  Maj::integer(2),
                                                  Maj::integer(6)),
                                              blah));
    multi_test!(
        state;
        (v1.clone(), "[]");
        (maj_vec_type(&mut state, v1), "any");
        (v2.clone(), "[1 2 3 4]");
        (maj_vec_type(&mut state, v2), "integer");
        (v3.clone(), "[1.0 3.0 2.5]");
        (maj_vec_type(&mut state, v3), "float");
        (v4.clone(), "\"Hello\"");
        (maj_vec_type(&mut state, v4), "char");
        (v5.clone(), "[1.0 3 2/5 2J6 blah]");
        (maj_vec_type(&mut state, v5), "any");
    );
}

#[test]
fn primitive_vec_coerce() {
    use crate::axioms::primitives::{
        maj_vector,
        maj_vec_push,
        maj_vec_type,
        maj_vec_coerce
    };
    let mut state = MajState::new();
    let integer = Maj::symbol(&mut state, "integer");
    let any = Maj::symbol(&mut state, "any");
    let v = maj_vector(&mut state, Maj::nil());
    test_format!(state, maj_vec_type(&mut state, v.clone()), "any");
    
    let _ = maj_vec_push(&mut state, v.clone(), Maj::integer(2));
    let _ = maj_vec_push(&mut state, v.clone(), Maj::integer(3));
    test_format!(state, maj_vec_type(&mut state, v.clone()), "any");
    
    let v2 = maj_vec_coerce(&mut state, integer, v);
    test_format!(state, maj_vec_type(&mut state, v2), "integer");

    test_format!(state, maj_vec_coerce(&mut state, any, Maj::string("Hello")),
                 "[#\\H #\\e #\\l #\\l #\\o]");
}

#[test]
fn primitives_vec_push() {
    use crate::axioms::primitives::maj_vec_push;
    let mut state = MajState::new();
    
    let v = Maj::vector_integer(vec![]);
    multi_test!(
        state;
        (v.clone(), "[]");
        (maj_vec_push(&mut state, Maj::integer(5), v.clone()), "[5]");
        (maj_vec_push(&mut state, Maj::integer(6), v.clone()), "[5 6]");
    );

    let a = Maj::symbol(&mut state, "a");
    test_fail!(maj_vec_push(&mut state, a.clone(), v.clone()));

    let myvec = Maj::vector_any(vec![]);
    let elements = vec![
        Maj::integer(5), Maj::integer(6), a,
        Maj::fraction(2, 3)
    ];
    for elt in &elements {
        test_dont_fail!(
            maj_vec_push(&mut state, elt.clone(), myvec.clone()));
    }
    test_format!(state, myvec, "[5 6 a 2/3]");
}

#[test]
fn primitives_vec_set() {
    use crate::axioms::primitives::maj_vec_set;
    let state = MajState::new();
    let v = Maj::vector_integer(vec![1, 2, 3, 4, 5]);
    multi_test!(
        state;
        (maj_vec_set(Maj::integer(2), Maj::integer(50), v.clone()),
         "[1 2 50 4 5]");
        (maj_vec_set(Maj::integer(4), Maj::integer(10), v.clone()),
         "[1 2 50 4 10]");
    );
    test_fail!(
        maj_vec_set(Maj::integer(5), Maj::integer(90), v.clone()));
}

#[test]
fn primitives_vec_pop() {
    use crate::axioms::primitives::maj_vec_pop;
    let state = MajState::new();
    let v = Maj::vector_integer(vec![1, 2, 3]);
    multi_test!(
        state;
        (v.clone(), "[1 2 3]");
        (maj_vec_pop(v.clone()), "3");
        (maj_vec_pop(v.clone()), "2");
        (maj_vec_pop(v.clone()), "1");
        (maj_vec_pop(v.clone()), "nil");
        (v, "[]");
    );
    test_fail!(maj_vec_pop(Maj::fraction(2, 3)));
}

#[test]
fn primitives_vec_deq() {
    use crate::axioms::primitives::maj_vec_deq;
    let state = MajState::new();
    let v = Maj::vector_integer(vec![0, 1, 2]);
    multi_test!(
        state;
        (v.clone(), "[0 1 2]");
        (maj_vec_deq(v.clone()), "0");
        (maj_vec_deq(v.clone()), "1");
        (maj_vec_deq(v.clone()), "2");
        (maj_vec_deq(v.clone()), "nil");
        (v, "[]");
    );
    test_fail!(maj_vec_deq(Maj::fraction(2, 3)));
}

#[test]
fn primitives_vec_length() {
    use crate::axioms::primitives::{
        maj_vec_length,
        maj_iota,
        maj_reverse,
        maj_vector
    };
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_vec_length(Maj::vector_any(vec![])), "0");
        (maj_vec_length(Maj::vector_integer(vec![1, 2, 3, 4])), "4");
        ({
            let iota50 = maj_iota(Maj::integer(50));
            let reviota = maj_reverse(iota50);
            let v = maj_vector(&mut state, reviota);
            maj_vec_length(v)
        }, "50");
    );
    test_fail!(maj_vec_length(state.make_stream_stdin()));
}

#[test]
fn primitives_vec_at() {
    use crate::axioms::primitives::maj_vec_at;
    let state = MajState::new();
    multi_test!(
        state;
        (maj_vec_at(Maj::integer(1),
                    Maj::vector_integer(vec![1, 2, 3])),
         "2");
        (maj_vec_at(Maj::integer(0), Maj::string("Hello")),
         "#\\H");
    );
    multi_fail_test!(
        maj_vec_at(Maj::integer(5), Maj::string("Hello"));
        maj_vec_at(Maj::integer(5),
                   maj_list!(Maj::integer(1),
                             Maj::integer(2),
                             Maj::integer(3)));
    );
}

#[test]
fn primitives_vec_remove() {
    use crate::axioms::primitives::maj_vec_remove;
    let vec = Maj::vector_integer(vec![1, 2, 3, 4, 5]);
    let state = MajState::new();
    multi_test!(
        state;
        (maj_vec_remove(Maj::integer(2), vec.clone()), "3");
        (maj_vec_remove(Maj::integer(2), vec.clone()), "4");
        (vec.clone(), "[1 2 5]");
        (maj_vec_remove(Maj::integer(2), vec.clone()), "5");
    );
    multi_fail_test!(
        maj_vec_remove(Maj::integer(2), vec);
        maj_vec_remove(Maj::integer(0), Maj::integer(2));
    );
}

#[test]
fn primitives_vec_insert() {
    use crate::axioms::primitives::maj_vec_insert;
    let mut state = MajState::new();
    let v = Maj::vector_integer(vec![1, 2, 3]);
    
    multi_test!(
        state;
        (v.clone(), "[1 2 3]");
        (maj_vec_insert(
            &mut state, Maj::integer(3), Maj::integer(5), v.clone()),
         "[1 2 3 5]");
        (maj_vec_insert(
            &mut state, Maj::integer(0), Maj::integer(5), v.clone()),
         "[5 1 2 3 5]");
        (maj_vec_insert(
            &mut state, Maj::integer(0), Maj::integer(5), v.clone()),
         "[5 5 1 2 3 5]");
        (maj_vec_insert(
            &mut state, Maj::integer(0), Maj::integer(5), v.clone()),
         "[5 5 5 1 2 3 5]");
        (maj_vec_insert(
            &mut state, Maj::integer(1), Maj::integer(7), v.clone()),
         "[5 7 5 5 1 2 3 5]");
        (maj_vec_insert(
            &mut state, Maj::integer(3), Maj::integer(8), v.clone()),
         "[5 7 5 8 5 1 2 3 5]");
    );

    let a = Maj::symbol(&mut state, "a");
    multi_fail_test!(
        maj_vec_insert(&mut state,
                       Maj::integer(20),
                       Maj::integer(8),
                       Maj::vector_integer(vec![1, 2, 3]));
        maj_vec_insert(&mut state,
                       Maj::integer(2),
                       a,
                       Maj::vector_integer(vec![1, 2, 3]));
    );
}

#[test]
fn primitives_number_coerce() {
    use crate::axioms::primitives::maj_number_coerce;
    let mut state = MajState::new();

    let float = Maj::symbol(&mut state, "float");
    let integer = Maj::symbol(&mut state, "integer");

    multi_test!(
        state;
        (maj_number_coerce(
            &mut state, float.clone(), Maj::integer(1)), "1.0");
        (maj_number_coerce(
            &mut state, integer.clone(), Maj::complex(
                Maj::integer(1), Maj::integer(3))), "1");
        (maj_number_coerce(
            &mut state, integer.clone(), Maj::float(2.3)), "2");
        (maj_number_coerce(
            &mut state, float, Maj::fraction(2, 5)), "0.4");
    );
    test_fail!(maj_number_coerce(&mut state, integer, Maj::t()));
}

#[test]
fn primitives_real_part_imag_part() {
    use crate::axioms::primitives::{
        maj_real_part,
        maj_imag_part
    };
    let state = MajState::new();
    multi_test!(
        state;
        (maj_real_part(Maj::complex(Maj::integer(2),
                                    Maj::integer(5))), "2");
        (maj_real_part(Maj::complex(Maj::fraction(3, 2),
                                    Maj::integer(7))), "3/2");
        (maj_imag_part(Maj::complex(Maj::integer(2),
                                    Maj::integer(5))), "5");
        (maj_imag_part(Maj::complex(Maj::integer(7),
                                    Maj::fraction(3, 2))), "3/2");
    );
    multi_fail_test!(
        maj_real_part(Maj::fraction(1, 2));
        maj_real_part(Maj::nil());
        maj_imag_part(Maj::fraction(1, 2));
        maj_imag_part(Maj::nil());
    );
}

#[test]
fn primitives_numer_denom() {
    use crate::axioms::primitives::{
        maj_numer,
        maj_denom
    };
    let state = MajState::new();
    multi_test!(
        state;
        (maj_numer(Maj::fraction(1, 2)), "1");
        (maj_numer(Maj::fraction(3, 2)), "3");
        (maj_denom(Maj::fraction(1, 2)), "2");
        (maj_denom(Maj::fraction(3, 2)), "2");
    );
    multi_fail_test!(
        maj_numer(Maj::integer(2));
        maj_numer(Maj::nil());
        maj_denom(Maj::integer(2));
        maj_denom(Maj::nil());
    );
}

#[test]
fn primitives_richest_number_type() {
    use crate::axioms::primitives::maj_richest_number_type;
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_richest_number_type(
            &mut state, Maj::integer(2), Maj::float(2.0)),
         "float");
        (maj_richest_number_type(
            &mut state, Maj::integer(2), Maj::fraction(2, 3)),
         "fraction");
        (maj_richest_number_type(
            &mut state, Maj::complex(Maj::integer(2),
                                     Maj::integer(3)),
            Maj::integer(9)),
         "complex");
        (maj_richest_number_type(
            &mut state, Maj::complex(Maj::fraction(2, 3),
                                     Maj::integer(-9)),
            Maj::float(2.0)),
         "complex");
    );
    test_fail!(maj_richest_number_type(
        &mut state, Maj::complex(Maj::fraction(2, 3),
                                 Maj::integer(-9)),
        Maj::t()));
}

#[test]
fn primitives_rich_number_coerce() {
    use crate::axioms::primitives::maj_rich_number_coerce;
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_rich_number_coerce(&mut state,
                                Maj::complex(Maj::integer(2),
                                             Maj::integer(3)),
                                Maj::fraction(2, 5)),
         "(2J3 2/5J0.0)");
        (maj_rich_number_coerce(&mut state,
                                Maj::float(1.5),
                                Maj::fraction(5, 2)),
         "(3/2 5/2)");
    );
    test_fail!(maj_rich_number_coerce(
        &mut state, Maj::integer(2), Maj::t()));
}

#[test]
fn primitives_arithm_plus() {
    use crate::axioms::primitives::maj_arithm_plus;
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_arithm_plus(&mut state, Maj::nil(), Maj::nil()), "1");
        (maj_arithm_plus(
            &mut state, Maj::nil(),
            maj_list!(Maj::complex(Maj::integer(2),
                                   Maj::integer(3)))), "2J-3");
        (maj_arithm_plus(
            &mut state, Maj::nil(),
            maj_list!(Maj::integer(1),
                      Maj::integer(2),
                      Maj::integer(3))),
         "6");
    );
    let a = Maj::symbol(&mut state, "a");
    test_fail!(maj_arithm_plus(
        &mut state, Maj::nil(), maj_list!(a)));
}

#[test]
fn primitives_arithm_minus() {
    use crate::axioms::primitives::maj_arithm_minus;
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_arithm_minus(&mut state, Maj::nil(), Maj::nil()), "0");
        (maj_arithm_minus(
            &mut state, Maj::nil(),
            maj_list!(Maj::integer(1))), "-1");
        (maj_arithm_minus(
            &mut state, Maj::nil(),
            maj_list!(Maj::integer(1),
                      Maj::integer(2),
                      Maj::integer(3))),
         "-4");
    );
    let a = Maj::symbol(&mut state, "a");
    test_fail!(maj_arithm_minus(
        &mut state, Maj::nil(), maj_list!(a)));
}

#[test]
fn primitives_arithm_times() {
    use crate::axioms::primitives::maj_arithm_times;
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_arithm_times(&mut state, Maj::nil(), Maj::nil()), "1");
        (maj_arithm_times(
            &mut state, Maj::nil(),
            maj_list!(Maj::integer(-50))), "-1");
        (maj_arithm_times(
            &mut state, Maj::nil(),
            maj_list!(Maj::integer(1),
                      Maj::integer(2),
                      Maj::integer(3))),
         "6");
        (maj_arithm_times(
            &mut state, Maj::nil(),
            maj_list!(Maj::complex(Maj::integer(3),
                                   Maj::integer(2)),
                      Maj::complex(Maj::integer(1),
                                   Maj::integer(7)))),
         "-11J23.0");
    );
    let a = Maj::symbol(&mut state, "a");
    test_fail!(maj_arithm_times(
        &mut state, Maj::nil(), maj_list!(a)));
}

#[test]
fn primitives_arithm_divide() {
    use crate::axioms::primitives::maj_arithm_divide;
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_arithm_divide(&mut state, Maj::nil(), Maj::nil()), "1");
        (maj_arithm_divide(
            &mut state, Maj::nil(),
            maj_list!(Maj::integer(3))), "1/3");
        (maj_arithm_divide(
            &mut state, Maj::nil(),
            maj_list!(Maj::float(0.5))), "2");
        (maj_arithm_divide(
            &mut state, Maj::nil(),
            maj_list!(Maj::fraction(1, 2),
                      Maj::complex(
                          Maj::integer(0),
                          Maj::integer(1)))),
         "0.0J-1/2");
        (maj_arithm_divide(
            &mut state, Maj::nil(),
            maj_list!(Maj::fraction(1, 2))), "2");
        (maj_arithm_divide(
            &mut state, Maj::nil(),
            maj_list!(Maj::integer(3),
                      Maj::integer(4))),
         "3/4");
    );
    let a = Maj::symbol(&mut state, "a");
    multi_fail_test!(
        maj_arithm_divide(
            &mut state, Maj::nil(),
            maj_list!(Maj::integer(2),
                      Maj::integer(3),
                      Maj::integer(0)));
        maj_arithm_divide(
            &mut state, Maj::nil(),
            maj_list!(Maj::integer(0)));
        maj_arithm_divide(
            &mut state, Maj::nil(),
            maj_list!(Maj::integer(2), a));
    );
}

#[test]
fn primitives_iota() {
    use crate::axioms::primitives::maj_iota;
    let mut state = MajState::new();
    multi_test!(
        state;
        (maj_iota(Maj::integer(0)), "nil");
        (maj_iota(Maj::integer(5)), "(0 1 2 3 4)");
    );
    multi_fail_test!(
        maj_iota(Maj::integer(-1));
        maj_iota(Maj::fraction(2, 3));
        maj_iota(Maj::symbol(&mut state, "a"));
    );
}

#[test]
fn primitives_arithm_equal() {
    use crate::axioms::primitives::maj_arithm_eq;
    use crate::core::environment::maj_env_push;
    
    let mut state = MajState::new();
    let test_env = maj_env_push(Maj::nil(),
                                Maj::symbol(&mut state, "*ulps*"),
                                Maj::integer(3));
    
    multi_boolean_test!(
        (maj_arithm_eq(&mut state, test_env.clone(),
                       Maj::fraction(2, 1), Maj::integer(2),
                       Maj::nil()), true);
        (maj_arithm_eq(&mut state, test_env.clone(),
                       Maj::integer(1), Maj::integer(1),
                       Maj::nil()), true);
        (maj_arithm_eq(&mut state, test_env.clone(),
                       Maj::complex(Maj::integer(2),
                                    Maj::integer(3)),
                       Maj::fraction(2, 3),
                       Maj::nil()), false);
        (maj_arithm_eq(&mut state, test_env.clone(),
                       Maj::float(2.0), Maj::integer(2),
                       Maj::nil()), true);
        (maj_arithm_eq(&mut state, test_env.clone(),
                       Maj::integer(2), Maj::fraction(4, 2),
                       maj_list!(
                           Maj::integer(2),
                           Maj::integer(3),
                           Maj::integer(5))), false);
    );
    test_fail!(maj_arithm_eq(&mut state, test_env,
                             Maj::t(), Maj::fraction(2, 3),
                             Maj::nil()));
}

#[test]
fn primitives_arithm_float_eq() {
    use crate::axioms::primitives::{
        maj_arithm_floateq,
        maj_arithm_times,
        maj_arithm_plus
    };
    use crate::core::environment::maj_env_push;
    
    let mut state = MajState::new();
    let test_env = maj_env_push(Maj::nil(),
                                Maj::symbol(&mut state, "*ulps*"),
                                Maj::integer(3));

    
    let f = Maj::float(0.1);
    let mut sum = Maj::float(0.0);

    // (repeat 10 (set sum (+ sum f)))
    for _ in 0..10 {
        sum = maj_arithm_plus(&mut state, test_env.clone(),
                              maj_list!(sum.clone(), f.clone()));
    }

    // (def product (* f 10))
    let product =
        maj_arithm_times(&mut state, test_env.clone(),
                         maj_list!(f.clone(), Maj::float(10.0)));
    
    test_boolean!(maj_arithm_floateq(&mut state, test_env.clone(),
                                     sum, product),
                  true);
}

#[test]
fn primitives_arithm_greater() {
    use crate::axioms::primitives::maj_arithm_greater;
    let mut state = MajState::new();

    let one = Maj::integer(1);
    
    multi_boolean_test!(
        (maj_arithm_greater(&mut state,
                            Maj::fraction(5, 2), Maj::integer(2),
                            Maj::nil()), true);
        (maj_arithm_greater(&mut state,
                            Maj::integer(1), one, Maj::nil()),
         false);
        (maj_arithm_greater(&mut state,
                            Maj::integer(10), Maj::integer(5),
                            maj_list!(Maj::integer(3), Maj::integer(1))),
         true);
        (maj_arithm_greater(&mut state,
                            Maj::integer(1), Maj::integer(2),
                            maj_list!(Maj::integer(3))), false);
    );

    multi_fail_test!(
        maj_arithm_greater(&mut state, Maj::complex(Maj::integer(2),
                                                    Maj::integer(3)),
                           Maj::fraction(2, 3), Maj::nil());
        maj_arithm_greater(&mut state, Maj::integer(3), Maj::t(),
                           Maj::nil());
    );
}

#[test]
fn primitives_arithm_lesser() {
    use crate::axioms::primitives::maj_arithm_lesser;
    let mut state = MajState::new();

    let one = Maj::integer(1);
    
    multi_boolean_test!(
        (maj_arithm_lesser(&mut state,
                           Maj::integer(2), Maj::fraction(5, 2),
                           Maj::nil()), true);
        (maj_arithm_lesser(&mut state,
                            Maj::integer(1), one, Maj::nil()),
         false);
        (maj_arithm_lesser(&mut state,
                           Maj::integer(1), Maj::integer(3),
                           maj_list!(Maj::integer(5),
                                     Maj::integer(10))),
         true);
        (maj_arithm_lesser(&mut state,
                           Maj::integer(3), Maj::integer(2),
                           maj_list!(Maj::integer(1))), false);
    );

    multi_fail_test!(
        maj_arithm_lesser(&mut state, Maj::complex(Maj::integer(2),
                                                   Maj::integer(3)),
                          Maj::fraction(2, 3), Maj::nil());
        maj_arithm_lesser(&mut state, Maj::integer(3), Maj::t(),
                          Maj::nil());
    );
}

#[test]
fn primitives_arithm_geq() {
    use crate::axioms::primitives::maj_arithm_geq;
    use crate::core::environment::maj_env_push;
    let mut state = MajState::new();
    let env = maj_env_push(Maj::nil(),
                           Maj::symbol(&mut state, "*ulps*"),
                           Maj::integer(3));
    
    multi_boolean_test!(
        (maj_arithm_geq(&mut state, env.clone(),
                        Maj::fraction(5, 2), Maj::integer(2),
                        Maj::nil()), true);
        (maj_arithm_geq(&mut state, env.clone(),
                        Maj::float(2.0), Maj::integer(2),
                        Maj::nil()), true);
        (maj_arithm_geq(&mut state, env.clone(),
                        Maj::integer(1), Maj::integer(2),
                        maj_list!(Maj::integer(3))), false);
        (maj_arithm_geq(&mut state, env.clone(),
                        Maj::integer(3), Maj::integer(3),
                        maj_list!(Maj::integer(1))), true);
    );
    multi_fail_test!(
        maj_arithm_geq(&mut state, env.clone(),
                       Maj::complex(Maj::integer(5),
                                    Maj::integer(9)),
                       Maj::complex(Maj::integer(6),
                                    Maj::integer(9)),
                       Maj::nil());
        maj_arithm_geq(&mut state, env,
                       Maj::integer(3), Maj::t(), Maj::nil());
    );
}

#[test]
fn primitives_arithm_leq() {
    use crate::axioms::primitives::maj_arithm_leq;
    use crate::core::environment::maj_env_push;
    let mut state = MajState::new();
    let env = maj_env_push(Maj::nil(),
                           Maj::symbol(&mut state, "*ulps*"),
                           Maj::integer(3));
    
    multi_boolean_test!(
        (maj_arithm_leq(&mut state, env.clone(),
                        Maj::fraction(5, 2), Maj::integer(2),
                        Maj::nil()), false);
        (maj_arithm_leq(&mut state, env.clone(),
                        Maj::float(2.0), Maj::integer(2),
                        Maj::nil()), true);
        (maj_arithm_leq(&mut state, env.clone(),
                        Maj::integer(1), Maj::integer(2),
                        maj_list!(Maj::integer(3))), true);
        (maj_arithm_leq(&mut state, env.clone(),
                        Maj::integer(3), Maj::integer(3),
                        maj_list!(Maj::integer(1))), false);
    );
    multi_fail_test!(
        maj_arithm_leq(&mut state, env.clone(),
                       Maj::complex(Maj::integer(5),
                                    Maj::integer(9)),
                       Maj::complex(Maj::integer(6),
                                    Maj::integer(9)),
                       Maj::nil());
        maj_arithm_leq(&mut state, env,
                       Maj::integer(3), Maj::t(), Maj::nil());
    );
}

#[test]
fn primitives_open_stream_close_stream() {
    use crate::axioms::primitives::{
        maj_open_stream,
        maj_close_stream
    };
    let mut state = MajState::new();
    let out_sym = Maj::symbol(&mut state, "out");
    let outstream;
    
    multi_regex_test!(
        state;
        ({
            outstream =
                maj_open_stream(&mut state, out_sym.clone(),
                                Maj::string("test-streams-out.txt"));
            outstream.clone()
        }, RE_OUT_STREAM);
    );
    multi_boolean_test!(
        (maj_close_stream(&mut state, outstream.clone()), true);
        (maj_close_stream(&mut state, outstream.clone()), false);
    );
    multi_fail_test!(
        maj_open_stream(&mut state, out_sym.clone(), Maj::string("/"));
        maj_close_stream(&mut state, out_sym.clone());
    );
}

#[test]
fn primitives_stat() {
    use crate::axioms::primitives::{
        maj_stat,
        maj_open_stream,
        maj_close_stream
    };
    let mut state = MajState::new();
    let outsym = Maj::symbol(&mut state, "out");
    let outstream =
        maj_open_stream(&mut state, outsym,
                        Maj::string("test-streams-out.txt"));

    if maj_errorp(outstream.clone()).to_bool() {
        panic!("Cannot open output stream");
    }
    
    multi_test!(
        state;
        (maj_stat(&mut state, outstream.clone()), "open");
        ({
            maj_close_stream(&mut state, outstream.clone());
            maj_stat(&mut state, outstream)
        }, "closed");
    );
    test_fail!(maj_stat(&mut state, Maj::integer(3)));
}

#[test]
fn primitives_write() {
    use crate::axioms::primitives::maj_write;
    let mut state = MajState::new();
    let outstream = state.make_stream_stdout();
    let instream  = state.make_stream_stdin();
    let x = Maj::symbol(&mut state, "x");
    let closure =
        // (lit closure nil (x) ((* x x)))
        maj_list!(Maj::lit(), Maj::closure(), Maj::nil(),
                  maj_list!(x.clone()),
                  maj_list!(
                      maj_list!(Maj::symbol(&mut state, "*"),
                                x.clone(), x)));
    test_boolean!(maj_write(&mut state, closure.clone(), outstream),
                  false);
    multi_fail_test!(
        maj_write(&mut state, closure.clone(), instream);
        maj_write(&mut state, closure.clone(), Maj::integer(5));
    );
}

#[test]
#[ignore]
fn primitives_read() {
    unimplemented!();
}

#[test]
#[ignore]
fn primitives_read_char_peek_char() {
    use crate::core::types::MajStreamDirection;
    use crate::axioms::primitives::{
        maj_peek_char,
        maj_read_char,
        maj_close_stream
    };
    use crate::axioms::predicates::maj_eq;
    let mut state = MajState::new();
    let eof = Maj::symbol(&mut state, "eof");
    let istream = Maj::stream(
        &mut state,
        "test-streams-in.txt",
        MajStreamDirection::In).unwrap();
    let stdout = state.make_stream_stdout();

    multi_test!(
        state;
        (maj_peek_char(&mut state, istream.clone()), "#\\H");
        (maj_peek_char(&mut state, istream.clone()), "#\\H");
        (maj_read_char(&mut state, istream.clone()), "#\\H");
        (maj_read_char(&mut state, istream.clone()), "#\\e");
        ({
            while !maj_eq(maj_read_char(&mut state, istream.clone()),
                          eof.clone()).to_bool() {}
            maj_peek_char(&mut state, istream.clone())
        }, "eof");
        (maj_read_char(&mut state, istream.clone()), "eof");
    );

    test_boolean!(maj_close_stream(&mut state, istream.clone()),
                  true);
    
    multi_fail_test!(
        maj_read_char(&mut state, stdout.clone());
        maj_peek_char(&mut state, stdout);
        maj_read_char(&mut state, istream.clone());
        maj_peek_char(&mut state, istream.clone());
        maj_read_char(&mut state, Maj::fraction(2, 3));
        maj_peek_char(&mut state, Maj::fraction(2, 3));
    );
}

#[test]
fn primitives_write_char_write_string() {
    use crate::axioms::primitives::{
        maj_write_char,
        maj_write_string,
        maj_close_stream
    };
    use crate::core::types::MajStreamDirection;
    let mut state = MajState::new();
    let ostream = Maj::stream(
        &mut state,
        "test-streams-out.txt",
        MajStreamDirection::Out).unwrap();
    let stdin  = state.make_stream_stdin();
    let stdout = state.make_stream_stdout();

    multi_test!(
        state;
        (maj_write_char(
            &mut state, Maj::character('a'), ostream.clone()),
         "nil");
        (maj_write_string(
            &mut state, Maj::string("Hello"), ostream.clone()),
         "nil");
    );

    test_boolean!(
        maj_close_stream(&mut state, ostream.clone()), true);
    
    multi_fail_test!(
        maj_write_char(
            &mut state, Maj::character('a'), ostream.clone());
        maj_write_string(
            &mut state, Maj::string("Hello"), ostream.clone());
        maj_write_char(
            &mut state, Maj::character('a'), stdin.clone());
        maj_write_string(
            &mut state, Maj::string("Hello"), stdin.clone());
        maj_write_char(
            &mut state, Maj::character('a'), Maj::integer(5));
        maj_write_string(
            &mut state, Maj::string("Hello"), Maj::integer(5));
        maj_write_char(
            &mut state, Maj::integer(5), stdout.clone());
        maj_write_string(
            &mut state, Maj::integer(5), stdout.clone());
    );
}

#[test]
fn environments_global_bindings() {
    // use crate::axioms::MajRawSym;
    // use crate::axioms::utils::sym_from_raw;
    let mut state = MajState::new();
    let my_value = Maj::symbol(&mut state, "my-value");

    multi_eval_ast_test!(
        state;
        // (def my-value 5)
        (maj_list!(Maj::symbol(&mut state, "def"),
                   my_value.clone(),
                   Maj::integer(5)),
         Maj::nil(),
         "my-value");
        // my-value
        (my_value.clone(), Maj::nil(), "5");
    );
}

#[test]
fn environments_lexical_bindings() {
    use crate::axioms::MajRawSym;
    use crate::axioms::utils::sym_from_raw;
    let mut state = MajState::new();
    let f = Maj::symbol(&mut state, "f");
    let def = Maj::symbol(&mut state, "def");
    let x = Maj::symbol(&mut state, "x");
    let g = Maj::symbol(&mut state, "g");
    let y = Maj::symbol(&mut state, "y");
    let plussym = Maj::symbol(&mut state, "+");
    let fnsym = sym_from_raw(MajRawSym::Fn);
    multi_eval_ast_test!(
        state;
        // (def f (fn (x) (+ x 1)))
        (maj_list!(def.clone(), f.clone(),
                   maj_list!(fnsym.clone(),
                             maj_list!(x.clone()),
                             maj_list!(
                                 plussym.clone(),
                                 x.clone(),
                                 Maj::integer(1)))),
         Maj::nil(),
         "f");
        // (f 5)
        (maj_list!(f.clone(), Maj::integer(5)),
         Maj::nil(),
         "6");
        // (def g ((fn (x) (fn (y) (+ x y))) 9))
        (maj_list!(def.clone(), g.clone(),
                   maj_list!(
                       maj_list!(fnsym.clone(),
                                 maj_list!(x.clone()),
                                 maj_list!(fnsym.clone(),
                                           maj_list!(y.clone()),
                                           maj_list!(
                                               plussym.clone(),
                                               x.clone(),
                                               y.clone()))),
                       Maj::integer(9))),
         Maj::nil(),
         "g");
        // (g 4)
        (maj_list!(g.clone(), Maj::integer(4)),
         Maj::nil(),
         "13");
    );
}

#[test]
fn environments_dynamic_bindings() {
    use crate::axioms::MajRawSym;
    use crate::axioms::utils::sym_from_raw;
    let mut state = MajState::new();
    let my_function = Maj::symbol(&mut state, "my-function");
    let def = Maj::symbol(&mut state, "def");
    let my_value = Maj::symbol(&mut state, "*my-value*");
    let fnsym = sym_from_raw(MajRawSym::Fn);
    
    multi_eval_ast_test!(
        state;
        // (def *my-value* 5)
        (maj_list!(def.clone(), my_value.clone(),
                   Maj::integer(5)),
         Maj::nil(),
         "*my-value*");
        // *my-value*
        (my_value.clone(), Maj::nil(), "5");
        // (def my-function (fn () *my-value*))
        (maj_list!(def.clone(), my_function.clone(),
                   maj_list!(
                       fnsym.clone(), Maj::nil(),
                       my_value.clone())),
         Maj::nil(),
         "my-function");
        (maj_list!(my_function.clone()),
         Maj::nil(),
         "5");
        // ((fn (*my-value*) (my-function)) 6)
        (maj_list!(
            maj_list!(fnsym.clone(),
                      maj_list!(my_value.clone()),
                      maj_list!(my_function.clone())),
            Maj::integer(6)),
         Maj::nil(),
         "6");
        // (let ((*my-value* 7))
        //   (my-function))
        (maj_list!(my_function.clone()),
         maj_list!(Maj::cons(my_value.clone(),
                             Maj::integer(7))),
         "7");
    );
}

#[test]
fn environment_mutability_set() {
    let mut state = MajState::new();
    let def = Maj::symbol(&mut state, "def");
    let set = Maj::symbol(&mut state, "set");
    let x   = Maj::symbol(&mut state, "x");
    let mutate_x = Maj::symbol(&mut state, "mutate-x");
    let mutate_x_local = Maj::symbol(&mut state, "mutate-x-local");
    multi_eval_ast_test!(
        state;
        // (def x 5)
        (maj_list!(def.clone(), x.clone(), Maj::integer(5)),
         Maj::nil(),
         "x");

        // x
        (x.clone(), Maj::nil(), "5");
        
        // (set x 9)
        (maj_list!(set.clone(), x.clone(), Maj::integer(9)),
         Maj::nil(),
         "x");
        
        // x
        (x.clone(), Maj::nil(), "9");

        // (do (set x (1+ x)) x)
        // context: ((x . 12))
        (maj_list!(
            Maj::do_sym(),
            maj_list!(
                set.clone(), x.clone(),
                maj_list!(Maj::symbol(&mut state, "1+"),
                          x.clone())),
            x.clone()),
         maj_list!(Maj::cons(x.clone(), Maj::integer(12))),
         "13");

        // (defn mutate-x () (set x (+ x 2)) x)
        (maj_list!(
            Maj::symbol(&mut state, "defn"),
            mutate_x.clone(), Maj::nil(),
            maj_list!(
                set.clone(), x.clone(),
                maj_list!(
                    Maj::symbol(&mut state, "+"),
                    x.clone(), Maj::integer(2))),
            x.clone()),
         Maj::nil(),
         "mutate-x");
        
        // (mutate-x) altering x globally
        (maj_list!(mutate_x.clone()), Maj::nil(), "11");
        
        // (mutate-x) altering x dynamically
        // context: ((x . 12))
        (maj_list!(mutate_x.clone()),
         maj_list!(Maj::cons(x.clone(), Maj::integer(12))),
         "14");

        // x, unchanged
        (x.clone(), Maj::nil(), "11");

        // (defn mutate-x-local () (set x (+ x 2)) x)
        // context: ((x . 12))
        (maj_list!(
            Maj::symbol(&mut state, "defn"),
            mutate_x_local.clone(), Maj::nil(),
            maj_list!(
                set.clone(), x.clone(),
                maj_list!(
                    Maj::symbol(&mut state, "+"),
                    x.clone(), Maj::integer(2))),
            x.clone()),
         maj_list!(Maj::cons(x.clone(), Maj::integer(12))),
         "mutate-x-local");

        // (mutate-x-local) changing captured environment
        (maj_list!(mutate_x_local.clone()), Maj::nil(), "14");
        
        // x, unchanged
        (x.clone(), Maj::nil(), "11");

        // (mutate-x-local) changing captured environment
        (maj_list!(mutate_x_local.clone()), Maj::nil(), "16");

        // x, unchanged
        (x.clone(), Maj::nil(), "11");
    );
}

#[test]
fn environment_mutability_set_car_set_cdr() {
    let mut state = MajState::new();
    let def     = Maj::symbol(&mut state, "def");
    let defn    = Maj::symbol(&mut state, "defn");
    let set_car = Maj::symbol(&mut state, "set-car");
    let set_cdr = Maj::symbol(&mut state, "set-cdr");
    let x       = Maj::symbol(&mut state, "x");
    let a       = Maj::symbol(&mut state, "a");
    let b       = Maj::symbol(&mut state, "b");
    let replace_x_elts = Maj::symbol(&mut state, "replace-x-elts");
    let replace_x_elts_local =
        Maj::symbol(&mut state, "replace-x-elts-local");
    multi_eval_ast_test!(
        state;
        // (def x '(1 . 2))
        (maj_list!(
            def.clone(), x.clone(),
            maj_list!(Maj::quote(), Maj::cons(Maj::integer(1),
                                              Maj::integer(2)))),
         Maj::nil(),
         "x");
        
        // x
        (x.clone(), Maj::nil(), "(1 . 2)");
        
        // (set-car x 5)
        (maj_list!(
            set_car.clone(), x.clone(), Maj::integer(5)),
         Maj::nil(),
         "(5 . 2)");
        
        // (set-cdr x 6)
        (maj_list!(
            set_cdr.clone(), x.clone(), Maj::integer(6)),
         Maj::nil(),
         "(5 . 6)");
        
        // x
        (x.clone(), Maj::nil(), "(5 . 6)");
        
        // defining replace-x-elts
        (maj_list!(
            defn.clone(), replace_x_elts.clone(),
            maj_list!(a.clone(), b.clone()),
            maj_list!(set_car.clone(), x.clone(), a.clone()),
            maj_list!(set_cdr.clone(), x.clone(), b.clone()),
            x.clone()),
         Maj::nil(),
         "replace-x-elts");

        // (replace-x-elts 20 90)
        // context: ((x . (9 . 10)))
        (maj_list!(
            replace_x_elts.clone(),
            Maj::integer(20), Maj::integer(90)),
         maj_list!(Maj::cons(x.clone(),
                             Maj::cons(Maj::integer(9),
                                       Maj::integer(10)))),
         "(20 . 90)");

        // x
        (x.clone(), Maj::nil(), "(5 . 6)");

        // (replace-x-elts 30 45)
        (maj_list!(replace_x_elts.clone(),
                   Maj::integer(30), Maj::integer(45)),
         Maj::nil(),
         "(30 . 45)");
        
        // x
        (x.clone(), Maj::nil(), "(30 . 45)");

        // defining replace-x-elts-local
        // context: ((x . (9 . 10)))
        (maj_list!(
            defn.clone(), replace_x_elts_local.clone(),
            maj_list!(a.clone(), b.clone()),
            maj_list!(set_car.clone(), x.clone(), a.clone()),
            maj_list!(set_cdr.clone(), x.clone(), b.clone()),
            x.clone()),
         maj_list!(Maj::cons(x.clone(),
                             Maj::cons(Maj::integer(9),
                                       Maj::integer(10)))),
         "replace-x-elts-local");

        // (replace-x-elts-local 13 50)
        (maj_list!(
            replace_x_elts_local.clone(),
            Maj::integer(13), Maj::integer(50)),
         Maj::nil(),
         "(13 . 50)");

        // x
        (x.clone(), Maj::nil(), "(30 . 45)");
    );
}

#[test]
fn reader_simple_expressions() {
    let mut state = MajState::new();
    multi_parser_test!(
        state;
        ("", vec![], "nil");
        ("test", vec!["test"], "(test)");
        ("(defn foo ()
            (print \"Hello world\"))",
         vec!["(", "defn", "foo", "(", ")",
              "(", "print", "\"Hello world\"", ")", ")"],
         "((defn foo nil (print \"Hello world\")))");
        ("foo (foo) 5",
         vec!["foo", "(", "foo", ")", "5"],
         "(foo (foo) 5)");
    );
    multi_parser_fail_test!(
        state;
        "(')";
        "(a . b c)";
        "(a .";
        "(a . ";
        "(a . b";
        "a b . c d e";
        "foo bar '";
        "foo bar ,";
        "foo bar ,@";
        "foo ',@";
        "#\\invalidchar";
    );
}

#[test]
#[ignore]
fn reader_numbers() {
    let mut state = MajState::new();
    multi_parser_fail_test!(
        state;
        // These are supposed to be parser errors!
        // Fix the parser so that these produce proper errors.
        "J";
        "j";
        ".";
        "/";
        "1/0";
        "1/00";
    );
}

#[test]
fn reader_strings() {
    let mut state = MajState::new();
    multi_parser_test!(
        state;
        ("\"Hello\\nworld\"",
         vec!["\"Hello\\nworld\""],
         "(\"Hello\nworld\")");
        ("\"Hello\\tworld\"",
        vec!["\"Hello\\tworld\""],
        "(\"Hello	world\")");
    );
}

#[test]
fn reader_comments() {
    let mut state = MajState::new();
    multi_parser_test!(
        state;
        (";", vec![], "nil");
        ("; ", vec![], "nil");
        (";;", vec![], "nil");
        (";; ", vec![], "nil");
        (";;;", vec![], "nil");
        (";;; ", vec![], "nil");
        ("; Comentário", vec![], "nil");
        (";; Comentário", vec![], "nil");
        (";;; Comentário", vec![], "nil");
        (";; Comentários seguidos
          ;; Em várias linhas",
         vec![],
         "nil");
        (";; Comentário com tokens
          (+ 1 2)",
         vec!["(", "+", "1", "2", ")"],
         "((+ 1 2))");
        ("(+ 1 2) ; Comentário ao final",
         vec!["(", "+", "1", "2", ")"],
        "((+ 1 2))");
        ("(+ 1 2) ; Comentário ao final e continua
          (- 3 4)",
         vec!["(", "+", "1", "2", ")",
              "(", "-", "3", "4", ")"],
        "((+ 1 2) (- 3 4))");
    );
}

#[test]
fn reader_quote() {
    let mut state = MajState::new();
    multi_parser_test!(
        state;
        ("'foo", vec!["'", "foo"], "((quote foo))");
        ("'a 'b 'c", vec!["'", "a", "'", "b", "'", "c"],
         "((quote a) (quote b) (quote c))");
        ("'a", vec!["'", "a"], "((quote a))");
        ("'(1 2 3)", vec!["'", "(", "1", "2", "3", ")"],
         "((quote (1 2 3)))");
    );
}

#[test]
fn reader_quasiquote_unquote_unquotesplice() {
    let mut state = MajState::new();
    multi_parser_test!(
        state;
        ("`(x)", vec!["`", "(", "x", ")"],
         "((quasiquote (x)))");
        ("`(+ ,x ,y)", vec!["`", "(", "+", ",", "x",
                            ",", "y", ")"],
         "((quasiquote (+ (unquote x) (unquote y))))");
        ("`,foo", vec!["`", ",", "foo"],
         "((quasiquote (unquote foo)))");
        ("`(,y ,@x)", vec!["`", "(", ",", "y", ",@", "x", ")"],
         "((quasiquote ((unquote y) (unquote-splice x))))");
    );
}

#[test]
fn reader_vector_syntax() {
    let mut state = MajState::new();
    multi_parser_test!(
        state;
        ("[]", vec!["[", "]"], "((vector))");
        ("[1 2 'a 3 (+ 2 5)]",
         vec!["[", "1", "2", "'", "a", "3",
              "(", "+", "2", "5", ")", "]"],
         "((vector 1 2 (quote a) 3 (+ 2 5)))");
    );
    multi_parser_fail_test!(
        state;
        "[']";
        "[1 2 3";
    );
}

#[test]
fn evaluator_quote() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(quote x)", "x");
        ("(quote (1 2 3))", "(1 2 3)");
        ("(quote (+ 5 6))", "(+ 5 6)");
    );
}

#[test]
fn evaluator_def() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(def x 5)", "x");
    );
    multi_eval_fail_test!(
        state;
        "(def (foo bar) 5)";
        "(def y (length x))";
    );
}

#[test]
fn evaluator_set() {
    let mut state = MajState::new();
    multi_eval_fail_test!(
        state;
        "(set x 5)";
    );
    multi_eval_test!(
        state;
        ("(def x 1)", "x");
        ("x", "1");
        ("(set x 5)", "x");
        ("x", "5");
        ("(let ((x 6)) x)", "6");
        ("(let ((x 6))
            (set x 9)
            x)",
         "9");
        ("x", "5");
    );
}

#[test]
fn evaluator_set_car() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(def x (quote (1 . 2)))", "x");
        ("(set-car x 3)", "(3 . 2)");
        ("(set-car (quote (9 . 0)) (quote f))",
         "(f . 0)");
        ("(let ((x (quote (4 . 5))))
            (set-car x 9))",
         "(9 . 5)");
        ("x", "(3 . 2)");
        ("(let ((a x))
            (set-car a 4))",
         "(4 . 2)");
        ("x", "(4 . 2)");
        ("(let ((a (copy x)))
            (set-car a 5))",
         "(5 . 2)");
        ("x", "(4 . 2)");
    );
}

#[test]
fn evaluator_set_cdr() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(def x (quote (1 . 2)))", "x");
        ("(set-cdr x 3)", "(1 . 3)");
        ("(set-cdr (quote (9 . 0)) (quote f))",
         "(9 . f)");
        ("(let ((x (quote (4 . 5))))
            (set-cdr x 9))",
         "(4 . 9)");
        ("x", "(1 . 3)");
        ("(let ((a x))
            (set-cdr a 4))",
         "(1 . 4)");
        ("x", "(1 . 4)");
        ("(let ((a (copy x)))
            (set-cdr a 5))",
         "(1 . 5)");
        ("x", "(1 . 4)");
    );
}

#[test]
fn evaluator_if() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(if (nilp (quote a))
              (quote is-nil)
              (quote is-not-nil))",
         "is-not-nil");
        ("(if (nilp nil)
              (quote is-nil)
              (quote is-not-nil))",
         "is-nil");
    );
}

#[test]
fn evaluator_fn() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(fn (x) (* x x))",
         "(lit closure nil (x) ((* x x)))");
        ("(fn (x . rest) (list x rest))",
         "(lit closure nil (x . rest) ((list x rest)))");
        ("((fn (x) (* x x)) 5)",
         "25");
        ("(def square (fn (x) (* x x)))",
         "square");
        ("(square 5)", "25");
    );
}

#[test]
fn evaluator_mac() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(mac (x) (quasiquote (list (unquote x))))",
         "(lit macro (lit closure nil (x) ((quasiquote (list (unquote x))))))");
        ("((mac (x) (quasiquote (list (unquote x)))) 5)",
         "(5)");
        ("(def encapsulate
            (mac (x) (quasiquote (list (unquote x)))))",
         "encapsulate");
        ("(macroexpand-1 (quote (encapsulate 5)))",
         "(list 5)");
        ("(encapsulate 5)", "(5)");
    );
}

#[test]
fn evaluator_quasiquote_unquote_unquote_splice() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(quasiquote a)", "a");
        ("(let ((x 5)  (y 6))
            (quasiquote (1 (unquote x) (unquote y))))",
         "(1 5 6)");
        ("(let ((lst (list 1 2 3)))
            (quasiquote (0 (unquote lst))))",
         "(0 (1 2 3))");
        ("(let ((lst (list 1 2 3)))
            (quasiquote (0 (unquote-splice lst))))",
         "(0 1 2 3)");
    );
}

#[test]
fn evaluator_do() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(do (quote a)
              (quote b)
              (list (quote c)
                    (quote d)
                    (quote e)))",
         "(c d e)");
        ("(do)", "nil");
    );
}

#[test]
#[ignore]
fn evaluator_apply() {
    unimplemented!();
}

#[test]
fn evaluator_while() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(let ((x 0)  (lst nil))
            (while (< x 5)
              (set lst (cons x lst))
              (set x (1+ x)))
            (list x lst))",
         "(5 (4 3 2 1 0))");
        ("(let ((lst (quote (1 2 3 4 5)))
                (result nil))
            (while (not (nilp lst))
              (let (((num . rest) lst))
                (set result
                     (cons (= 2 num) result))
                (set lst rest)))
            result)",
         "(nil nil nil t nil)");
    );
    multi_eval_fail_test!(
        state;
        "(let ((lst (quote (1 2 3 a 5))))
           (while (not (nilp lst))
             (let (((num . rest) lst))
               (= 2 num))
             (set lst rest)))";
        "(let ((lst (quote (1 2 3 NaN 5))))
           (while (not (nilp lst))
             (let (((num . rest) lst))
               (= 2 num))
             (set lst rest)))";
    );
}

#[test]
fn evaluator_and() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(and (= 1 1) (= 0.5 1/2) (quote ok))",
         "ok");
        ("(and (= 1 1) (< 0.5 1/2) (quote ok))",
         "nil");
        ("(and)", "t");
    );
}

#[test]
fn evaluator_or() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(or (not (= 1 1))
              (< 0.5 1/2)
              (quote ok))",
         "ok");
        ("(or (= 1 1) (< 0.5 1/2))",
         "t");
        ("(or)", "nil");
    );
}

#[test]
fn evaluator_letrec() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(let ((results nil))
            (letrec ((foo ()
                       (set results
                         (cons (quote foo) results))
                       (bar))
                     (bar ()
                       (set results
                         (cons (quote bar) results))
                       (baz))
                     (baz ()
                       (set results
                         (cons (quote baz) results))
                       results))
              (foo)))",
         "(baz bar foo)");
        ("(letrec ((iter (x)
                     (if (> x 0)
                       (iter (1- x))
                       (quote finished))))
            (iter 5))",
         "finished");
        ("(def my-function
               (letrec ((foo () (bar))
                        (bar () (quote hello-from-bar)))
                 foo))",
         "my-function");
        ("(my-function)", "hello-from-bar");
    );
}

#[test]
#[ignore]
fn evaluator_unwind_protect() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(def *my-stream* nil)", "*my-stream*");
        ("(unwind-protect
              (do (set *my-stream*
                       (open-stream 'in
                                    \"test-streams-in.txt\"))
                  (read-char *my-stream*))
            (do (close-stream *my-stream*)
                (set *my-stream* nil)))",
         "#\\H");
        ("*my-stream*", "nil");
        ("(and (set *my-stream*
                  (open-stream 'out \"test-streams-out.txt\"))
               t)",
         "t");
    );
    multi_eval_fail_test!(
        state;
        "(unwind-protect (read-char *my-stream*)
           (do (close-stream *my-stream*)
               (set *my-stream* nil)))";
             
    );
    multi_eval_test!(
        state;
        ("*my-stream*", "nil");
    );
}

#[test]
#[ignore]
fn evaluator_closure_application_default() {
    unimplemented!();
}

#[test]
#[ignore]
fn evaluator_closure_application_partial() {
    unimplemented!();
}

#[test]
#[ignore]
fn evaluator_closure_application_optional_args() {
    unimplemented!();
}

#[test]
#[ignore]
fn evaluator_closure_application_forced() {
    unimplemented!();
}

#[test]
fn evaluator_bootstrap_constants() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(type *stdin*)", "stream");
        ("(type *stdout*)", "stream");
        ("*ulps*", "3");
    );
}

#[test]
fn evaluator_bootstrap_map_mapc() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(map (fn (x) (* x x))
               (quote (1 2 3 4 5)))",
         "(1 4 9 16 25)");
        ("(map (fn (x) (* x x))
               (quote (1 2 3 4 . 5)))",
         "(1 4 9 16)");
        ("(mapc (fn (x) (* x x))
                (quote (1 2 3 4 5)))",
         "nil");
        ("(let ((results nil))
            (mapc (fn (x)
                    (set results
                         (cons (* x x) results)))
                  (quote (1 2 3 4 5)))
            results)",
         "(25 16 9 4 1)");
    );
}

#[test]
fn evaluator_bootstrap_vectorequal() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(vector= [1 2 3] [1 2 3])",     "t");
        ("(let ((v [5 6 7 8]))
            (vector= v [5 6 7 8]))",      "t");
        ("(vector= [1 2] [3 4 5])",       "nil");
        ("(vector= \"Hello\" \"Hello\")", "t");
        ("(vector= ['a 'b] [#\\a #\\b])", "nil");
    );
    multi_eval_fail_test!(
        state;
        "(vector= 'a ['a])";
        "(vector= [5] 5)";
    );
}

#[test]
fn evaluator_bootstrap_equal() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(equal 0.5 1/2)",             "t");
        ("(equal [1 2 3] [1 2 3])",     "t");
        ("(equal 'a 'b)",               "nil");
        ("(equal '(a b c) '(a b c))",   "t");
        ("(equal #\\F #\\F)",           "t");
        ("(equal *stdin* *stdin*)",     "t");
        ("(equal *stdin* *stdout*)",    "nil");
        ("(equal 20 'a)",               "nil");
        ("(equal *stdout* '(a b c))",   "nil");
        ("(equal [6 5] '(6 5))",        "nil");
        ("(equal \"Teste\" \"Teste\")", "t");
    );
}

#[test]
fn evaluator_bootstrap_assp_assoc() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(assp (fn (x) (eq (quote b) x))
                (quote ((a . 1) (b . 2) (c . 3))))",
         "(b . 2)");
        ("(assp (eq (quote b))
                (quote ((a . 1) (b . 2) (c . 3))))",
         "(b . 2)");
        ("(assp (eq (quote z))
                (quote ((a . 1) (b . 2) (c . 3))))",
         "nil");
        ("(assoc (quote b)
                 (quote ((a . 1) (b . 2) (c . 3))))",
         "(b . 2)");
        ("(assoc (quote a)
                 (quote ((a . 1) (b . 2) (c . 3))))",
         "(a . 1)");
        ("(letfn ((equal (x y)
                    (= x y)))
            (assoc 2 (quote ((1 . a) (2 . b) (3 . c)))))",
         "(2 . b)");
        ("(assoc (quote z)
                 (quote ((a . 1) (b . 2) (c . 3))))",
         "nil");
    );
}

#[test]
fn evaluator_bootstrap_member() {
    let mut state = MajState::new();
    multi_eval_test!(
        state;
        ("(member (quote a) (quote (x y z a b c)))",
         "(a b c)");
        ("(member (quote x) (quote (x y z a b c)))",
         "(x y z a b c)");
        ("(member (quote c) (quote (x y z a b c)))",
         "(c)");
        ("(letfn ((equal (x y)
                    (= x y)))
            (member 1 (quote (5 6 7 1 2 3))))",
         "(1 2 3)");
        ("(member (quote n) (quote (x y z a b c)))",
         "nil");
    );
}

#[test]
#[ignore]
fn evaluator_primitives_application() {
    unimplemented!();
}

#[test]
fn macros_cond() {
    let mut state = MajState::new();
    multi_macroexpand_1_test! {
        state;
        ("(cond (a b)
                (t c))",
         "(if a (do b) (cond (t c)))");
        ("(cond ((numberp x) x))",
         "(if (numberp x) (do x) nil)");
    };
    multi_eval_test! {
        state;
        ("(let ((x 1))
            (cond ((zerop x) 0)
                  ((< x 1) (- x))
                  (t x)))",
         "1");
        ("(let ((x 0))
            (cond ((zerop x) 0)
                  ((< x 1) (- x))
                  (t x)))",
         "0");
        ("(cond (t (quote a)))", "a");
        ("(cond ((eq (quote a)
                     (quote b))
                 (quote ok)))",
         "nil");
    };
}

#[test]
fn macros_defn() {
    let mut state = MajState::new();
    multi_macroexpand_1_test! {
        state;
        ("(defn name lambda-list body1 body2)",
         "(def name (fn lambda-list body1 body2))");
        ("(defn identity (x) x)",
         "(def identity (fn (x) x))");
    }
    multi_eval_test! {
        state;
        ("(defn identity (x) x)", "identity");
        ("(closurep identity)",   "t");
        ("identity", "(lit closure nil (x) (x))");
    }
}

#[test]
fn macros_defmac() {
    let mut state = MajState::new();
    multi_macroexpand_1_test! {
        state;
        ("(defmac name lambda-list body1 body2)",
         "(def name (mac lambda-list body1 body2))");
        ("(defmac call (f) (quasiquote ((unquote f))))",
         "(def call (mac (f) (quasiquote ((unquote f)))))");
    }
    multi_eval_test! {
        state;
        ("(defmac call (f) (quasiquote ((unquote f))))",
         "call");
        ("(macrop call)",   "t");
        ("(defn foo () 1)", "foo");
        ("(call foo)", "1");
    }
}

#[test]
fn macros_let_letstar() {
    let mut state = MajState::new();
    multi_macroexpand_1_test! {
        state;
        ("(let ((x 1)  (y 2))
            (+ x y))",
         "((fn (y x) (+ x y)) 2 1)");
        ("(let* ((x 1)  (y (1+ x)))
            (+ x y))",
         "(let ((x 1)) (let* ((y (1+ x))) (+ x y)))");
        ("(let* ((x (* 2 3))
                 (y (1+ x))
                 (z (+ y 3)))
            (+ x y z))",
         "(let ((x (* 2 3))) (let* ((y (1+ x)) (z (+ y 3))) (+ x y z)))");
    }
    multi_eval_test! {
        state;
        ("(let ((x 1)  (y 2))
            (+ x y))",
         "3");
        ("(let* ((x (* 2 3))
                 (y (1+ x))
                 (z (+ y 3)))
            (+ x y z))",
         "23");
    }
}

#[test]
fn macros_letfn_letfnstar() {
    let mut state = MajState::new();
    multi_macroexpand_1_test! {
        state;
        ("(letfn ((foo (x) (+ 1 2 x))
                  (bar (x) x))
            (foo 5))",
         "((fn (bar foo) (foo 5)) (fn (x) x) (fn (x) (+ 1 2 x)))");
        ("(letfn* ((foo (x) (+ 1 2 x))
                   (bar (x) (foo x)))
            (bar 5))",
         "(letfn ((foo (x) (+ 1 2 x))) (letfn* ((bar (x) (foo x))) (bar 5)))");
    }
    multi_eval_test! {
        state;
        ("(letfn ((foo (x) (+ 1 2 x)))
            (foo 5))",
         "8");
        ("(letfn* ((foo (x) (+ 1 2 x))
                   (bar (x) (foo x)))
            (bar 5))",
         "8");
    };
}

#[test]
fn macros_when_unless() {
    let mut state = MajState::new();
    multi_macroexpand_1_test! {
        state;
        ("(when t
            (print \"Hello\")
            (+ 2 3))",
         "(if t (do (print \"Hello\") (+ 2 3)) nil)");
        ("(unless t
            (print \"Hello\")
            (+ 2 3))",
         "(if (not t) (do (print \"Hello\") (+ 2 3)) nil)");
    }
    multi_eval_test! {
        state;
        ("(when t (+ 2 3))",         "5");
        ("(when (= 2 3) (+ 2 3))",   "nil");
        ("(unless t (+ 2 3))",       "nil");
        ("(unless (= 2 3) (+ 2 3))", "5");
    }
}

#[test]
fn macros_until() {
    let mut state = MajState::new();
    multi_macroexpand_1_test! {
        state;
        ("(until (>= x 5)
            (set x (1+ x)))",
         "(while (not (>= x 5)) (set x (1+ x)))");
    }
    multi_eval_test! {
        state;
        ("(let ((x 0))
            (until (>= x 5)
              (set x (1+ x)))
            x)",
         "5");
    }
}

#[test]
#[ignore]
fn macros_repeat() {
    // Needs a macroexpand-1 macro which accepts
    // generated-symbol regex
    unimplemented!();
}

#[test]
#[ignore]
fn macros_with_open_stream() {
    let mut state = MajState::new();
    multi_macroexpand_1_test! {
        state;
        ("(with-open-stream (s (quote in) \"test-streams-in.txt\")
            (read-char s))",
         "(let ((s (open-stream (quote in) \"test-streams-in.txt\"))) (unwind-protect (do (read-char s)) (close-stream s)))");
    }
    multi_eval_test! {
        state;
        ("(with-open-stream (s (quote in) \"test-streams-in.txt\")
            (read-char s))",
         "#\\H");
    }
}
