use crate::grammar::Symbol::{NonTerminal, Terminal};
use crate::grammar::{Grammar, Rule, Symbol};

use std::collections::HashSet;

#[derive(Debug, Clone)]
struct DottedRule {
    rule: Rule,
    dot_pos: usize,
}

impl PartialEq for DottedRule {
    fn eq(&self, other: &Self) -> bool {
        self.rule == other.rule && self.dot_pos == other.dot_pos
    }
}
impl Eq for DottedRule {}

#[derive(Debug, Clone)]
struct Edge {
    d_rule: DottedRule,
    span: (usize, usize),
    history: Vec<usize>,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.d_rule == other.d_rule && self.span == other.span
    }
}
impl Eq for Edge {}

// Vec index serves as Edge id
type Chart = Vec<Edge>;

pub struct Parser<'g> {
    chart: Chart,
    grammar: &'g Grammar,
    privileged: HashSet<Symbol>,
    starting_rule: Rule,
}

impl<'g> Parser<'g> {
    pub fn new(grammar: &Grammar, privileged: HashSet<Symbol>) -> Result<Parser, String> {
        // Check that the set of privileged instructions is a subset of the grammar's non-terminals
        for non_term in &privileged {
            if !grammar.in_non_terminals(&non_term) {
                return Err("Privileged non-terminal, not in the set of non-terminals".to_string());
            }
        }

        let starting_rule = grammar.get_starting_rule()?;

        let mut chart = Chart::new();

        // Initialize the chart
        chart.push(Edge {
            d_rule: DottedRule {
                rule: starting_rule.clone(),
                dot_pos: 0,
            },
            span: (0, 0),
            history: Vec::new(),
        });

        Ok(Parser {
            chart,
            grammar,
            privileged,
            starting_rule,
        })
    }

    pub fn parse(&mut self, input: Vec<&str>) -> Result<(), String> {
        let mut input_terminals = Vec::new();
        // First check that the input is comprised of terminals in the grammar and convert them
        for string in &input {
            let terminal = Terminal(string.to_string());
            if !self.grammar.in_terminals(&terminal) {
                return Err("Input symbol not in grammar's terminals".to_string());
            }
            input_terminals.push(terminal);
        }

        while self.get_end_edges(&input_terminals).is_empty() {
            self.predict();
            self.scan(&input_terminals);
            self.cont();
        }

        for edge in self.get_end_edges(&input_terminals) {
            self.print_parse_tree(edge);
            println!("\n")
        }

        Ok(())
    }

    fn get_end_edges(&mut self, input: &Vec<Symbol>) -> Vec<Edge> {
        let end = Edge {
            d_rule: DottedRule {
                rule: self.starting_rule.clone(),
                dot_pos: self.starting_rule.1.len(),
            },
            span: (0, input.len()),
            history: Vec::new(),
        };

        let mut end_edges = Vec::new();

        for edge in self.chart.iter() {
            if edge == &end {
                end_edges.push(edge.clone());
            }
        }

        end_edges
    }

    fn predict(&mut self) {
        let mut new_edges = Vec::new();
        for edge in self.chart.iter() {
            // If the dot is at the RHS of the rule there is nothing to expand
            if edge.d_rule.rule.1.len() == edge.d_rule.dot_pos {
                continue;
            }

            // Do not expand any privileged instructions so that the chart doesn't get filled with
            // rules like N -> they, or V -> fish
            if self
                .privileged
                .contains(&edge.d_rule.rule.1[edge.d_rule.dot_pos])
            {
                continue;
            }

            // If the dot is in front of a non-terminal,
            match &edge.d_rule.rule.1[edge.d_rule.dot_pos] {
                NonTerminal(x) => {
                    // Expand it all the possible productions from the non-terminal
                    for rule in self.grammar.get_rules(NonTerminal(x.clone())) {
                        let new_edge = Edge {
                            d_rule: DottedRule { rule, dot_pos: 0 },
                            span: (edge.span.1, edge.span.1),
                            history: Vec::new(),
                        };

                        // Do not add duplicate edges
                        if !self.chart.contains(&new_edge) && !new_edges.contains(&new_edge) {
                            new_edges.push(new_edge)
                        }
                    }
                }
                _ => (),
            }
        }

        // Update chart with new edges
        new_edges.into_iter().for_each(|e| self.chart.push(e));
    }

    fn scan(&mut self, input: &Vec<Symbol>) {
        let mut new_edges = Vec::new();
        for edge in self.chart.iter() {
            let rule = &edge.d_rule.rule;

            // If the dot is at the RHS of the rule there is nothing to scan
            if rule.1.len() == edge.d_rule.dot_pos {
                continue;
            }

            // If the span is at the end of the input then there is nothing to scan
            if edge.span.1 >= input.len() {
                continue;
            }

            // If edge has dot in front a privileged non-terminal
            if self.privileged.contains(&rule.1[edge.d_rule.dot_pos]) {
                // Check if the privileged non-terminal reduces to the input symbol
                match self
                    .grammar
                    .get_terminal_rule(&rule.1[edge.d_rule.dot_pos], &input[edge.span.1])
                {
                    Some(rule) => {
                        let new_edge = Edge {
                            d_rule: DottedRule { rule, dot_pos: 1 },
                            span: (edge.span.0, edge.span.1 + 1),
                            history: Vec::new(),
                        };

                        // Don't add duplicate edges
                        if !new_edges.contains(&new_edge) && !self.chart.contains(&new_edge) {
                            new_edges.push(new_edge);
                        }
                    }
                    None => (),
                }
            }

            // Shift if the edge's rule has a dot in front of a terminal matching the input symbol
            if &rule.1[edge.d_rule.dot_pos] == &input[edge.span.1] {
                let new_edge = Edge {
                    d_rule: DottedRule {
                        rule: rule.clone(),
                        dot_pos: edge.d_rule.dot_pos + 1,
                    },
                    span: (edge.span.0, edge.span.1 + 1),
                    history: Vec::new(),
                };

                // Don't add duplicate edges
                if !new_edges.contains(&new_edge) && !self.chart.contains(&new_edge) {
                    new_edges.push(new_edge);
                }
            }
        }

        // Update chart with new edges
        new_edges.into_iter().for_each(|e| self.chart.push(e));
    }

    fn cont(&mut self) {
        let mut chart_len = self.chart.len();

        // Loop until there are no new edges being added
        loop {
            let mut new_edges = Vec::new();
            for (i, edge) in self.chart.iter().enumerate() {
                // Get any edge whose rule has the dot at the very RHS
                if edge.d_rule.dot_pos == edge.d_rule.rule.1.len() {
                    for edge_2 in self.chart.iter() {
                        // Get all the edges whose rule has the dot in front of this
                        // in front of the same non-terminal as the completed production above
                        if edge_2.d_rule.dot_pos < edge_2.d_rule.rule.1.len()
                            && edge_2.d_rule.rule.1[edge_2.d_rule.dot_pos] == edge.d_rule.rule.0
                            && edge_2.span.1 == edge.span.0
                        {
                            // Add the completed edge into the new edge's history
                            let mut history = edge_2.history.clone();
                            history.push(i as usize);

                            let new_edge = Edge {
                                d_rule: DottedRule {
                                    rule: edge_2.d_rule.rule.clone(),
                                    dot_pos: edge_2.d_rule.dot_pos + 1,
                                },
                                span: (edge_2.span.0, edge.span.1),
                                history,
                            };

                            // Don't add duplicate edges
                            if !self.chart.contains(&new_edge) {
                                new_edges.push(new_edge);
                            }
                        }
                    }
                }
            }
            // Update chart with new edges
            new_edges.into_iter().for_each(|e| self.chart.push(e));

            // Break if no new edges were added
            if chart_len == self.chart.len() {
                break;
            } else {
                chart_len = self.chart.len();
            }
        }
    }

    fn print_parse_tree(&self, end_edge: Edge) {
        let mut stack = vec![end_edge];

        loop {
            let edge = stack.pop();

            match edge {
                Some(edge) => {
                    println!("{:?}", edge.d_rule.rule);
                    for i in edge.history {
                        stack.push(self.chart[i].clone());
                    }
                }
                None => break,
            }
        }
    }
}
