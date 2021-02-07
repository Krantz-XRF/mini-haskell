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

use std::ops::Bound::*;
use std::collections::{BTreeSet, BTreeMap, VecDeque, BinaryHeap};
use std::rc::Rc;
use std::cmp::Reverse;

use derivative::Derivative;

use super::*;
use crate::partition_refinement::{Partitions, Part, SetIdx, Element};

type NFAStateSet = BTreeSet<NFAState>;

pub struct DFA {
    state_count: u32,
    input_set: Box<[DFAInput]>,
    transitions: BTreeMap<(DFAState, DFAInput), DFAState>,
    accepted_states: BTreeSet<DFAState>,
}

fn pop_set(q: &mut VecDeque<Part>, p: &Partitions) -> Option<SetIdx> {
    let s = q.front_mut()?.pop_set_according_to(p);
    if q.front().unwrap().is_empty() { q.pop_front(); }
    Some(s)
}

impl DFAState {
    const MIN: DFAState = DFAState(u32::MIN);
    const MAX: DFAState = DFAState(u32::MAX);
}

#[derive(Copy, Clone)]
#[derive(Derivative)]
#[derivative(Ord, PartialOrd, Eq, PartialEq)]
struct OrdIter<I: Iterator> where I::Item: Ord {
    head: I::Item,
    #[derivative(Ord = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(PartialEq = "ignore")]
    tail: I,
}

impl<I: Iterator> OrdIter<I> where I::Item: Ord {
    fn new(mut p: I) -> Option<Self> {
        Some(OrdIter { head: p.next()?, tail: p })
    }

    fn pop(self) -> (I::Item, Option<OrdIter<I>>) {
        let OrdIter { head, mut tail } = self;
        (head, tail.next().map(|head| OrdIter { head, tail }))
    }
}

#[must_use]
struct GenericUnion<I>(BinaryHeap<Reverse<OrdIter<I>>>)
    where I: Iterator, I::Item: Ord;

fn generic_union<I>(xss: impl Iterator<Item=I>) -> GenericUnion<I>
    where I: Iterator, I::Item: Ord {
    GenericUnion(xss.flat_map(OrdIter::new).map(Reverse).collect())
}

impl<I> Iterator for GenericUnion<I>
    where I: Iterator, I::Item: Ord {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        let min = self.0.pop()?;
        let (head, tail) = min.0.pop();
        if let Some(tail) = tail {
            self.0.push(Reverse(tail));
        }
        loop {
            match self.0.peek() {
                Some(p) if p.0.head == head => {
                    let x = self.0.pop().unwrap();
                    let (_, tail) = x.0.pop();
                    if let Some(tail) = tail {
                        self.0.push(Reverse(tail));
                    }
                }
                _ => break,
            }
        }
        Some(head)
    }
}

impl DFA {
    pub fn minimize(self) -> DFA {
        let mut reverse_trans = BTreeSet::new();
        for (&(s, a), &t) in &self.transitions {
            reverse_trans.insert((t, a, s));
        }
        let mut pending = VecDeque::new();
        let mut resulting = Partitions::new(self.state_count);
        resulting.refine_with(self.accepted_states.iter().map(|s| s.0))
            .for_each(|p| pending.push_back(resulting[p]));
        while let Some(s) = pop_set(&mut pending, &resulting) {
            for c in self.input_set.iter().copied() {
                // x = delta^-1(c, s)
                let x = generic_union(resulting.set_iter(s).map(|x| {
                    let x = DFAState(x);
                    reverse_trans.range((x, c, DFAState::MIN)..=(x, c, DFAState::MAX)).map(|t| t.2.0)
                }));
                for y in resulting.refine_with(x) {
                    let py = resulting[y];
                    if !pending.iter().any(|z| py.is_subset_of(z)) {
                        pending.push_back(py);
                    }
                }
            }
        }
        resulting.simplify();
        let q0 = resulting.parent_set_of(Element(0));
        resulting.promote_to_head(q0);
        DFA {
            state_count: resulting.set_count() as u32,
            input_set: self.input_set,
            transitions: self.transitions.iter()
                .map(|((s, a), t)|
                    ((DFAState(resulting.parent_set_of(Element(s.0)).unwrap()), *a),
                     DFAState(resulting.parent_set_of(Element(t.0)).unwrap())))
                .collect(),
            accepted_states: self.accepted_states.iter()
                .map(|s| DFAState(resulting.parent_set_of(Element(s.0)).unwrap()))
                .collect(),
        }
    }

    pub fn debug_format(&self) -> Result<String, std::fmt::Error> {
        let mut buffer = String::new();
        use std::fmt::Write;
        writeln!(buffer, r#"digraph {{"#)?;
        writeln!(buffer, r#"  rankdir="LR";"#)?;
        for ((s, a), t) in &self.transitions {
            writeln!(buffer, r#"  {} -> {} [label="{}"];"#, s.0, t.0, a.0)?;
        }
        writeln!(buffer, r#"  start [shape="plaintext"];"#)?;
        writeln!(buffer, r#"  start -> 0;"#)?;
        for f in &self.accepted_states {
            writeln!(buffer, r#"  {} [shape="doublecircle"];"#, f.0)?;
        }
        writeln!(buffer, r#"}}"#)?;
        Ok(buffer)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct Transition {
    departure: NFAState,
    input: NFAInput,
    destination: NFAState,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct DFAState(u32);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct DFAInput(u32);

#[derive(Default)]
struct StateCollector {
    states: BTreeMap<Rc<NFAStateSet>, DFAState>,
    states_to_process: VecDeque<Rc<NFAStateSet>>,
}

impl StateCollector {
    fn add_or_get_state_set(&mut self, s: &Rc<NFAStateSet>) -> DFAState {
        if self.states.contains_key(s) {
            *self.states.get(s).unwrap()
        } else {
            let n = DFAState(self.states.len() as u32);
            self.states.insert(s.clone(), n);
            self.states_to_process.push_back(s.clone());
            n
        }
    }
}

struct Determiner {
    input_set: Box<[u32]>,
    transitions: BTreeSet<Transition>,
}

impl Determiner {
    fn trans_once(&self, s: NFAState, a: NFAInput) -> impl Iterator<Item=NFAState> + '_ {
        let l = Transition { departure: s, input: a, destination: NFAState(0) };
        let r = Transition { departure: s, input: a, destination: NFAState(u32::MAX) };
        self.transitions.range((Included(l), Included(r))).map(|t| t.destination)
    }

    fn transitioned(&self, s: &NFAStateSet, a: NFAInput) -> NFAStateSet {
        self.epsilon_closure(s.iter().flat_map(move |&x| self.trans_once(x, a)))
    }

    fn epsilon_closure_single(&self, x: NFAState) -> NFAStateSet {
        self.epsilon_closure(std::iter::once(x))
    }

    fn epsilon_closure(&self, s: impl IntoIterator<Item=NFAState>) -> NFAStateSet {
        let mut to_insert = s.into_iter().collect::<VecDeque<NFAState>>();
        let mut res = NFAStateSet::new();
        while !to_insert.is_empty() {
            let x = to_insert.pop_front().unwrap();
            if res.insert(x) {
                for y in self.trans_once(x, NFAInput::EPSILON) {
                    to_insert.push_front(y);
                }
            }
        }
        res
    }

    fn determine(&mut self, m: NFA) -> DFA {
        let mut new_transitions = BTreeMap::new();
        let mut states = StateCollector::default();
        let start = Rc::new(self.epsilon_closure_single(m.start));
        let _ = states.add_or_get_state_set(&start);
        while !states.states_to_process.is_empty() {
            let s = states.states_to_process.pop_front().unwrap();
            let ns = *states.states.get(&s).unwrap();
            for a in self.input_set.iter().copied() {
                let t = Rc::new(self.transitioned(&s, NFAInput::new(a)));
                if !t.is_empty() {
                    let nt = states.add_or_get_state_set(&t);
                    new_transitions.insert((ns, DFAInput(a)), nt);
                }
            }
        }
        DFA {
            state_count: states.states.len() as u32,
            transitions: new_transitions,
            accepted_states: states.states.iter()
                .filter(|s| s.0.contains(&m.accepted))
                .map(|s| s.1).copied().collect(),
            input_set: self.input_set.iter().copied().map(DFAInput).collect::<Vec<_>>().into_boxed_slice(),
        }
    }
}

impl Builder {
    pub fn finish(self, m: NFA) -> DFA {
        Determiner {
            input_set: {
                let mut xs = self.transitions.iter()
                    .flat_map(|e| e.input.0)
                    .collect::<Vec<_>>();
                xs.sort_unstable();
                xs.dedup();
                xs.into_boxed_slice()
            },
            transitions: self.transitions.iter()
                .map(|&Edge { departure, destination, input }|
                    Transition { departure, input, destination })
                .collect(),
        }.determine(m)
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
    fn test_builder_finish() {
        let e: Expr = parse_quote!(('a'..'f' | 'A'..'F' | '_') ('0'..'9' | 'a'..'f' | 'A'..'F' | '_')*);
        let mut builder = Builder::new();
        let r: RegEx<UnicodeCharClass> = e.try_into().unwrap();
        let (cls, r) = r.classify_chars();
        let m = builder.build(r);
        let n = builder.finish(m);
        assert_eq!(cls, vec![0, 48, 58, 65, 71, 95, 96, 97, 103, 1114112]);
        assert_eq!(
            n.debug_format().unwrap(),
            indoc!(r#"
                digraph {
                  rankdir="LR";
                  0 -> 1 [label="3"];
                  0 -> 2 [label="5"];
                  0 -> 3 [label="7"];
                  1 -> 4 [label="1"];
                  1 -> 5 [label="3"];
                  1 -> 6 [label="5"];
                  1 -> 7 [label="7"];
                  2 -> 4 [label="1"];
                  2 -> 5 [label="3"];
                  2 -> 6 [label="5"];
                  2 -> 7 [label="7"];
                  3 -> 4 [label="1"];
                  3 -> 5 [label="3"];
                  3 -> 6 [label="5"];
                  3 -> 7 [label="7"];
                  4 -> 4 [label="1"];
                  4 -> 5 [label="3"];
                  4 -> 6 [label="5"];
                  4 -> 7 [label="7"];
                  5 -> 4 [label="1"];
                  5 -> 5 [label="3"];
                  5 -> 6 [label="5"];
                  5 -> 7 [label="7"];
                  6 -> 4 [label="1"];
                  6 -> 5 [label="3"];
                  6 -> 6 [label="5"];
                  6 -> 7 [label="7"];
                  7 -> 4 [label="1"];
                  7 -> 5 [label="3"];
                  7 -> 6 [label="5"];
                  7 -> 7 [label="7"];
                  start [shape="plaintext"];
                  start -> 0;
                  1 [shape="doublecircle"];
                  2 [shape="doublecircle"];
                  3 [shape="doublecircle"];
                  4 [shape="doublecircle"];
                  5 [shape="doublecircle"];
                  6 [shape="doublecircle"];
                  7 [shape="doublecircle"];
                }
            "#)
        );
        let n = n.minimize();
        assert_eq!(
            n.debug_format().unwrap(),
            indoc!(r#"
                digraph {
                  rankdir="LR";
                  0 -> 1 [label="3"];
                  0 -> 1 [label="5"];
                  0 -> 1 [label="7"];
                  1 -> 1 [label="1"];
                  1 -> 1 [label="3"];
                  1 -> 1 [label="5"];
                  1 -> 1 [label="7"];
                  start [shape="plaintext"];
                  start -> 0;
                  1 [shape="doublecircle"];
                }
            "#)
        );
    }
}
