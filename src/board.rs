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

/******************************************************************************
 *                                  Stones.                                   *
 ******************************************************************************/

/// The behavior profile of an arbitrary stone.
trait Stone
{
	/// Answer the state of the receiver given the specified board state.
	fn for_board (&self, board: &Board) -> Self;
}

/// The absence of stoniness.
#[derive(Copy, Clone, Debug)]
struct NoStone;

impl Stone for NoStone
{
	/// Answer a copy of the receiver.
	fn for_board (&self, _board: &Board) -> Self
	{
		*self
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
#[derive(Copy, Clone, Debug)]
struct OrdinaryStone
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

impl Stone for OrdinaryStone
{
	/// Answer a copy of the receiver.
	fn for_board (&self, _board: &Board) -> Self
	{
		*self
	}
}

impl Display for OrdinaryStone
{
	fn fmt (&self, f: &mut Formatter) -> Result
	{
		write!(f, "{}", self.rep)
	}
}

/// A wild stone has potential to provide one or more colors; all other wild
/// stones lose a color whenever a wild stone is committed to a particular
/// color. A wild stone's color space is a property of the [board], not of the
/// wild stone itself (featherweight pattern). Always represented by `*`.
///
/// [board]: Board
#[derive(Copy, Clone, Debug)]
struct WildStone;

impl Stone for WildStone
{
	/// Answer a copy of the receiver.
	fn for_board (&self, _board: &Board) -> Self
	{
		*self
	}
}

impl Display for WildStone
{
	fn fmt (&self, f: &mut Formatter) -> Result
	{
		write!(f, "*")
	}
}

/// A toggle stone cannot be matched directly. It alternately obstructs and
/// permits access to stones above it. Initially open is represented by `O`,
/// initially closed is represented by `X`.
#[derive(Copy, Clone, Debug)]
struct ToggleStone
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

impl Stone for ToggleStone
{
	/// Answer the state of the receiver given the specified board state.
	fn for_board (&self, board: &Board) -> Self
	{
		ToggleStone { phase: self.phase + board.turn }
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
		write!(f, "{}", if self.phase & 1 == 0 { "O" } else { "X" })
	}
}

/// An arbitrary stone.
#[derive(Copy, Clone, Debug)]
enum AnyStone
{
	None (NoStone),
	Ordinary (OrdinaryStone),
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
			Wild(s) => Wild(s.for_board(board)),
			Toggle(s) => Toggle(s.for_board(board))
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
			Wild(s) => s.fmt(f),
			Toggle(s) => s.fmt(f)
		}
	}
}

/******************************************************************************
 *                                   Board.                                   *
 ******************************************************************************/

/// The state of the game board during a particular turn.
struct Board
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

	/// The row stride of the physical board, i.e., the number of [stones] in
	/// any given row.
	///
	/// [stones]: Stone
	width: u32,

	/// The column stride of the physical board, i.e., the number of [stones] in
	/// any given column.
	///
	/// [stones]: Stone
	height: u32,

	/// The physical board, as a single linear vector.
	grid: Vec<AnyStone>
}

impl Board
{
	/// Apply the specified closure to the [stone] at `(x,y)`, where the origin
	/// `(0,0)` is the uppermost leftmost grid cell.
	///
	/// [stone]: AnyStone
	fn stone_do (
		&self,
		x: u32,
		y: u32,
		action: &mut dyn for<'r, 's> FnMut(&'r Board, &'s AnyStone))
	{
		let index = (y * self.width + x) as usize;
		let stone = self.grid[index];
		action(self, &stone)
	}

	/// Apply the specified closure to the [stone] at `(x,y)`, where the origin
	/// `(0,0)` is the uppermost leftmost grid cell.
	fn mut_stone_do (
		&mut self,
		x: u32,
		y: u32,
		action: &mut dyn for<'r, 's> FnMut(&'r mut Board, &'s AnyStone))
	{
		let index = (y * self.width + x) as usize;
		let stone = self.grid[index];
		action(self, &stone)
	}

	/// Apply the specified closure to every [stone].
	///
	/// [stone]: AnyStone
	fn board_do (
		&self,
		action: &mut dyn for<'r, 's> FnMut(&'r Board, &'s AnyStone))
	{
		for row in 0..self.height
		{
			for column in 0..self.width
			{
				self.stone_do(row, column, action)
			}
		}
	}

	/// Apply the specified closure to every [stone].
	///
	/// [stone]: AnyStone
	fn mut_board_do (
		&mut self,
		action: &mut dyn for<'r, 's> FnMut(&'r mut Board, &'s AnyStone))
	{
		for row in 0..self.height
		{
			for column in 0..self.width
			{
				self.mut_stone_do(row, column, action)
			}
		}
	}
}

impl Board
{
	fn parse (tsb: &str) -> Board
	{
		unimplemented!()
	}
}

impl Display for Board
{
	fn fmt (&self, f: &mut Formatter) -> Result
	{
		for row in 0..self.height
		{
			for column in 0..self.width
			{
				let index = (row * self.width + column) as usize;
				let stone = self.grid[index];
				write!(f, "{}", stone)?;
			}
		}
		Ok(())
	}
}
