// Copyright 2016 Joe Wilm, The Alacritty Project Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use bitflags::bitflags;

use crate::ansi::{Color, NamedColor};
use crate::grid;
use crate::index::Column;
use arrayvec::ArrayString;

// Maximum number of zerowidth characters which will be stored per cell.
pub const MAX_ZEROWIDTH_CHARS: usize = 5;
pub const MAX_CELL_LEN: usize = 4 * (1 + MAX_ZEROWIDTH_CHARS);

bitflags! {
    pub struct Flags: u16 {
        const INVERSE           = 0b00_0000_0001;
        const BOLD              = 0b00_0000_0010;
        const ITALIC            = 0b00_0000_0100;
        const UNDERLINE         = 0b00_0000_1000;
        const WRAPLINE          = 0b00_0001_0000;
        const WIDE_CHAR         = 0b00_0010_0000;
        const WIDE_CHAR_SPACER  = 0b00_0100_0000;
        const DIM               = 0b00_1000_0000;
        const DIM_BOLD          = 0b00_1000_0010;
        const HIDDEN            = 0b01_0000_0000;
        const STRIKEOUT         = 0b10_0000_0000;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Cell {
    pub fg: Color,
    pub bg: Color,
    pub flags: Flags,
    pub contents: ArrayString<[u8; MAX_CELL_LEN]>,
}

impl Default for Cell {
    fn default() -> Cell {
        Cell::new(
            ' ',
            Color::Named(NamedColor::Foreground),
            Color::Named(NamedColor::Background),
        )
    }
}

/// Get the length of occupied cells in a line
pub trait LineLength {
    /// Calculate the occupied line length
    fn line_length(&self) -> Column;
}

impl LineLength for grid::Row<Cell> {
    fn line_length(&self) -> Column {
        let mut length = Column(0);

        if self[Column(self.len() - 1)].flags.contains(Flags::WRAPLINE) {
            return Column(self.len());
        }

        for (index, cell) in self[..].iter().rev().enumerate() {
            if cell.contents.as_str() != " " {
                length = Column(self.len() - index);
                break;
            }
        }

        length
    }
}

impl Cell {
    #[inline]
    pub fn bold(&self) -> bool {
        self.flags.contains(Flags::BOLD)
    }

    #[inline]
    pub fn inverse(&self) -> bool {
        self.flags.contains(Flags::INVERSE)
    }

    #[inline]
    pub fn dim(&self) -> bool {
        self.flags.contains(Flags::DIM)
    }

    pub fn new(c: char, fg: Color, bg: Color) -> Cell {
        let mut contents = ArrayString::new();
        contents.push(c);
        Cell {
            contents,
            bg,
            fg,
            flags: Flags::empty(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        (self.contents.as_str() == " " || self.contents.as_str() == "\t")
            && self.bg == Color::Named(NamedColor::Background)
            && !self
                .flags
                .intersects(Flags::INVERSE | Flags::UNDERLINE | Flags::STRIKEOUT)
    }

    #[inline]
    pub fn reset(&mut self, template: &Cell) {
        // memcpy template to self
        *self = *template;
    }

    #[inline]
    pub fn as_str(&self) -> arrayvec::ArrayString<[u8; MAX_CELL_LEN]> {
        self.contents
    }

    #[inline]
    pub fn chars(&self) -> [char; 1 + MAX_ZEROWIDTH_CHARS] {
        let mut out = [' '; 1 + MAX_ZEROWIDTH_CHARS];
        for (i, chr) in self
            .contents
            .as_str()
            .chars()
            .enumerate()
            .take(1 + MAX_ZEROWIDTH_CHARS)
        {
            out[i] = chr;
        }
        out
    }

    #[inline]
    pub fn push_extra(&mut self, c: char) {
        self.contents.push(c);
    }

    #[inline]
    pub fn first_char(&self) -> char {
        self.contents
            .as_str()
            .chars()
            .next()
            .expect("cell should always have at least one char")
    }

    #[inline]
    pub fn set_char(&mut self, chr: char) -> ArrayString<[u8; MAX_CELL_LEN]> {
        let mut contents = ArrayString::new();
        contents.push(chr);
        std::mem::swap(&mut self.contents, &mut contents);
        contents
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, LineLength};

    use crate::grid::Row;
    use crate::index::Column;

    #[test]
    fn line_length_works() {
        let template = Cell::default();
        let mut row = Row::new(Column(10), &template);
        row[Column(5)].set_char('a');

        assert_eq!(row.line_length(), Column(6));
    }

    #[test]
    fn line_length_works_with_wrapline() {
        let template = Cell::default();
        let mut row = Row::new(Column(10), &template);
        row[Column(9)].flags.insert(super::Flags::WRAPLINE);

        assert_eq!(row.line_length(), Column(10));
    }
}

#[cfg(all(test, feature = "bench"))]
mod benches {
    extern crate test;
    use super::Cell;

    #[bench]
    fn cell_reset(b: &mut test::Bencher) {
        b.iter(|| {
            let mut cell = Cell::default();

            for _ in 0..100 {
                cell.reset(test::black_box(&Cell::default()));
            }

            test::black_box(cell);
        });
    }
}
