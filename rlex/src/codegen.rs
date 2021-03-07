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

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use itertools::Itertools;

use crate::ast::{RootDef, RegEx, Result};
use crate::automata::builder::{Builder, NFA, determine::DFA};
use crate::automata::builder::determine::{DFAState, TaggedDFA};

pub struct TaggedRegEx<Chr, Tag> {
    pub regex: Rc<RegEx<Chr>>,
    pub tag: Tag,
}

pub fn gen_dfa<Tag: Display + Clone>(
    rs: impl IntoIterator<Item=TaggedRegEx<Vec<u32>, Tag>>, chars: &[u32],
) -> Result<TaggedDFA<Tag>> {
    let mut builder = Builder::new();
    let mut tags = HashMap::new();
    let mut ms = Vec::new();
    for r in rs {
        let m = builder.build(&r.regex);
        tags.insert(m.accepted, r.tag);
        ms.push(m);
    }
    let m = builder.alt(ms);
    let mut m = builder.finish(m);
    let mut acc_class = Vec::new();
    let mut tagged_states: Result<HashMap<DFAState, Tag>> = Ok(HashMap::new());
    for (t, ns) in std::mem::take(&mut m.accepted_states) {
        let ns = ns.into_iter()
            .filter(|x| tags.contains_key(x))
            .collect::<Vec<_>>();
        if ns.is_empty() { continue; }
        let this = if ns.len() == 1 {
            let s = ns.into_iter().next().unwrap();
            acc_class.push((s, t));
            Ok(tags.get(&s).unwrap())
        } else {
            Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    r#"lexeme types [{}] conflict on input "{}"."#,
                    ns.iter().map(|n| tags.get(n).unwrap()).format(", "),
                    m.name_an_input_for(t).into_iter()
                        .map(|a| std::char::from_u32(chars[a.0 as usize])
                            .unwrap().escape_debug())
                        .format("")
                ),
            ))
        };
        match (&mut tagged_states, this) {
            (Ok(res), Ok(x)) => {
                let inserted = res.insert(t, x.clone()).is_none();
                assert!(inserted, "mapping should be unique.")
            }
            (Ok(_), Err(e)) => tagged_states = Err(e),
            (Err(e_res), Err(e)) => e_res.combine(e),
            (Err(_), Ok(_)) => (),
        }
    }
    acc_class.sort_unstable();
    let acc_class = acc_class.into_iter().group_by(|t| t.0);
    let acc_class = acc_class.into_iter().map(|g| g.1.map(|t| t.1));
    Ok(TaggedDFA {
        state_count: m.state_count,
        input_set: m.input_set,
        transitions: m.transitions,
        accepted_states: tagged_states?,
    }.minimize_with(acc_class))
}
