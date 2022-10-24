pub mod definitions;
pub mod predicates;
pub mod primitives;
pub mod bootstrap;
pub mod utils;

pub use definitions::MajRawSym;

use gc::Gc;
use crate::core::{ MajState, Maj };

pub type MajPrimFn = fn(&mut MajState, Gc<Maj>, Gc<Maj>) -> Gc<Maj>;

#[macro_export]
macro_rules! maj_destructure_args {
    ($args:expr, $first:ident) => {
        let $first:  Gc<Maj> = maj_car($args.clone());
    };

    ($args:expr, $first:ident, $rest:ident) => {
        let $first:  Gc<Maj> = maj_car($args.clone());
        let $rest:   Gc<Maj> = maj_cdr($args.clone());
    };

    ($args:expr, $first:ident, $rest:ident, $second:ident) => {
        let $first:  Gc<Maj> = maj_car($args.clone());
        let $rest:   Gc<Maj> = maj_cdr($args.clone());
        let $second: Gc<Maj> = maj_car($rest.clone());
    };

    ($args:expr, $first:ident, $rest:ident, $second:ident,
    $rest2:ident) => {
        let $first:  Gc<Maj> = maj_car($args.clone());
        let $rest:   Gc<Maj> = maj_cdr($args.clone());
        let $second: Gc<Maj> = maj_car($rest.clone());
        let $rest2:  Gc<Maj> = maj_cdr($rest.clone());
    };

    ($args:expr, $first:ident, $rest:ident, $second:ident,
     $rest2:ident, $third:ident) => {
        let $first:  Gc<Maj> = maj_car($args.clone());
        let $rest:   Gc<Maj> = maj_cdr($args.clone());
        let $second: Gc<Maj> = maj_car($rest.clone());
        let $rest2:  Gc<Maj> = maj_cdr($rest.clone());
        let $third:  Gc<Maj> = maj_car($rest2.clone());
    };
}

#[derive(Copy, Clone)]
pub enum MajPrimArgs {
    None,
    Required(u64),
    Variadic(u64)
}

use definitions::maj_gen_symbols;
use predicates::maj_gen_predicates;
use primitives::maj_gen_primitives;
use bootstrap::maj_gen_bootstrap;

pub fn majestic_initialize(mut state: &mut MajState) {
    maj_gen_symbols(&mut state);
    maj_gen_predicates(&mut state);
    maj_gen_primitives(&mut state);
    maj_gen_bootstrap(&mut state);
}
