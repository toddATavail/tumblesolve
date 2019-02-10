//
// solve.rs
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
//! ## Solver
//!
//! Herein is functionality specific to solving Tumblestone puzzles.
//!

use crate::board::*;

/******************************************************************************
 *                         Legal move determination.                          *
 ******************************************************************************/

/// The sentinel color of [wild stones](WildStone).
pub const WILD_COLOR: u32 = 0;

impl Board
{
	/// Answer `true` if the receiver is solved, `false` otherwise.
	fn is_solved (&self) -> bool
	{
		self.removable_stones() == 0
	}

	/// Solve the board. Answer the sequences of moves required to solve the
	/// board, or `None` if the board has no solution.
	pub fn solve (&mut self) -> Option<Vec<Point>>
	{
		let mut moves = Vec::<Point>::new();
		match self.solve_recursively(&mut moves, WILD_COLOR, true)
		{
			true if moves.len() % 3 == 0 => Some(moves),
			_ => None
		}
	}

	/// Solve the receiver recursively. `moves` is the sequence of moves played
	/// thus far, `color` is the active color filter, and `allow_wild` is `true`
	/// iff a [wild stone] may be chosen.
	///
	/// [wild stone]: WildStone
	fn solve_recursively (
		&mut self,
		moves: &mut Vec<Point>,
		color: u32,
		allow_wild: bool) -> bool
	{
		// If the board has been solved, then return; let the callers deal with
		// restoring the board to its original state.
		if self.is_solved()
		{
			return true
		}
		// Iterate through all available moves, using the current color and wild
		// stone permissiveness.
		let available = self.frontier(color, allow_wild);
		for p in available
		{
			moves.push(p);
			let mut stone = AnyStone::None(NoStone);
			let mut undo = self.remove(p, &mut stone, color);
			// Update the allowed next color and wild permissiveness based on 1)
			// whether a triplet is already in progress and 2) the nature of the
			// stone just removed.
			let (new_color, new_allow_wild) =
				if self.turn() % 3 == 0
				{
					(WILD_COLOR, true)
				}
				else
				{
					match stone
					{
						AnyStone::Ordinary(o) => (o.color(), allow_wild),
						AnyStone::Wild(_) => (color, false),
						_ => unreachable!()
					}
				};
			// Recurse using the new move sequence, color filter, and wild
			// permissiveness.
			if self.solve_recursively(moves, new_color, new_allow_wild)
			{
				undo(self);
				return true;
			}
			// Undo the effects of the latest move prior to playing the next
			// one.
			undo(self);
			moves.truncate(moves.len() - 1);
		}
		return false
	}

	/// Compute the frontier of the board, i.e., those [stones] which may be
	/// physically manipulated. Answer the coordinates of all stones that pass
	/// the specified color and wild filters.
	///
	/// [stones]: AnyStone
	fn frontier (&self, color: u32, allow_wild: bool) -> Vec<Point>
	{
		use crate::board::AnyStone::*;
		let mut vec = Vec::<(u32, u32)>::new();
		let mut next_column ;
		for column in 0..self.width()
		{
			for row in (0..self.height()).rev()
			{
				next_column = false;
				self.stone_do((column, row), &mut |_, stone|
				{
					match stone
					{
						None(_) => {},
						Ordinary(_) if color == 0 =>
						{
							vec.push((column, row));
							next_column = true;
						}
						Ordinary(o @ OrdinaryStone {..})
							if o.color() == color =>
						{
							vec.push((column, row));
							next_column = true;
						},
						Ordinary(_) => next_column = true,
						Wild(_) if !allow_wild => next_column = true,
						Wild(_) if color == 0 =>
						{
							vec.push((column, row));
							next_column = true;
						},
						Wild(_) if color & self.wild_colors() != 0 =>
						{
							vec.push((column, row));
							next_column = true;
						},
						Wild(_) => next_column = true,
						Toggle(toggle) =>
						{
							next_column = !toggle.is_open();
						},
					}
				});
				if next_column { break }
			}
		}
		vec
	}
}
