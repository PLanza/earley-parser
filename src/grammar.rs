use std::collections::HashSet;

use crate::parser::Parser;

#[derive(Hash, Clone, Debug)]
pub enum Symbol {
    NonTerminal(String),
    Terminal(String),
}

pub enum SymbolType {
    NT,
    T,
}

pub type Rule = (Symbol, Vec<Symbol>);

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Symbol::NonTerminal(x) => match other {
                Symbol::NonTerminal(y) => x.eq(y),
                Symbol::Terminal(_) => false,
            },
            Symbol::Terminal(x) => match other {
                Symbol::NonTerminal(_) => false,
                Symbol::Terminal(y) => x.eq(y),
            },
        }
    }
}
impl Eq for Symbol {}

// Creates a set of non-terminals from a vector of strings
pub fn create_non_terminal_set(non_terminals: Vec<&str>) -> HashSet<Symbol> {
    let mut set: HashSet<Symbol> = HashSet::new();
    for non_term in non_terminals {
        set.insert(Symbol::NonTerminal(non_term.to_string()));
    }

    set
}

// Creates a set of terminals from a vector of strings
pub fn create_terminal_set(terminals: Vec<&str>) -> HashSet<Symbol> {
    let mut set: HashSet<Symbol> = HashSet::new();
    for term in terminals {
        set.insert(Symbol::Terminal(term.to_string()));
    }

    set
}

// Creates a set of rules from a vector of pairs of a non-terminal string, and
// a vector of pairs of symbol strings and their corresponding type
// S -> NP VP => ("S", vec![("NP", NT), ("VP", NT)])
pub fn create_rule_set(rules: Vec<(&str, Vec<(&str, SymbolType)>)>) -> HashSet<Rule> {
    let mut set: HashSet<Rule> = HashSet::new();
    for rule in rules {
        let mut right_side: Vec<Symbol> = Vec::new();
        for symbol in rule.1 {
            match symbol.1 {
                SymbolType::NT => right_side.push(Symbol::NonTerminal(symbol.0.to_string())),
                SymbolType::T => right_side.push(Symbol::Terminal(symbol.0.to_string())),
            }
        }

        set.insert((Symbol::NonTerminal(rule.0.to_string()), right_side));
    }

    set
}

#[derive(Debug)]
pub struct Grammar {
    non_terminals: HashSet<Symbol>,
    terminals: HashSet<Symbol>,
    start: Symbol,
    rules: HashSet<Rule>,
}

impl Grammar {
    pub fn new(
        non_terminals: HashSet<Symbol>,
        terminals: HashSet<Symbol>,
        start: Symbol,
        rules: HashSet<Rule>,
    ) -> Result<Grammar, String> {
        if !non_terminals.contains(&start) {
            return Err("Starting symbol not in the set of non-terminals".to_string());
        }

        for rule in &rules {
            if !non_terminals.contains(&rule.0) {
                return Err("Left side of rule is not a non-terminal".to_string());
            }
            for symbol in &rule.1 {
                if !non_terminals.contains(&symbol) && !terminals.contains(&symbol) {
                    return Err(
                        "Symbol on right side of rule is neither a terminal, nor a non-terminal"
                            .to_string(),
                    );
                }
            }
        }

        Ok(Grammar {
            non_terminals,
            terminals,
            start,
            rules,
        })
    }

    pub fn get_parser(&self, privileged: HashSet<Symbol>) -> Result<Parser, String> {
        Parser::new(self, privileged)
    }

    pub fn get_starting_rule(&self) -> Result<Rule, String> {
        for rule in &self.rules {
            if rule.0 == self.start {
                return Ok(rule.clone());
            }
        }
        Err("No rule reducing the starting symbol".to_string())
    }

    pub fn get_rules(&self, symbol: Symbol) -> Vec<Rule> {
        let mut rules = Vec::new();

        for rule in &self.rules {
            if rule.0 == symbol {
                rules.push(rule.clone());
            }
        }

        rules
    }

    pub fn in_terminals(&self, symbol: &Symbol) -> bool {
        self.terminals.contains(symbol)
    }

    pub fn in_non_terminals(&self, symbol: &Symbol) -> bool {
        self.non_terminals.contains(symbol)
    }

    pub fn get_terminal_rule(&self, non_term: &Symbol, terminal: &Symbol) -> Option<Rule> {
        for rule in &self.rules {
            if &rule.0 == non_term && rule.1.len() == 1 {
                if &rule.1[0] == terminal {
                    return Some(rule.clone());
                }
            }
        }

        None
    }
}
