pub mod grammar;
pub mod parser;

use grammar::SymbolType::{NT, T};
use grammar::{create_non_terminal_set, create_rule_set, create_terminal_set, Grammar};

fn main() -> Result<(), String> {
    let non_terminals = create_non_terminal_set(vec!["S", "NP", "VP", "PP", "N", "V", "P"]);
    let terminals = create_terminal_set(vec!["can", "fish", "rivers", "they", "in", "December"]);

    let rules = create_rule_set(vec![
        ("S", vec![("NP", NT), ("VP", NT)]),
        ("NP", vec![("N", NT), ("PP", NT)]),
        ("NP", vec![("N", NT)]),
        ("PP", vec![("P", NT), ("NP", NT)]),
        ("VP", vec![("VP", NT), ("PP", NT)]),
        ("VP", vec![("V", NT), ("VP", NT)]),
        ("VP", vec![("V", NT), ("NP", NT)]),
        ("VP", vec![("V", NT)]),
        ("N", vec![("can", T)]),
        ("N", vec![("fish", T)]),
        ("N", vec![("rivers", T)]),
        ("N", vec![("they", T)]),
        ("N", vec![("December", T)]),
        ("P", vec![("in", T)]),
        ("V", vec![("can", T)]),
        ("V", vec![("fish", T)]),
    ]);

    let grammar = Grammar::new(
        non_terminals,
        terminals,
        grammar::Symbol::NonTerminal("S".to_string()),
        rules,
    )?;

    let privileged = create_non_terminal_set(vec!["N", "V", "P"]);

    let mut parser = grammar.get_parser(privileged)?;

    parser.parse(vec![
        "they", "can", "fish", "in", "rivers", "in", "December",
    ])?;

    Ok(())
}
