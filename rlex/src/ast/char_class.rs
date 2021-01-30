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

//! Unicode character classes.

use std::fmt::{Display, Formatter};
use crate::ast::op::Pretty;

/// Unicode character ranges, inclusive for `begin`, exclusive for `end`.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct UnicodeCharRange {
    pub(super) begin: u32,
    pub(super) end: u32,
}

impl Display for UnicodeCharRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.begin + 1 == self.end {
            write!(f, "{}", unsafe { std::char::from_u32_unchecked(self.begin) })
        } else {
            write!(f, "[{}-{}]",
                   unsafe { std::char::from_u32_unchecked(self.begin) },
                   unsafe { std::char::from_u32_unchecked(self.end - 1) })
        }
    }
}

pub struct EndPointIter {
    range: [u32; 2],
    index: usize,
}

impl Iterator for EndPointIter {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= 2 { return None; }
        let res = Some(self.range[self.index]);
        self.index += 1;
        res
    }
}

impl UnicodeCharRange {
    /// Create a new [`UnicodeCharRange`], both endpoints included.
    pub fn new(begin: char, end: char) -> Self {
        UnicodeCharRange {
            begin: begin as u32,
            end: end as u32 + 1,
        }
    }

    /// Create a new [`UnicodeCharRange`], `end` excluded.
    pub fn new_exclusive(begin: char, end: char) -> Self {
        UnicodeCharRange {
            begin: begin as u32,
            end: end as u32,
        }
    }

    /// Iterate through both end points.
    pub fn end_points(&self) -> EndPointIter {
        EndPointIter { range: [self.begin, self.end], index: 0 }
    }

    /// Create a new [`UnicodeCharRange`], both endpoints included.
    pub fn from_raw(begin: u32, end: u32) -> Self {
        UnicodeCharRange { begin, end }
    }
}

impl From<char> for UnicodeCharRange {
    fn from(c: char) -> Self {
        UnicodeCharRange::new(c, c)
    }
}

impl From<std::ops::Range<char>> for UnicodeCharRange {
    fn from(r: std::ops::Range<char>) -> Self {
        UnicodeCharRange::new_exclusive(r.start, r.end)
    }
}

impl From<std::ops::RangeInclusive<char>> for UnicodeCharRange {
    fn from(r: std::ops::RangeInclusive<char>) -> Self {
        UnicodeCharRange::new(*r.start(), *r.end())
    }
}

impl From<std::ops::RangeFrom<char>> for UnicodeCharRange {
    fn from(r: std::ops::RangeFrom<char>) -> Self {
        UnicodeCharRange::from(r.start..='\u{10FFFF}')
    }
}

impl From<std::ops::RangeTo<char>> for UnicodeCharRange {
    fn from(r: std::ops::RangeTo<char>) -> Self {
        UnicodeCharRange::from('\0'..r.end)
    }
}

impl From<std::ops::RangeToInclusive<char>> for UnicodeCharRange {
    fn from(r: std::ops::RangeToInclusive<char>) -> Self {
        UnicodeCharRange::from('\0'..=r.end)
    }
}

/// Unicode character class.
pub struct UnicodeCharClass {
    intervals: Vec<UnicodeCharRange>,
}

impl Display for UnicodeCharClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let no_bracket = self.intervals.len() == 1 &&
            self.intervals[0].begin + 1 != self.intervals[0].end;
        if !no_bracket { write!(f, "[")?; }
        for r in self.intervals.iter() {
            write!(f, "{}", r)?;
        }
        if !no_bracket { write!(f, "]")?; }
        Ok(())
    }
}

impl Pretty for UnicodeCharClass {
    type Context = ();
    fn pretty_fmt(&self, f: &mut Formatter<'_>, _: ()) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl UnicodeCharClass {
    pub fn empty() -> Self { UnicodeCharClass { intervals: Vec::new() } }
    pub fn from_sorted(intervals: Vec<UnicodeCharRange>) -> Self { UnicodeCharClass { intervals } }
    pub fn iter(&self) -> impl Iterator<Item=&UnicodeCharRange> { self.intervals.iter() }
}

impl IntoIterator for UnicodeCharClass {
    type Item = UnicodeCharRange;
    type IntoIter = <Vec<UnicodeCharRange> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.intervals.into_iter()
    }
}

impl From<char> for UnicodeCharClass {
    fn from(c: char) -> Self { UnicodeCharClass { intervals: vec![c.into()] } }
}

impl From<UnicodeCharRange> for UnicodeCharClass {
    fn from(r: UnicodeCharRange) -> Self { UnicodeCharClass { intervals: vec![r] } }
}

impl From<Vec<UnicodeCharRange>> for UnicodeCharClass {
    fn from(mut intervals: Vec<UnicodeCharRange>) -> Self {
        intervals.sort();
        if intervals.is_empty() { return UnicodeCharClass { intervals }; }
        let mut next_to_write = 1;
        for i in 1..intervals.len() {
            let last = next_to_write - 1;
            if intervals[last].end >= intervals[i].begin {
                intervals[last] = UnicodeCharRange {
                    begin: intervals[last].begin,
                    end: std::cmp::max(intervals[last].end, intervals[i].end),
                }
            } else {
                intervals[next_to_write] = intervals[i];
                next_to_write += 1
            }
        }
        intervals.truncate(next_to_write);
        UnicodeCharClass { intervals }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_display(x: impl Display, s: &str) {
        assert_eq!(format!("{}", x), s)
    }

    #[test]
    fn test_range() {
        assert_display(UnicodeCharRange::new('a', 'z'), "[a-z]");
        assert_display(UnicodeCharRange::new_exclusive('a', 'z'), "[a-y]");
        assert_display(UnicodeCharRange::from('a'..'z'), "[a-y]");
        assert_display(UnicodeCharRange::from('a'..), "[a-\u{10FFFF}]");
        assert_display(UnicodeCharRange::from('a'..='z'), "[a-z]");
        assert_display(UnicodeCharRange::from(..'z'), "[\0-y]");
        assert_display(UnicodeCharRange::from(..='z'), "[\0-z]");
    }

    #[test]
    fn test_class() {
        assert_display(UnicodeCharClass::from(
            vec![
                UnicodeCharRange::new('2', '3'),
                UnicodeCharRange::new('0', '5'),
                UnicodeCharRange::new('a', 'z'),
                UnicodeCharRange::new('A', 'Z'),
                UnicodeCharRange::new('6', '6'),
            ]
        ), "[[0-6][A-Z][a-z]]");
    }
}
