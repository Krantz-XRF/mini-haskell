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

use std::ops::{Range, Index, IndexMut};
use std::collections::HashMap;
use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug = "transparent")]
#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct SetIdx(u32);

impl SetIdx {
    pub fn unwrap(self) -> u32 { self.0 }
}

#[derive(Derivative)]
#[derivative(Debug = "transparent")]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct BufferIdx(u32);

#[derive(Derivative)]
#[derivative(Debug = "transparent")]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Element(pub u32);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Part { start: BufferIdx, end: BufferIdx }

impl Part {
    fn as_range(self) -> Range<usize> { self.start.0 as usize..self.end.0 as usize }
    fn len(self) -> u32 { self.end.0 - self.start.0 }
    pub fn is_empty(self) -> bool { self.start == self.end }
    pub fn pop_set_according_to(&mut self, p: &Partitions) -> SetIdx {
        let a = p.back_buffer[self.start.0 as usize];
        let s = p.parent_set_of(a);
        self.start = p.partitions[s.0 as usize].end;
        s
    }
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.start >= other.start && self.end <= other.end
    }
}

pub struct Partitions {
    back_buffer: Vec<Element>,
    parent_set: Vec<SetIdx>,
    positions: Vec<BufferIdx>,
    partitions: Vec<Part>,
}

impl Index<SetIdx> for Partitions {
    type Output = Part;
    fn index(&self, index: SetIdx) -> &Part {
        &self.partitions[index.0 as usize]
    }
}

impl IndexMut<SetIdx> for Partitions {
    fn index_mut(&mut self, index: SetIdx) -> &mut Part {
        &mut self.partitions[index.0 as usize]
    }
}

impl Partitions {
    pub fn new(n: u32) -> Self {
        Partitions {
            back_buffer: (0..n).map(Element).collect(),
            parent_set: vec![SetIdx(0); n as usize],
            positions: (0..n).map(BufferIdx).collect(),
            partitions: vec![Part { start: BufferIdx(0), end: BufferIdx(n) }],
        }
    }

    pub fn simplify(&mut self) {
        let n = self.partitions.len();
        let mut idx_map = vec![0; n];
        let mut next_to_write = 0;
        #[allow(clippy::needless_range_loop)]
        for k in 0..n {
            if self.partitions[k].is_empty() { continue; }
            idx_map[k] = next_to_write as u32;
            self.partitions.swap(k, next_to_write);
            next_to_write += 1;
        }
        for s in &mut self.parent_set {
            *s = SetIdx(idx_map[s.0 as usize]);
        }
    }

    pub fn promote_to_head(&mut self, n: SetIdx) {
        let p0 = self[SetIdx(0)];
        let pn = self[n];
        self.partitions.swap(0, n.0 as usize);
        for i in p0.as_range() {
            self.parent_set[self.back_buffer[i].0 as usize] = n;
        }
        for i in pn.as_range() {
            self.parent_set[self.back_buffer[i].0 as usize] = SetIdx(0);
        }
    }

    #[cfg(test)]
    pub fn sets(&self) -> impl Iterator<Item=impl Iterator<Item=u32> + '_> + '_ {
        self.partitions.iter().map(move |p|
            self.back_buffer[p.as_range()].iter().map(|e| e.0))
    }

    pub fn set_count(&self) -> usize { self.partitions.len() }

    pub fn parent_set_of(&self, e: Element) -> SetIdx {
        self.parent_set[e.0 as usize]
    }

    fn position_of(&self, e: Element) -> BufferIdx {
        self.positions[e.0 as usize]
    }

    pub fn set_iter(&self, s: SetIdx) -> impl Iterator<Item=u32> + '_ {
        let rng = self[s].as_range();
        self.back_buffer[rng].iter().map(|e| e.0)
    }

    pub fn refine_with(&mut self, s: impl Iterator<Item=u32>) -> impl Iterator<Item=SetIdx> {
        let s = s.map(Element);
        let mut affected = HashMap::new();
        for x in s {
            let parent = self.parent_set_of(x);
            let rng_end = &mut self[parent].end;
            // record 'parent' set is affected
            affected.entry(parent).or_insert(*rng_end);
            // shrink 'parent' set
            rng_end.0 -= 1;
            // swap 'x' out of 'parent'
            let rng_end = rng_end.0 as usize;
            let pos = self.position_of(x).0 as usize;
            assert_eq!(self.back_buffer[pos], x);
            let element_end = self.back_buffer[rng_end];
            self.back_buffer.swap(pos, rng_end);
            self.positions.swap(x.0 as usize, element_end.0 as usize);
        }
        let mut newly_formed = Vec::new();
        for (a, new_a_end) in affected {
            // form a new set 'new_a'
            let new_a = SetIdx(self.partitions.len() as u32);
            let new_a_rng = Part { start: self[a].end, end: new_a_end };
            self.partitions.push(new_a_rng);
            // record set pair '(a, new_a)' if a != {}
            let a_rng = self[a];
            if a_rng.len() > new_a_rng.len() {
                newly_formed.push(new_a);
            } else {
                newly_formed.push(a);
            }
            // update parents for elements in 'new_a'
            for i in new_a_rng.as_range() {
                let e = self.back_buffer[i];
                self.parent_set[e.0 as usize] = new_a;
            }
        }
        newly_formed.into_iter()
    }

    #[cfg(test)]
    pub fn dump(&self) {
        let mut last = 0;
        while last < self.back_buffer.len() {
            if last != 0 { eprint!(" ") }
            let p = self.parent_set_of(self.back_buffer[last]);
            eprint!("[");
            let mut pt = self[p].as_range();
            last = pt.end;
            if let Some(i) = pt.next() {
                eprint!("{}", self.back_buffer[i].0);
                for i in pt {
                    eprint!(" {}", self.back_buffer[i].0);
                }
            }
            eprint!("]");
        }
        eprintln!();
    }
}
