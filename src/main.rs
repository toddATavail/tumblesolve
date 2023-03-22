//
// main.rs
// Copyright © 2019-2023, Todd L Smith.
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

mod board;
mod solve;

use std::env::args;
use std::fs::read_to_string;
use std::io::{Error, stdin};
use board::{Board, ParseError};

/// Execute the solver against the board extracted from the command line. Our
/// command-line parsing is trivial, so we don't need to bother with, e.g.,
/// Clap. The sole argument is the board file (`.tsb`); there are several
/// example boards in the project `examples` directory.
fn main () -> Result<(), AppError>
{
	let args: Vec<String> = args().collect();
	let file = match args.get(1)
	{
		Some(file) => file,
		None => return Err(AppError::UsageError)
	};
    let contents = read_to_string(file)?;
    let mut board = Board::parse(&contents)?;
	match board.solve()
	{
		Some(moves) =>
		{
			for m in moves
			{
				board.with_highlight(
					m,
					&mut |board| println!("{}", board));
				board.force_remove(m);
				println!(
					"Press \u{1b}[38;5;15m[Enter]\u{1b}[0m for next hint.");
				stdin().read_line(&mut String::new())?;
			}
		}
		None => println!("\u{1b}[38;5;11mNo solution exists.\u{1b}[0m")
	}
    Ok(())
}

/// An application error.
#[derive(Debug)]
enum AppError
{
	/// The user has invoked the application incorrectly.
	UsageError,

	/// An I/O error has occurred.
    IOError (Error),

	/// A parse error has occurred.
    ParseError (ParseError)
}

impl From<ParseError> for AppError
{
    fn from (error: ParseError) -> Self
   	{
   		AppError::ParseError(error)
   	}
}

impl From<Error> for AppError
{
    fn from (error: Error) -> Self
   	{
   		AppError::IOError(error)
   	}
}
