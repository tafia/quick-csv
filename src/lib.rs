extern crate rustc_serialize;

pub mod columns;
pub mod error;

use self::columns::{Columns, BytesColumns};
use std::io::{self, BufRead, BufReader, Write, Cursor};
use std::fs::File;
use std::path::Path;
use std::iter::{Enumerate, Iterator};

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
        }
    }

    /// Sets a new delimiter
    pub fn delimiter(mut self, delimiter: u8) -> Csv<B> {
        self.delimiter = delimiter;
        self
    }

    /// Defines whether there is a header or not
    pub fn has_header(mut self, has_header: bool) -> Csv<B> {
        self.has_header = has_header;
        self
    }

   /// gets first row as Vec<String>
    pub fn header(&mut self) -> Result<Vec<String>> {
        if self.has_header {            
            if let Some(r) = self.next() {
                let r = try!(r);
                return r.decode();
            }
        }
        Ok(Vec::new())
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

impl Csv<BufReader<Cursor<Vec<u8>>>> {
    /// Creates a CSV reader for an in memory string buffer.
    pub fn from_string<'a, S>(s: S) -> Csv<Cursor<Vec<u8>>>
            where S: Into<String> {
        Csv::from_bytes(s.into().into_bytes())
    }

    /// Creates a CSV reader for an in memory buffer of bytes.
    pub fn from_bytes<'a, V>(bytes: V) -> Csv<Cursor<Vec<u8>>>
            where V: Into<Vec<u8>> {
        Csv::from_reader(Cursor::new(bytes.into()))
    }
}

/// Iterator on csv returning rows
impl<B: BufRead> Iterator for Csv<B> {
    type Item = Result<Row>;
    fn next(&mut self) -> Option<Result<Row>> {
        let mut buf = Vec::new();
        let mut cols = Vec::new();
        match read_line(&mut self.reader, &mut buf, self.delimiter, &mut cols) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.end_with(&[b'\n']) {
                    buf.pop();
                    if buf.end_with(&[b'\r']) {
                        buf.pop();
                    }
                }
                cols.push(buf.len());
                Some(Ok(Row {
                    line: buf,
                    cols: cols,
                }))
            }
            Err(e) => Some(Err(Error::from(e)))
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
    pub fn columns<'a>(&'a self) -> Result<Columns<'a>> {
        match ::std::str::from_utf8(&self.line) {
            Err(_) => Err(Error::from(io::Error::new(io::ErrorKind::InvalidData,
                                                     "stream did not contain valid UTF-8"))),
            Ok(s) => Ok(Columns::new(s, &self.cols)),
        }
    }

    pub fn bytes_columns<'a>(&'a self) -> BytesColumns<'a> {
        BytesColumns::new(&self.line, &self.cols)
    }

    /// Decode row into custom decodable type
    pub fn decode<T: Decodable>(&self) -> Result<T> {
        let mut columns = try!(self.columns());
        Decodable::decode(&mut columns)
    }

}

/// Consumes bytes as long as they are within quotes
/// manages "" as quote escape
/// returns
/// - Ok(true) if entirely consumed
/// - Ok(false) if no issue but it reached end of buffer
/// - Err(Error::UnescapeQuote) if a quote if found within the column
fn consume_quote<'a>(bytes: &'a mut Enumerate<::std::slice::Iter<u8>>, delimiter: u8) -> Result<bool> {
    loop {
        match bytes.next() {
            Some((_, &b'\"')) => {
                match bytes.clone().next() {
                    Some((_, &b'\"')) => bytes.next(), // escaping quote
                    None | Some((_, &b'\r')) | Some((_, &b'\n')) => return Ok(true),
                    Some((_, d)) if *d == delimiter => return Ok(true),
                    _ => return Err(Error::UnescapedQuote),
                }
            },
            None => return Ok(false),
            _ => (),
        }
    }
}

fn read_line<R: BufRead>(r: &mut R, buf: &mut Vec<u8>,
    delimiter: u8, cols: &mut Vec<usize>) -> Result<usize>
{
    let mut read = 0;
    let mut in_quote = false;
    loop {
        let (done, used) = {
            let available = match r.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(Error::from(e))
            };
            
            if available.is_empty() { return Ok(read); }
            
            let mut bytes = available.iter().enumerate();
            let (mut done, mut used) = (false, available.len());

            // previous buffer was exhausted without exiting from quotes
            if in_quote && try!(consume_quote(&mut bytes, delimiter)) {
                in_quote = false;
            }

            // use a simple loop instead of for loop to allow nested loop
            loop {
                match bytes.next() {
                    Some((i, &b'\"')) => {
                        if i > 0 && available[i - 1] != delimiter {
                            return Err(Error::UnexpextedQuote);
                        }
                        if !try!(consume_quote(&mut bytes, delimiter)) {
                            in_quote = true;
                            let _ = buf.write(available);
                            break;
                        }
                    },
                    Some((i, &b'\n')) => {
                        used = i + 1;
                        let _ = buf.write(&available[..used]);
                        done = true;
                        break;
                    },
                    Some((i, &d)) => {
                        if d == delimiter { cols.push(read + i); }
                    },
                    None => {
                        let _ = buf.write(available);
                        break;
                    },
                }
            }
            (done, used)
        };
        r.consume(used);
        read += used;
        if done {
            return Ok(read);
        }
    }
}
