/*
 * mini-haskell: light-weight Haskell for fun
 * Copyright (C) 2021  Xie Ruifeng
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

mod determine;

use std::collections::BTreeSet;
use crate::ast::{RegEx, RegOp};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Edge {
    departure: NFAState,
    destination: NFAState,
    input: NFAInput,
}

impl Edge {
    pub fn new(departure: NFAState, destination: NFAState, input: NFAInput) -> Self {
        Edge { departure, destination, input }
    }
}

pub struct Builder {
    next_available_state: u32,
    transitions: BTreeSet<Edge>,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct NFAState(u32);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct NFAInput(Option<u32>);

impl NFAInput {
    pub const EPSILON: NFAInput = NFAInput(None);
    pub const fn new(x: u32) -> Self { NFAInput(Some(x)) }
}

pub struct NFA {
    start: NFAState,
    accepted: NFAState,
}

impl Builder {
    pub fn new() -> Self {
        Builder { next_available_state: 0, transitions: BTreeSet::new() }
    }

    fn state(&mut self) -> NFAState {
        let res = NFAState(self.next_available_state);
        self.next_available_state += 1;
        res
    }

    fn add_arc(&mut self, s: NFAState, t: NFAState, a: NFAInput) -> bool {
        self.transitions.insert(Edge::new(s, t, a))
    }

    fn new_arc(&mut self, s: NFAState, t: NFAState, a: NFAInput) {
        assert!(self.add_arc(s, t, a), "transition {:?}({:?} -> {:?}) already exists.", a, s, t)
    }

    fn new_nfa(&mut self, f: impl FnOnce(&mut Builder, NFAState, NFAState)) -> NFA {
        let s = self.state();
        let t = self.state();
        f(self, s, t);
        NFA { start: s, accepted: t }
    }

    pub fn atom(&mut self, xs: &[u32]) -> NFA {
        self.new_nfa(|this, s, t| for x in xs {
            this.new_arc(s, t, NFAInput::new(*x))
        })
    }

    pub fn alt(&mut self, ms: impl Iterator<Item=NFA>) -> NFA {
        self.new_nfa(|this, s, t| for m in ms {
            this.new_arc(s, m.start, NFAInput::EPSILON);
            this.new_arc(m.accepted, t, NFAInput::EPSILON);
        })
    }

    pub fn concat(&mut self, ms: impl Iterator<Item=NFA>) -> NFA {
        self.new_nfa(|this, s, t| {
            let mut last = s;
            for m in ms {
                this.new_arc(last, m.start, NFAInput::EPSILON);
                last = m.accepted;
            }
            this.new_arc(last, t, NFAInput::EPSILON);
        })
    }

    pub fn some(&mut self, m: NFA) -> NFA {
        self.add_arc(m.accepted, m.start, NFAInput::EPSILON);
        m
    }

    pub fn optional(&mut self, m: NFA) -> NFA {
        self.new_nfa(|this, s, t| {
            this.add_arc(s, t, NFAInput::EPSILON);
            this.add_arc(s, m.start, NFAInput::EPSILON);
            this.add_arc(m.accepted, t, NFAInput::EPSILON);
        })
    }

    pub fn build(&mut self, regex: RegEx<Vec<u32>>) -> NFA {
        regex.fold(&mut |op| match op {
            RegOp::Atom(a) => self.atom(&a),
            RegOp::Alt(rs) => self.alt(rs.into_iter()),
            RegOp::Concat(rs) => self.concat(rs.into_iter()),
            RegOp::Some(r) => self.some(*r),
            RegOp::Optional(r) => self.optional(*r),
        })
    }

    pub fn debug_format_nfa(&self, n: &NFA) -> Result<String, std::fmt::Error> {
        let mut buffer = String::new();
        use std::fmt::Write;
        writeln!(buffer, r#"digraph {{"#)?;
        writeln!(buffer, r#"  rankdir="LR";"#)?;
        for e in &self.transitions {
            let &Edge { departure: s, destination: t, input: a } = e;
            let a = a.0.map_or("ε".to_string(), |c| c.to_string());
            writeln!(buffer, r#"  {} -> {} [label="{}"];"#, s.0, t.0, a)?;
        }
        writeln!(buffer, r#"  start [shape="plaintext"];"#)?;
        writeln!(buffer, r#"  start -> {};"#, n.start.0)?;
        writeln!(buffer, r#"  {} [shape="doublecircle"];"#, n.accepted.0)?;
        writeln!(buffer, r#"}}"#)?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use syn::parse_quote;
    use indoc::indoc;

    use super::*;
    use crate::syntax::Expr;
    use crate::ast::UnicodeCharClass;

    #[test]
    fn test_nfa_builder() {
        let e: Expr = parse_quote!(('a'..'f' | 'A'..'F' | '_') ('0'..'9' | 'a'..'f' | 'A'..'F' | '_')+);
        let mut builder = Builder::new();
        let r: RegEx<UnicodeCharClass> = e.try_into().unwrap();
        let (cls, r) = r.classify_chars();
        let m = builder.build(r);
        assert_eq!(cls, vec![0, 48, 58, 65, 71, 95, 96, 97, 103, 1114112]);
        assert_eq!(
            builder.debug_format_nfa(&m).unwrap(),
            indoc!(r#"
                digraph {
                  rankdir="LR";
                  0 -> 1 [label="7"];
                  1 -> 7 [label="ε"];
                  2 -> 3 [label="3"];
                  3 -> 7 [label="ε"];
                  4 -> 5 [label="5"];
                  5 -> 7 [label="ε"];
                  6 -> 0 [label="ε"];
                  6 -> 2 [label="ε"];
                  6 -> 4 [label="ε"];
                  7 -> 16 [label="ε"];
                  8 -> 9 [label="1"];
                  9 -> 17 [label="ε"];
                  10 -> 11 [label="7"];
                  11 -> 17 [label="ε"];
                  12 -> 13 [label="3"];
                  13 -> 17 [label="ε"];
                  14 -> 15 [label="5"];
                  15 -> 17 [label="ε"];
                  16 -> 8 [label="ε"];
                  16 -> 10 [label="ε"];
                  16 -> 12 [label="ε"];
                  16 -> 14 [label="ε"];
                  17 -> 16 [label="ε"];
                  17 -> 19 [label="ε"];
                  18 -> 6 [label="ε"];
                  start [shape="plaintext"];
                  start -> 18;
                  19 [shape="doublecircle"];
                }
            "#)
        );
    }
}
