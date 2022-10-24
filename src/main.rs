#![feature(seek_stream_len)]

pub mod core;
pub mod axioms;
pub mod printing;
pub mod evaluator;
pub mod reader;

#[cfg(test)]
mod tests;

use self::core::{ Maj, MajState };
use self::printing::{ maj_format, maj_format_raw };
use self::evaluator::maj_eval;
use self::reader::tokenizer::maj_tokenize;
use self::reader::parser::maj_parse;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::validate::{
    ValidationContext,
    ValidationResult,
    Validator
};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};
use colored::*;
use std::{ env, process };

#[derive(Completer, Helper, Highlighter, Hinter)]
struct MajInputValidator {}

impl Validator for MajInputValidator {
    fn validate(
        &self,
        ctx: &mut ValidationContext
    ) -> Result<ValidationResult, ReadlineError> {
        use ValidationResult::{
            Incomplete,
            Invalid,
            Valid
        };

        let input = ctx.input();
        let mut ignore_one = false;
        let mut count = 0;
        for c in input.chars() {
            if ignore_one {
                ignore_one = false;
            } else {
                match c {
                    '(' => {
                        count += 1;
                    },
                    ')' => {
                        if count == 0 {
                            count = -1;
                            break;
                        } else {
                            count -= 1;
                        }
                    },
                    '\\' => {
                        // Gambiarra for preventing
                        // incomplete input on characters
                        // such as #\( and #\)
                        ignore_one = true;
                    }
                    _ => {},
                }
            }
        }

        if count > 0 {
            Ok(Incomplete)
        } else if count < 0 {
            Ok(
                Invalid(Some("No matching parenthesis found"
                             .to_owned()))
            )
        } else {
            Ok(Valid(None))
        }
    }
}

fn repl(mut state: &mut MajState, options: &ArgsOptions) {
    use crate::axioms::predicates::maj_errorp;
    if !options.silent {
        println!("Press C-c or C-d to quit");
    }

    let validator = MajInputValidator {};

    let mut rl = Editor::new();
    rl.set_helper(Some(validator));
    let _ = rl.load_history(".majestic_history");
    let mut prompt = if options.silent {
        String::from("")
    } else {
        format!("{}", "> ".green())
    };

    let mut show_tokens = false;

    loop {
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                if line == "#t" {
                    show_tokens = !show_tokens;
                    prompt =
                        if options.silent {
                            String::from("")
                        } else if show_tokens {
                            format!("{}", "> ".magenta())
                        } else {
                            format!("{}", "> ".green())
                        };
                } else {
                    //use crate::printing::maj_pretty_format;
                    match maj_tokenize(line.as_ref()) {
                        Ok(tokens) => {
                            if show_tokens {
                                println!("{}",
                                         format!("{:?}", tokens)
                                         .magenta());
                            }
                            match maj_parse(&mut state,
                                                  tokens.clone()) {
                                Ok(expressions) => {
                                    if show_tokens {
                                        println!("{}",
                                                 maj_format_raw(
                                                     &state,
                                                     expressions.clone(),
                                                     false).magenta());
                                    }
                                    use crate::axioms::utils::{
                                        STACK_RED_ZONE,
                                        STACK_PER_RECURSION
                                    };
                                    let results =
                                        stacker::maybe_grow(
                                            STACK_RED_ZONE,
                                            STACK_PER_RECURSION,
                                            || maj_eval(&mut state,
                                                        Maj::cons(
                                                            Maj::do_sym(),
                                                            expressions),
                                                        Maj::nil()));
                                            if !maj_errorp(results.clone()).to_bool() {
                                                println!("{}",
                                                         maj_format(&state, results)
                                                         .cyan());
                                            } else {
                                                eprintln!("{} {}",
                                                          "Error:".red().bold(),
                                                          maj_format(&state, results));
                                            }
                                            
                                        },
                                    Err(msg) =>
                                        eprintln!("{} {}",
                                                  "Parser error:".red().bold(),
                                                  msg),
                                }
                            },
                            Err((line, msg)) =>
                                eprintln!("{} on line {}: {}",
                                          "Syntax error".red().bold(),
                                          line, msg),
                        };
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    if !options.silent {
                        println!("{}", "C-c".cyan().dimmed());
                    }
                    break
                },
                Err(ReadlineError::Eof) => {
                    if !options.silent {
                        println!("{}", "C-d".cyan().dimmed());
                    }
                    break
                },
                Err(err) => {
                    eprintln!("REPL Error: {:?}", err);
                    break
                }
            }
        }

        if let Err(_) = rl.save_history(".majestic_history") {
            eprintln!("Failed saving history to .majestic_history.");
        }

        if !options.silent {
            println!("Quaerendo invenietis.");
        }
    }

struct ArgsOptions {
    silent:      bool,
    loadfiles:   Vec<String>,
    programname: String,
    quit:        bool,
    execlines:   Vec<String>,
    showhelp:    bool,
}

impl ArgsOptions {
    fn empty() -> ArgsOptions {
        ArgsOptions {
            silent:     false,
            loadfiles:   Vec::new(),
            programname: String::new(),
            quit:        false,
            execlines:   Vec::new(),
            showhelp:    false,
        }
    }

    fn new() -> Result<ArgsOptions, String> {
        let mut options = ArgsOptions::empty();

        let mut fetch_load        = false;
        let mut fetch_exec        = false;
        let mut fetch_programname = true;

        for argument in env::args() {
            if fetch_programname {
                options.programname = argument;
                fetch_programname = false;
            } else if fetch_load {
                options.loadfiles.push(argument);
                fetch_load = false;
            } else if fetch_exec {
                options.execlines.push(argument);
                fetch_exec = false;
            } else {
                match argument.as_ref() {
                    "--load" | "-l" => fetch_load = true,
                    "--silent" | "-s" => options.silent = true,
                    "--quit" | "-q" => options.quit = true,
                    "--eval" | "-e" => fetch_exec = true,
                    "--help" | "-h" | "-?" =>
                        options.showhelp = true,
                    "--script" => {
                        options.silent = true;
                        options.quit = true;
                        fetch_load = true;
                    },
                    _ => return Err(
                        format!("Unknown argument: {}", argument)),
                }
            }
        }
        Ok(options)
    }
}

static TIMESTAMP: &'static str = include_str!(
    concat!(env!("OUT_DIR"), "/timestamp.txt"));

static VERSION: &'static str = include_str!(
    concat!(env!("OUT_DIR"), "/version.txt"));

static TARGET: &'static str = include_str!(
    concat!(env!("OUT_DIR"), "/target.txt"));

fn handle_load_file(options: &ArgsOptions, mut state: &mut MajState) {
    use crate::axioms::primitives::maj_load;
    use crate::axioms::predicates::maj_errorp;
    for file in &options.loadfiles {
        if !options.silent {
            println!("; loading {}...", file);
        }
        let results =
            maj_load(&mut state, Maj::nil(), Maj::string(file));
        if maj_errorp(results.clone()).to_bool() {
            eprintln!("{} {}", "Error:".red().bold(),
                      maj_format(&state, results));
            if options.quit {
                process::exit(1);
            }
        }
    }
}

fn handle_exec_evals(options: &ArgsOptions, mut state: &mut MajState) {
    for codeline in &options.execlines {
        match maj_tokenize(codeline.as_ref()) {
            Ok(tokens) => {
                match maj_parse(&mut state, tokens.clone()) {
                    Ok(expressions) => {
                        let results =
                            maj_eval(&mut state,
                                     Maj::cons(Maj::do_sym(),
                                               expressions),
                                     Maj::nil());
                        println!("{}", maj_format(&state, results)
                                 .cyan());
                    },
                    Err(msg) => {
                        eprintln!("Parser error: {}", msg)
                    },
                }
            },
            Err((line, msg)) =>
                eprintln!("Syntax error on line {}: {}",
                          line, msg),
        }
    }
}

fn show_help(programname: String) {
    if VERSION == "" {
        println!("Built at {}", TIMESTAMP);
    }
    println!("Usage: {} [options]
Options:
\t-l, --load [file]   Load and execute contents of file
\t-s, --silent        Do not show ribbon nor REPL prompt
\t-q, --quit          Halt interpreter execution after executing
\t                    commands given through arguments
\t-e, --eval [text]   Evaluate given string of text
\t-h, -?, --help      Show this help text
\t--script [file]     Same as --silent --quit --load [file]",
             programname);
}

fn main() {
    match ArgsOptions::new() {
        Ok(options) => {
            if !options.silent {
                println!("Majestic Lisp v{} {}",
                         format!("{}{}",
                                 env!("CARGO_PKG_VERSION"),
                                 if VERSION != "" { " nightly" }
                                 else { "" }),
                         TARGET);
                if VERSION != "" {
                    println!("Build {} {}", VERSION, TIMESTAMP);
                }
                println!("Copyright (c) 2020-2022 Lucas S. Vieira");
            }

            if options.showhelp {
                show_help(options.programname);
                process::exit(0);
            }

            let mut state = MajState::new();

            // Load files
            handle_load_file(&options, &mut state);

            // Execute code from console
            handle_exec_evals(&options, &mut state);

            if !options.quit {
                repl(&mut state, &options);
            }
        },
        Err(error) => {
            eprintln!("Arguments error: {}", error);
            show_help(String::from("majestic"));
            process::exit(1);
        }
    }
}
