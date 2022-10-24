use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs::File;
use gc::Gc;
use super::Maj;
use crate::axioms::{ MajPrimFn, MajPrimArgs };
use bimap::BiMap;

use std::fmt;

type MajInternalStream = Option<File>;

pub struct MajState {
    symbols:      BiMap<u64, String>,
    last_sym:     u64,
    primitives:   HashMap<u64, (MajPrimFn, MajPrimArgs)>,
    streams:      Vec<MajInternalStream>,
    free_streams: VecDeque<usize>,
    stdin_peeked: Option<char>,
    global_env:   Gc<Maj>
}

impl MajState {
    pub fn new() -> MajState {
        use crate::axioms::majestic_initialize;
        let mut state =
            MajState {
                symbols:      BiMap::new(),
                primitives:   HashMap::new(),
                last_sym:     0,
                streams:      Vec::new(),
                free_streams: VecDeque::new(),
                stdin_peeked: None,
                global_env:   Maj::nil()
            };
        majestic_initialize(&mut state);
        state
    }
}

impl MajState {
    pub fn gen_symbol(&mut self, name: &str) -> u64 {
        match self.symbols.get_by_right(&name.to_string()) {
            Some(old_sym) => *old_sym,
            None => {
                let new_sym = self.last_sym;
                self.last_sym += 1;
                self.symbols.insert(new_sym, name.to_string());
                new_sym
            }
        }
    }
}

impl MajState {
    pub fn gen_random_symbol(&mut self) -> u64 {
        let new_sym = self.last_sym;
        let sym_name = format!(":G{}", new_sym);
        self.last_sym += 1;
        self.symbols.insert(new_sym, sym_name.to_string());
        new_sym
    }
}

impl MajState {
    pub fn symbol_name(&self, sym: &u64) -> String {
        match self.symbols.get_by_left(sym) {
            Some(string) => string.clone(),
            None => format!("~uninterned##{}", sym)
        }
    }
}

impl MajState {
    pub fn register_primitive(
        &mut self,
        name: &'static str,
        arity: MajPrimArgs,
        f: MajPrimFn
    ) {
        use crate::maj_list;
        let symbol = Maj::symbol(self, name);
        if let Maj::Sym(num) = *symbol.clone() {
            self.primitives.insert(num, (f, arity));
            let (arity_type, arity) = match arity {
                MajPrimArgs::None =>
                    (Maj::symbol(self, "required"),
                     Maj::integer(0)),
                MajPrimArgs::Required(n) =>
                    (Maj::symbol(self, "required"),
                     Maj::integer(n as i64)),
                MajPrimArgs::Variadic(n) =>
                    (Maj::symbol(self, "variadic"),
                     Maj::integer(n as i64)),
            };
            self.push(symbol.clone(),
                  maj_list!(Maj::lit(), Maj::prim(),
                            symbol, arity_type, arity));
        } else {
            panic!("Error creating symbol for primitive function");
        }
    }
}

impl MajState {
    pub fn find_primitive(&self, sym: Gc<Maj>) -> Option<&(MajPrimFn, MajPrimArgs)> {
        match *sym {
            Maj::Sym(num) => self.primitives.get(&num),
            _             => None,
        }
    }
}

impl fmt::Display for MajState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::printing::maj_format_env;
        let _ =
            writeln!(f, "{} symbols registered", self.symbols.len());
        let _ =
            writeln!(f, "{} streams registered ({} free)",
                     self.streams.len(),
                     self.free_streams.len());
        let _ =
            writeln!(f, "{} primitives registered", self.primitives.len());
        let _ =
            writeln!(f, "global environment table:");
        let env = self.global_env.clone();
        let _ =
            writeln!(f, "{}", maj_format_env(&self, env));
        Ok(())
    }
}

use super::types::{
    MajStream,
    MajStreamDirection,
    MajStreamType
};

impl MajState {
    pub fn make_stream_stdin(&mut self) -> Gc<Maj> {
        Gc::new(Maj::Stream(MajStream {
            direction: MajStreamDirection::In,
            handle:    usize::MAX,
            stype:     MajStreamType::Stdin
        }))
    }
}

impl MajState {
    pub fn make_stream_stdout(&mut self) -> Gc<Maj> {
        Gc::new(Maj::Stream(MajStream {
            direction: MajStreamDirection::Out,
            handle:    usize::MAX,
            stype:     MajStreamType::Stdout
        }))
    }
}

impl MajState {
    pub fn make_stream_stderr(&mut self) -> Gc<Maj> {
        Gc::new(Maj::Stream(MajStream {
            direction: MajStreamDirection::Out,
            handle:    usize::MAX,
            stype:     MajStreamType::Stderr
        }))
    }
}

impl MajState {
    pub fn make_stream(
        &mut self,
        file: &str,
        direction: MajStreamDirection
    ) -> Option<Gc<Maj>> {
        match direction {
            MajStreamDirection::In  => {
                let handle = File::open(file);
                if handle.is_err() {
                    return None;
                }

                let handle = handle.unwrap();
                let index;
                if self.free_streams.is_empty() {
                    self.streams.push(Some(handle));
                    index = self.streams.len() - 1;
                } else {
                    index = self.free_streams
                        .pop_front()
                        .unwrap();
                    self.streams[index] = Some(handle);
                }
                Some(Gc::new(Maj::Stream(
                    MajStream {
                        direction,
                        handle: index,
                        stype: MajStreamType::File,
                    })))
            }
            MajStreamDirection::Out => {
                let handle = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file);
                if handle.is_err() {
                    return None;
                }

                let handle = handle.unwrap();
                let index;
                if self.free_streams.is_empty() {
                    self.streams.push(Some(handle));
                    index = self.streams.len() - 1;
                } else {
                    index = self.free_streams
                        .pop_front()
                        .unwrap();
                    self.streams[index] = Some(handle);
                }
                Some(Gc::new(Maj::Stream(
                    MajStream {
                        direction,
                        handle: index,
                        stype: MajStreamType::File,
                    })))
            },
        }
    }
}

impl MajState {
    pub fn close_stream(&mut self, stream: Gc<Maj>) -> Gc<Maj> {
        use crate::axioms::primitives::maj_err;
        use crate::maj_list;
        if let Maj::Stream(mstream) = &*stream.clone() {
            if mstream.is_internal() {
                return maj_err(
                    Maj::string("Cannot close standard streams"),
                    Maj::nil());
            }

            let index = mstream.handle;
            if self.streams.len() <= index {
                maj_err(
                    Maj::string("Invalid stream {}"),
                    maj_list!(stream))
            } else {
                if self.streams[index].is_none() {
                    Maj::nil()
                } else {
                    self.streams[index] = None;
                    self.free_streams.push_back(index);
                    Maj::t()
                }
            }
        }  else {
            maj_err(
                Maj::string("Not a stream: {}"),
                maj_list!(stream))
        }
    }
}

impl MajState {
    pub fn stat_stream(&mut self, which: usize) -> Gc<Maj> {
        use crate::axioms::primitives::maj_err;
        if self.streams.len() <= which {
            maj_err(
                Maj::string("Invalid stream"),
                Maj::nil())
        } else {
            if self.streams[which].is_none() {
                Maj::nil()
            } else {
                Maj::t()
            }
        }
    }
}

impl MajState {
    pub fn borrow_stream(&self, which: usize) -> Option<&File> {
        if self.streams.len() <= which {
            None
        } else if self.streams[which].is_none() {
            None
        } else {
            Some(&self.streams[which].as_ref().unwrap())
        }
    }
}

impl MajState {
    pub fn push_stdin_peeked(&mut self, c: char) {
        if self.stdin_peeked.is_some() {
            panic!("Cannot overwrite peeked *stdin* character!");
        }
        self.stdin_peeked = Some(c);
    }
}

impl MajState {
    pub fn pop_stdin_peeked(&mut self) -> Option<char> {
        let result = self.stdin_peeked;
        self.stdin_peeked = None;
        result
    }
}

use super::environment::{
    maj_env_push,
    maj_env_lookup,
    maj_env_assoc
};

impl MajState {
    pub fn push(&mut self, sym: Gc<Maj>, val: Gc<Maj>) -> Gc<Maj> {
        let mut new_ge = self.global_env.clone();
        new_ge = maj_env_push(new_ge.clone(),
                              sym.clone(),
                              val.clone());
        self.global_env = new_ge;
        sym.clone()
    }
}

impl MajState {
    pub fn assoc(&self, lexenv: Gc<Maj>, sym: Gc<Maj>) -> Gc<Maj> {
        use crate::axioms::predicates::maj_errorp;
        let result = maj_env_assoc(lexenv, sym.clone());
        if maj_errorp(result.clone()).to_bool() {
            maj_env_assoc(self.global_env.clone(), sym)
        } else {
            result
        }
    }

    pub fn lookup(&self, lexenv: Gc<Maj>, sym: Gc<Maj>) -> Gc<Maj> {
        use crate::axioms::predicates::maj_errorp;
        let result = maj_env_lookup(lexenv, sym.clone());
        if maj_errorp(result.clone()).to_bool() {
            maj_env_lookup(self.global_env.clone(), sym)
        } else {
            result
        }
    }
}

impl MajState {
    pub fn get_global_env(&self) -> Gc<Maj> {
        self.global_env.clone()
    }
}
