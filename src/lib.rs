//! Quick Csv reader which performs **very** well.
//! 
//! ## Example
//! 
//! First, create a `Csv` from a `BufRead` reader, a file or a string
//! 
//! ```rust
//! extern crate quick_csv;
//! 
//! fn main() {
//!     let data = "a,b\r\nc,d\r\ne,f";
//!     let csv = quick_csv::Csv::from_string(data);
//!     for row in csv.into_iter() {
//!         // work on csv row ...
//!         if let Ok(_) = row {
//!             println!("new row!");
//!         } else {
//!             println!("cannot read next line");
//!         }
//!     }
//! }
//! ```
//! 
//! `Row` is on the other hand provides 3 methods to access csv columns:
//! - `columns`: 
//!   - iterator over columns.
//!   - Iterator item is a `&str`, which means you only have to `parse()` it to the needed type and you're done
//! 
//!   ```rust
//!   # let row = quick_csv::Csv::from_string("a,b,c,d,e,38,f").next().unwrap().unwrap();
//!   let mut cols = row.columns().expect("cannot convert to utf8");
//!   let fifth = cols.nth(5).unwrap().parse::<f64>().unwrap();
//!   println!("Doubled fifth column: {}", fifth * 2.0);
//!   ```
//! 
//! - `decode`:
//!   - deserialize into you `Decodable` struct, a-la rust-csv.
//!   - most convenient way to deal with your csv data
//! 
//!   ```rust
//!   let row = quick_csv::Csv::from_string("a,b,54").next().unwrap().unwrap();
//!   if let Ok((col1, col2, col3)) = row.decode::<(String, u64, f64)>() {
//!       println!("col1: '{}', col2: {}, col3: {}", col1, col2, col3);
//!   }
//!   ``` 
//! 
//! - `bytes_columns`:
//!   - similar to `columns` but columns are of type `&[u8]`, which means you may want to convert it to &str first
//!   - performance gain compared to `columns` is minimal, use it only if you *really* need to as it is less convenient

#![deny(missing_docs)]

extern crate rustc_serialize;

pub mod columns;
pub mod error;

use self::columns::{Columns, BytesColumns};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::iter::Iterator;
use std::path::Path;

use error::{Error, Result};
use rustc_serialize::Decodable;

#[cfg(test)] mod test;

/// Csv reader
/// 
/// Iterates over the rows of the csv
///
/// # Example
///
/// ```rust
/// let csv = quick_csv::Csv::from_file("./examples/data/bench.csv").unwrap();
/// for row in csv.into_iter() {
///     let row = row.unwrap(); // unwrap result, panic if not utf8 
///     {
///         // either use columns iterator directly (Item = &str)
///         if let Ok(mut columns) = row.columns() {
///             println!("Column 1: '{:?}', Column 2: '{:?}'", columns.next(), columns.next());
///         }
///     }
///
///     {
///         // or decode it directly into something simpler
///         if let Ok((col1, col2)) = row.decode::<(String, u64)>() {
///             println!("Column 1: '{:?}', Column 2: '{:?}'", &col1, &col2);
///         }
///     }
///
/// }
/// ```
pub struct Csv<B: BufRead> {
    /// delimiter
    delimiter: u8,
    /// reader
    reader: B,
    /// header
    has_header: bool,
    /// header
    headers: Option<Vec<String>>,
    /// flexible column count
    flexible: bool,
    /// column count
    len: Option<usize>,
    /// if was error, exit next
    exit: bool,
    /// line count
    current_line: usize,
}

impl<B: BufRead> Csv<B> {

    /// Creates a Csv from a generic BufReader
    /// 
    /// Note: default delimiter = ','
    pub fn from_reader(reader: B) -> Csv<B> {
        Csv {
            reader: reader,
            delimiter: b',',
            has_header: false,
            headers: None,
            flexible: false,
            len: None,
            exit: false,
            current_line: 0,
        }
    }

    /// Sets a new delimiter
    pub fn delimiter(mut self, delimiter: u8) -> Csv<B> {
        self.delimiter = delimiter;
        self
    }

    /// Sets flexible columns
    pub fn flexible(mut self, flexible: bool) -> Csv<B> {
        self.flexible = flexible;
        self
    }

    /// Defines whether there is a header or not
    pub fn has_header(mut self, has_header: bool) -> Csv<B> {
        self.has_header = has_header;
        let _ = self.headers();
        self
    }

   /// gets first row as Vec<String>
    pub fn headers(&mut self) -> Vec<String> {
        if let Some(ref h) = self.headers {
            return h.clone();
        }
        if self.has_header {            
            if let Some(r) = self.next() {
                if let Ok(r) = r {
                    let h = r.decode().ok().unwrap_or_else(Vec::new);
                    self.headers = Some(h.clone());
                    return h;
                }
            }
        }
        Vec::new()
    }

    /// Get column count
    pub fn column_count(&self) -> Option<usize> {
        self.len
    }

    /// Gets the current line number
    ///
    /// Useful if you get an error and want to investigate the source
    pub fn current_line(&self) -> usize {
        self.current_line
    }

}

impl Csv<BufReader<File>> {
    /// Creates a csv from a file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Csv<BufReader<File>>>
    {
        let reader = BufReader::new(try!(File::open(path)));
        Ok(Csv::from_reader(reader))
    }
}

impl<'a> Csv<&'a [u8]> {
    /// Creates a CSV reader for an in memory string buffer.
    pub fn from_string(s: &'a str) -> Csv<&'a [u8]> {
        Csv::from_reader(s.as_bytes())
    }
}

/// Iterator on csv `Row`s
impl<B: BufRead> Iterator for Csv<B> {
    type Item = Result<Row>;
    fn next(&mut self) -> Option<Result<Row>> {
        if self.exit { return None; }
        let mut buf = Vec::new();
        let mut cols = self.len.map_or_else(Vec::new, Vec::with_capacity);
        match read_line(&mut self.reader, &mut buf, self.delimiter, &mut cols) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with(&[b'\r']) {
                    buf.pop();
                }
                cols.push(buf.len());
                let c = cols.len();
                if let Some(n) = self.len {
                    if n != c && !self.flexible {
                        self.exit = true;
                        return Some(Err(Error::ColumnMismatch(n, c)));
                    }
                } else {
                    self.len = Some(c);
                }
                self.current_line += 1;
                Some(Ok(Row {
                    line: buf,
                    cols: cols,
                }))
            }
            Err(e) => {
                self.exit = true;
                Some(Err(e))
            },
        }
    }
}

/// Row struct used as Csv iterator Item
///
/// Row can be decoded into a Result<T: Decodable>
pub struct Row {
    line: Vec<u8>,
    cols: Vec<usize>,
}

impl Row {

    /// Gets an iterator over columns
    pub fn columns(&self) -> Result<Columns> {
        match ::std::str::from_utf8(&self.line) {
            Err(_) => Err(Error::Io(io::Error::new(io::ErrorKind::InvalidData,
                                    "stream did not contain valid UTF-8"))),
            Ok(s) => Ok(Columns::new(s, &self.cols)),
        }
    }

    ///  Creates a new BytesColumns iterator over &[u8]
    pub fn bytes_columns(&self) -> BytesColumns {
        BytesColumns::new(&self.line, &self.cols)
    }

    /// Decode row into custom decodable type
    pub fn decode<T: Decodable>(&self) -> Result<T> {
        let mut columns = try!(self.columns());
        Decodable::decode(&mut columns)
    }

    /// Gets columns count
    pub fn len(&self) -> usize {
        self.cols.len()
    }

    /// `Row` is empty if there is no columns
    pub fn is_empty(&self) -> bool {
        self.cols.is_empty()
    }

}

/// Consumes bytes as long as they are within quotes
/// manages "" as quote escape
/// returns
/// - Ok(true) if entirely consumed
/// - Ok(false) if no issue but it reached end of buffer
/// - Err(Error::UnescapeQuote) if a quote if found within the column
macro_rules! consume_quote {
    ($bytes: expr, $delimiter: expr, $in_quote: expr,
     $start: expr, $buf: expr, $available: expr, $quote_count: expr) => {
        $in_quote = false;
        loop {
            match $bytes.next() {
                Some((_, &b'\"')) => {
                    match $bytes.clone().next() {
                        Some((i, &b'\"')) => {
                            $bytes.next(); // escaping quote
                            let _ = $buf.write(&$available[$start..i]);
                            $start = i + 1;
                            $quote_count += 1;
                        },
                        None | Some((_, &b'\r')) | Some((_, &b'\n')) => break,
                        Some((_, d)) if *d == $delimiter => break,
                        Some((_, _)) => return Err(Error::UnescapedQuote),
                    }
                },
                None => {
                    $in_quote = true;
                    break;
                },
                _ => (),
            }
        }
    }
}

fn read_line<R: BufRead>(r: &mut R, buf: &mut Vec<u8>,
    delimiter: u8, cols: &mut Vec<usize>) -> Result<usize>
{
    let mut read = 0;
    let mut in_quote = false;
    let mut done = false;
    let mut quote_count = 0;
    while !done {
        let used = {
            let available = match r.fill_buf() {
                Ok(n) if n.is_empty() => return Ok(read),
                Ok(n) => n,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(Error::from(e)),
            };
            
            let mut bytes = available.iter().enumerate();
            let mut start = 0;

            // previous buffer was exhausted without exiting from quotes
            if in_quote {
                consume_quote!(bytes, delimiter, in_quote, start, buf, available, quote_count);
            }

            // use a simple loop instead of for loop to allow nested loop
            let used: usize;
            loop {
                match bytes.next() {
                    Some((i, &b'\"')) => {
                        if i == 0 || available[i - 1] == delimiter {
                            consume_quote!(bytes, delimiter, in_quote, start, buf, available, quote_count);
                        } else {
                            return Err(Error::UnexpextedQuote);
                        }
                    },
                    Some((i, &b'\n')) => {
                        done = true;
                        used = i + 1;
                        let _ = buf.write(&available[start..i]);
                        break;
                    },
                    Some((i, &d)) => {
                        if d == delimiter { cols.push(read + i - quote_count); }
                    },
                    None => {
                        used = available.len();
                        let _ = buf.write(&available[start..used]);
                        break;
                    },
                }
            }
            used
        };
        r.consume(used);
        read += used;
    }
    Ok(read)
}
