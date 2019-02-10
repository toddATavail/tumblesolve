//
// board.rs
// Copyright 2019, Todd L Smith.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//

//!
//! ## Board
//!
//! Herein is functionality specific to Tumblestone puzzles, but not solutions
//! to those puzzles.
//!

use std::fmt::{Display, Formatter, Result};
use std::collections::HashMap;
use std::num::ParseIntError;
use std::result;
use std::str::ParseBoolError;
use tokesies::*;

/******************************************************************************
 *                                  Stones.                                   *
 ******************************************************************************/

/// The behavior profile of an arbitrary stone.
pub trait Stone
{
	/// Answer the state of the receiver given the specified board state.
	fn for_board (&self, board: &Board) -> Self;

	/// Answer `true` if the receiver is, by nature, directly removable.
	fn is_removable (&self) -> bool;
}

/// The absence of stoniness. Always represented by `'_'` in input, as `' '` in
/// output.
#[derive(Copy, Clone, Hash, Debug)]
pub struct NoStone;

impl Stone for NoStone
{
	/// Answer a copy of the receiver.
	fn for_board (&self, _board: &Board) -> Self
	{
		*self
	}

	fn is_removable (&self) -> bool
	{
		false
	}
}

impl Display for NoStone
{
	fn fmt (&self, f: &mut Formatter) -> Result
	{
		write!(f, " ")
	}
}

/// An ordinary stone.
#[derive(Copy, Clone, Hash, Debug)]
pub struct OrdinaryStone
{
	/// The character that represents this ordinary stone.
	rep: char,

	/// A bit mask that uniquely represents the color of this ordinary stone.
	/// Exactly 1 bit must be set. Bit masks support [wild stones]. The chosen
	/// representation imposes a 32-color limit on any specific board.
	///
	/// [wild stones]: WildStone
	color: u32
}

impl OrdinaryStone
{
	/// Answer the color of the receiver.
	pub fn color (&self) -> u32
	{
		self.color
	}
}

impl Stone for OrdinaryStone
{
	/// Answer a copy of the receiver.
	fn for_board (&self, _board: &Board) -> Self
	{
		*self
	}

	fn is_removable (&self) -> bool
	{
		true
	}
}

impl Display for OrdinaryStone
{
	fn fmt (&self, f: &mut Formatter) -> Result
	{
		write!(f, "{}", self.rep)
	}
}

/// A survivor stone cannot be removed directly, but automatically disappears
/// when the last stone in its row has been removed.
#[derive(Copy, Clone, Hash, Debug)]
pub struct SurvivorStone;

impl Stone for SurvivorStone
{
	/// Answer a copy of the receiver.
	fn for_board (&self, _board: &Board) -> Self
	{
		*self
	}

	fn is_removable (&self) -> bool
	{
		false
	}
}

impl Display for SurvivorStone
{
	fn fmt (&self, f: &mut Formatter) -> Result
	{
		write!(f, "\u{1b}[38;5;8m#")
	}
}

/// A wild stone has potential to provide one or more colors; all other wild
/// stones lose a color whenever a wild stone is committed to a particular
/// color. A wild stone's color space is a property of the [board], not of the
/// wild stone itself (flyweight pattern). Always represented by `*`.
///
/// [board]: Board
#[derive(Copy, Clone, Hash, Debug)]
pub struct WildStone;

impl Stone for WildStone
{
	/// Answer a copy of the receiver.
	fn for_board (&self, _board: &Board) -> Self
	{
		*self
	}

	fn is_removable (&self) -> bool
	{
		true
	}
}

impl Display for WildStone
{
	fn fmt (&self, f: &mut Formatter) -> Result
	{
		write!(f, "\u{1b}[38;5;5m*")
	}
}

/// A toggle stone cannot be matched directly. It alternately obstructs and
/// permits access to stones above it. Initially open is represented by `'/'`,
/// initially closed is represented by `'+'`.
#[derive(Copy, Clone, Hash, Debug)]
pub struct ToggleStone
{
	/// The phase of the toggle stone, either `0` or `1`. The [turn number] is
	/// added to the phase to determine whether the toggle stone is currently an
	/// obstruction — even indicates that the toggle stone is "open", odd that
	/// it is "closed".
	///
	/// [turn number]: Board::turn
	/// [board]: Board
	phase: u32
}

impl ToggleStone
{
	/// Answer `true` if the receiver is open, `false` otherwise.
	pub fn is_open (&self) -> bool
	{
		self.phase & 1 == 0
	}
}

impl Stone for ToggleStone
{
	/// Answer the state of the receiver given the specified board state.
	fn for_board (&self, board: &Board) -> Self
	{
		ToggleStone { phase: self.phase + board.turn }
	}

	fn is_removable (&self) -> bool
	{
		false
	}
}

impl Display for ToggleStone
{
	/// To display an appropriate representation for a particular board, use
	/// [`for_board`] before rendition.
	///
	/// [`for_board`]: Stone::for_board
	fn fmt (&self, f: &mut Formatter) -> Result
	{
		write!(
			f,
			"{}",
			if self.is_open() { "\u{1b}[38;5;15m/" } else { "\u{1b}[38;5;8m+" })
	}
}

/// An arbitrary stone.
#[derive(Copy, Clone, Hash, Debug)]
pub enum AnyStone
{
	None (NoStone),
	Ordinary (OrdinaryStone),
	Survivor (SurvivorStone),
	Wild (WildStone),
	Toggle (ToggleStone)
}

impl Stone for AnyStone
{
	fn for_board (&self, board: &Board) -> Self
	{
		use self::AnyStone::*;
		match self
		{
			None(s) => None(s.for_board(board)),
			Ordinary(s) => Ordinary(s.for_board(board)),
			Survivor(s) => Survivor(s.for_board(board)),
			Wild(s) => Wild(s.for_board(board)),
			Toggle(s) => Toggle(s.for_board(board))
		}
	}

	fn is_removable (&self) -> bool
	{
		use self::AnyStone::*;
		match self
		{
			None(s) => s.is_removable(),
			Ordinary(s) => s.is_removable(),
			Survivor(s) => s.is_removable(),
			Wild(s) => s.is_removable(),
			Toggle(s) => s.is_removable()
		}
	}
}

impl Display for AnyStone
{
	fn fmt (&self, f: &mut Formatter) -> Result
	{
		use self::AnyStone::*;
		match self
		{
			None(s) => s.fmt(f),
			Ordinary(s) => s.fmt(f),
			Survivor(s) => s.fmt(f),
			Wild(s) => s.fmt(f),
			Toggle(s) => s.fmt(f)
		}
	}
}

/******************************************************************************
 *                                   Board.                                   *
 ******************************************************************************/

/// The default board width.
const DEFAULT_WIDTH: u32 = 5;

/// An `(x,y`) point for locating a [stone] on a [board].
///
/// [stone]: AnyStrong
/// [board]: Board
pub type Point = (u32, u32);

/// The state of the game board during a particular turn.
#[derive(Debug)]
pub struct Board
{
	/// The current turn. This, combined with initial [phase], impacts the
	/// obstructiveness of [toggle stones].
	///
	/// [phase]: ToggleStone::phase
	/// [toggle stones]: ToggleStone
	turn: u32,

	/// The bitwise OR of all active [colors] for [wild stones]. Every wild
	/// stone has the same color space.
	///
	/// [colors]: OrdinaryStone::color
	/// [wild stones]: WildStone
	wild_colors: u32,

	/// `true` if the board is color locked, i.e., it is not permitted to play
	/// two triplets of the same color sequentially, `false` otherwise.
	color_locked: bool,

	/// The point to display highlighted, if any.
	highlight: Option<Point>,

	/// The row stride of the physical board, i.e., the number of [stones] in
	/// any given row.
	///
	/// [stones]: AnyStone
	width: u32,

	/// The column stride of the physical board, i.e., the number of [stones] in
	/// any given column.
	///
	/// [stones]: AnyStone
	height: u32,

	/// The count of removable [stones].
	///
	/// [stones]: AnyStone
	removable_stones: u32,

	/// The physical board, as a single linear vector.
	grid: Vec<AnyStone>,

	/// The property map.
	properties: PropertyMap
}

impl Board
{
	/// Parse a board from the specified string. The string should depict a
	/// legend and a grid. The length of the longest line and the number of
	/// occupied rows establish the width and height, respectively, of the
	/// resultant board.
	///
	/// The string should begin with a legend, specified as a linefeed-separated
	/// list of `key = value` options. Following the legend are three hyphens
	/// (`---`), after which the board must occur. The grid terminates with the
	/// first blank line.
	///
	/// Column spacing defaults to `1`, but may be overridden by the
	/// `columnspacing` property. Row spacing defaults to `1`, but may be
	/// overridden by the `rowspacing` property.
	pub fn parse (tsb: &str) -> BoardResult
	{
		use self::ParseError::*;
		let mut colors = ColorMap::new();
		let mut next_color = 1;
		let mut legend = PropertyMap::new();
		legend.insert(PropertyKey::Width, PropertyValue::U32(DEFAULT_WIDTH));
		let index = match tsb.find("\n---\n")
		{
			Some(index) =>
			{
				Board::parse_legend(
					&tsb[..=index],
					&mut legend,
					&mut colors,
					&mut next_color)?;
				index + 5
			},
			None => 0
		};
		let grid = Board::parse_grid(
			&tsb[index..],
			&mut legend,
			&mut colors,
			&mut next_color)?;
		let wild_colors = match legend.get(&PropertyKey::Wild)
		{
			Some(PropertyValue::U32(mask)) => *mask,
			_ => 0
		};
		let color_locked = match legend.get(&PropertyKey::ColorLock)
		{
			Some(PropertyValue::Bool(b)) => *b,
			_ => false
		};
		let width = match legend.get(&PropertyKey::Width)
		{
			Some(PropertyValue::U32(width)) => *width,
			_ => unreachable!()
		};
		let height = (grid.len() as u32 + (width - 1)) / width;
		if width * height != grid.len() as u32
		{
			return Err(IncompleteBoard)
		}
		let removable_stones =
			grid.iter().filter(|s| s.is_removable()).count() as u32;
		let wild_stones = grid.iter().filter(|s|
		{
			use self::AnyStone::*;
			match s
			{
				Wild(_) => true,
				_ => false
			}
		}).count() as u32;
		if wild_colors.count_ones() != wild_stones
		{
			return Err(WrongWildCount)
		}
		Ok(Board
		{
			turn: 0,
			wild_colors,
			color_locked,
			highlight: None,
			width,
			height,
			removable_stones,
			grid,
			properties: legend
		})
	}

	/// Parse a board legend from the specified string. A legend is specified as
	/// a linefeed-separated list of `key = value` options. A board legend is
	/// terminated by a line containing only three hyphens (`---`). Populate the
	/// supplied map. The color map and next color mask are provided to support
	/// [wild stones].
	///
	/// [wild stones]: WildStone
	fn parse_legend (
		legend: &str,
		map: &mut PropertyMap,
		colors: &mut ColorMap,
		next_color: &mut u32) -> LegendResult
	{
		use self::LegendParseState::*;
		use self::PropertyKey::*;
		use self::PropertyValue::*;
		use self::ParseError::*;
		let mut key = None::<PropertyKey>;
		let mut state = ExpectKeyOrLinefeedOrEnd;
		let tokens = FilteredTokenizer::new(
			LegendFilter, legend).collect::<Vec<Token>>();
		for token in tokens
		{
			match (state, token.term.as_ref())
			{
				(ExpectKeyOrLinefeedOrEnd, "=") =>
					return Err(InvalidPropertySyntax),
				(ExpectKeyOrLinefeedOrEnd, "\n") =>
					state = ExpectKeyOrLinefeedOrEnd,
				(ExpectKeyOrLinefeedOrEnd, term) =>
				{
					key = Some(match term
					{
						"width" => Width,
						"wild" => Wild,
						"colorlock" => ColorLock,
						unknown =>
						{
							if unknown.len() == 1
							{
								Display(unknown.chars().next().unwrap())
							}
							else
							{
								Unknown(unknown.to_string())
							}
						}
					});
					state = ExpectEquals;
				},
				(ExpectEquals, "=") => state = ExpectValue,
				(ExpectEquals, _) => return Err(InvalidPropertySyntax),
				(ExpectValue, term) =>
				{
					let unwrapped = key.unwrap();
					match unwrapped
					{
						Width => map.insert(
							unwrapped, U32(term.parse::<u32>()?)),
						Wild =>
						{
							for c in term.chars()
							{
								if colors.get(&c) != None
								{
									return Err(RepeatedWildColor);
								}
								colors.insert(c, *next_color);
								*next_color = *next_color << 1;
							}
							map.insert(unwrapped, U32(*next_color - 1))
						},
						ColorLock => map.insert(
							unwrapped, Bool(term.parse::<bool>()?)),
						Display(c) =>
						{
							let s = format!(
								"\u{1b}[38;5;{}m{}",
								term,
								c);
							map.insert(unwrapped, String(s))
						},
						Unknown(_) => map.insert(
							unwrapped, String(term.to_string()))
					};
					key = None;
					state = ExpectLinefeed;
				},
				(ExpectLinefeed, "\n") => state = ExpectKeyOrLinefeedOrEnd,
				(ExpectLinefeed, _) => return Err(InvalidPropertySyntax)
			}
		}
		if state == ExpectKeyOrLinefeedOrEnd { Ok(()) }
		else { Err(InvalidPropertySyntax) }
	}

	/// Parse a grid from the specified string.
	fn parse_grid (
		grid: &str,
		_legend: &mut PropertyMap,
		colors: &mut ColorMap,
		next_color: &mut u32) -> GridResult
	{
		use self::AnyStone::*;
		let mut vec = Vec::<AnyStone>::new();
		let tokens = FilteredTokenizer::new(
			StoneFilter, grid).collect::<Vec<Token>>();
		for token in tokens
		{
			vec.push(match token.term.as_ref()
			{
				"_" => None(NoStone),
				"#" => Survivor(SurvivorStone),
				"*" => Wild(WildStone),
				"/" => Toggle(ToggleStone {phase: 0}),
				"+" => Toggle(ToggleStone {phase: 1}),
				s @ _ =>
				{
					let c = s.chars().next().unwrap();
					let color = *colors.entry(c).or_insert_with(|| {
						let color = *next_color;
						*next_color = *next_color << 1;
						color
					});
					Ordinary(OrdinaryStone {rep: c, color})
				}
			});
		}
		Ok(vec)
	}

	/// Answer the current turn.
	pub fn turn (&self) -> u32
	{
		self.turn
	}

	/// Answer the width of the board, in stones.
	pub fn width (&self) -> u32
	{
		self.width
	}

	/// Answer the height of the board, in rows.
	pub fn height (&self) -> u32
	{
		self.height
	}

	/// Answer the bitwise OR of all active [colors] for [wild stones]. Every
	/// wild stone has the same color space.
	///
	/// [colors]: OrdinaryStone::color
	/// [wild stones]: WildStone
	pub fn wild_colors (&self) -> u32
	{
		self.wild_colors
	}

	/// Answer `true` if the receiver is color locked, or `false` otherwise.
	pub fn color_locked (&self) -> bool
	{
		self.color_locked
	}

	/// Answer the count of removable [stones].
	///
	/// [stones]: AnyStone
	pub fn removable_stones (&self) -> u32
	{
		self.removable_stones
	}

	/// Remove the [stone] at the specified location, capturing it, and
	/// asserting that it has the specified color. The color information is
	/// needed for proper treatment of [wild stones]. Answer a closure that can
	/// reverse the effect of this removal.
	///
	/// [stone]: AnyStone
	/// [wild stones]: WildStone
	#[must_use]
	pub fn remove (
		&mut self,
		p: Point,
		s: &mut AnyStone,
		color: u32) -> Box<for<'r> FnMut(&'r mut Board)>
	{
		use self::AnyStone::*;
		let index = (p.1 * self.width + p.0) as usize;
		let stone = self.grid[index].for_board(self);
		*s = stone;
		match stone
		{
			Ordinary(o) =>
			{
				assert!(color == 0 || color == o.color);
				self.grid[index] = None(NoStone);
				self.turn += 1;
				self.removable_stones -= 1;
				let survivors = self.remove_survivors(p);
				Box::new(move |board: &mut Board|
				{
					board.add_survivors(&survivors);
					board.removable_stones += 1;
					board.turn -= 1;
					board.grid[index] = stone;
				})
			},
			Wild(_) if color == 0 =>
			{
				self.grid[index] = None(NoStone);
				self.turn += 1;
				self.removable_stones -= 1;
				let survivors = self.remove_survivors(p);
				Box::new(move |board: &mut Board|
				{
					board.add_survivors(&survivors);
					board.removable_stones += 1;
					board.turn -= 1;
					board.grid[index] = stone;
				})
			},
			Wild(_) =>
			{
				assert_ne!(self.wild_colors & color, 0);
				self.grid[index] = None(NoStone);
				self.turn += 1;
				self.removable_stones -= 1;
				self.wild_colors &= !color;
				let survivors = self.remove_survivors(p);
				Box::new(move |board: &mut Board|
				{
					board.add_survivors(&survivors);
					board.wild_colors |= color;
					board.removable_stones += 1;
					board.turn -= 1;
					board.grid[index] = stone;
				})
			},
			_ => unreachable!()
		}
	}

	/// Remove all [survivors] from the row of the specified point, but only if
	/// there are no removable stones in the row with them. Answer the removed
	/// survivors.
	///
	/// [survivors]: SurvivorStone
	#[must_use]
	fn remove_survivors (&mut self, p: Point) -> Vec<Point>
	{
		use self::AnyStone::*;
		let removable = (0..self.width)
			.map(|column|
			{
				let index = (p.1 * self.width + column) as usize;
				self.grid[index]
			})
			.filter(|s| s.is_removable())
			.count();
		if removable == 0
		{
			let mut survivors = Vec::<Point>::new();
			for column in 0..self.width
			{
				let index = (p.1 * self.width + column) as usize;
				match self.grid[index]
				{
					Survivor(_) =>
					{
						survivors.push((column, p.1));
						self.grid[index] = AnyStone::None(NoStone);
					},
					_ => {}
				}
			}
			survivors
		}
		else
		{
			vec![]
		}
	}

	/// Add [survivors] to the specified locations.
	///
	/// [survivors]: SurvivorStone
	fn add_survivors (&mut self, survivors: &Vec<Point>)
	{
		for p in survivors
		{
			let index = (p.1 * self.width + p.0) as usize;
			self.grid[index] = AnyStone::Survivor(SurvivorStone);
		}
	}

	/// Forcibly remove the [stone] at the specified location without doing any
	/// accounting other than incrementing the turn. This is a destructive
	/// operation, and should not be used for computing a board solution.
	///
	/// [stone]: AnyStone
	pub fn force_remove (&mut self, p: Point)
	{
		let index = (p.1 * self.width + p.0) as usize;
		self.grid[index] = AnyStone::None(NoStone);
		self.turn += 1;
		let _ = self.remove_survivors(p);
	}

	/// Apply the specified closure to the [stone] at `(x,y)`, where the origin
	/// `(0,0)` is the uppermost leftmost grid cell.
	///
	/// [stone]: AnyStone
	pub fn stone_do (
		&self,
		p: Point,
		action: &mut dyn for<'r, 's> FnMut(&'r Board, &'s AnyStone))
	{
		let index = (p.1 * self.width + p.0) as usize;
		let stone = self.grid[index].for_board(self);
		action(self, &stone)
	}

	/// Apply the specified closure while the specified [stone] is highlighted.
	///
	/// [stone]: AnyStone
	pub fn with_highlight (
		&mut self,
		p: Point,
		action: &mut dyn for<'r> FnMut(&'r Board))
	{
		self.highlight = Some(p);
		action(self);
		self.highlight = None;
	}
}

const NW_CORNER: char = '\u{250F}';
const NE_CORNER: char = '\u{2513}';
const SE_CORNER: char = '\u{251B}';
const SW_CORNER: char = '\u{2517}';
const V_LINE: char = '\u{2503}';
const H_LINE: char = '\u{2501}';

impl Display for Board
{
	fn fmt (&self, f: &mut Formatter) -> Result
	{
		use self::AnyStone::*;
		use self::PropertyKey::*;
		use self::PropertyValue::*;
		write!(f, "Turn #{}", self.turn + 1)?;
		if let Some((column, row)) = self.highlight
		{
			write!(f, ": \u{1b}[38;5;15m({}, {})\u{1b}[0m", column, row)?;
		}
		// Write the top of the box.
		write!(f, "\n{}", NW_CORNER)?;
		for _ in 0..(self.width << 1) - 1 { write!(f, "{}", H_LINE)?; }
		writeln!(f, "{}", NE_CORNER)?;
		// Write the contexts of the box.
		for row in 0..self.height
		{
			write!(f, "{}", V_LINE)?;
			for column in 0..self.width
			{
				let index = (row * self.width + column) as usize;
				let stone = self.grid[index].for_board(self);
				let highlight =
					if Some((column, row))==self.highlight {"\u{1b}[48;5;231m"}
					else { "" };
				let space = if column == self.width - 1 { "" } else { " " };
				match stone
				{
					Ordinary(o) =>
					{
						match self.properties.get(&Display(o.rep))
						{
							Some(String(display)) => write!(
								f,
								"{}{}\u{1b}[0m{}",
								highlight,
								display,
								space)?,
							_ => write!(
								f, "{}{}\u{1b}[0m{}", highlight, o, space)?
						}
					},
					s => write!(f, "{}{}\u{1b}[0m{}", highlight, s, space)?
				};
			}
			writeln!(f, "{}", V_LINE)?;
		}
		// Write the bottom of the box.
		write!(f, "{}", SW_CORNER)?;
		for _ in 0..(self.width << 1) - 1 { write!(f, "{}", H_LINE)?; }
		writeln!(f, "{}", SE_CORNER)?;
		Ok(())
	}
}

/******************************************************************************
 *                             Property support.                              *
 ******************************************************************************/

pub type PropertyMap = HashMap<PropertyKey, PropertyValue>;

/// A board property key.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum PropertyKey
{
	/// The width, in stones, i.e., the row stride.
	Width,

	/// The specification of colors for [wild stones](WildStone).
	Wild,

	/// A color is locked once completed, and cannot be played until another
	/// color has been played.
	ColorLock,

	/// The specification of display properties for a stone.
	Display (char),

	/// An unknown property.
	Unknown (String)
}

/// A board property value.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum PropertyValue
{
	/// An arbitrary `bool`.
	Bool (bool),

	/// An arbitrary `u32`.
	U32 (u32),

	/// Arbitrary text.
	String (String)
}

/******************************************************************************
 *                              Parsing support.                              *
 ******************************************************************************/

type BoardResult = result::Result<Board, ParseError>;
type ColorMap = HashMap<char, u32>;
type LegendResult = result::Result<(), ParseError>;
type GridResult = result::Result<Vec<AnyStone>, ParseError>;

/// The enumeration of errors that can result from [parsing] a [board].
///
/// [parsing]: Board::parse
/// [board]: Board
#[derive(Debug)]
pub enum ParseError
{
	/// Invalid property syntax in board legend.
	InvalidPropertySyntax,

	/// Invalid property value for well-known property key.
	InvalidPropertyValue,

	/// Repeated color in [wild stone](WildStone) specification.
	RepeatedWildColor,

	/// Incomplete board, i.e., the last row is not fully populated.
	IncompleteBoard,

	/// Wrong count of [wild stones](WildStone).
	WrongWildCount
}

impl From<ParseIntError> for ParseError
{
	fn from (_error: ParseIntError) -> Self
	{
		ParseError::InvalidPropertyValue
	}
}

impl From<ParseBoolError> for ParseError
{
	fn from (_error: ParseBoolError) -> Self
	{
		ParseError::InvalidPropertyValue
	}
}

/// The token filter for the board legend.
struct LegendFilter;

impl filters::Filter for LegendFilter
{
	fn on_char (&self, c: &char) -> (bool, bool)
	{
		match *c
		{
			' ' | '\t' => (true, false),
			'=' | '\n' => (true, true),
			_ => (false, false)
		}
	}
}

/// A parse expectation for a board legend parser.
#[derive(PartialEq, Eq)]
enum LegendParseState
{
	/// Expect either a new property key, a linefeed, or end-of-string.
	ExpectKeyOrLinefeedOrEnd,

	/// Expect an equals sign (`=`).
	ExpectEquals,

	/// Expect a property value.
	ExpectValue,

	/// Expect a linefeed.
	ExpectLinefeed
}

/// The token filter for the board grid.
struct StoneFilter;

impl filters::Filter for StoneFilter
{
	fn on_char (&self, c: &char) -> (bool, bool)
	{
		match *c
		{
			' ' | '\t' | '\n' => (true, false),
			_ => (true, true)
		}
	}
}
