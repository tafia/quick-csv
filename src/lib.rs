extern crate rustc_serialize;

pub mod columns;
pub mod error;

use self::columns::Columns;
use std::io::{BufRead, BufReader, Write};
use std::fs::File;
use std::path::Path;

use error::{Error, Result};
use rustc_serialize::Decodable;

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
///         let mut columns = row.columns();
///         println!("Column 1: '{:?}', Column 2: '{:?}'", columns.next(), columns.next());
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
}

impl<B: BufRead> Csv<B> {

    /// Creates a Csv from a generic BufReader
    /// 
    /// Note: default delimiter = ','
    pub fn from_reader(reader: B) -> Csv<B> {
        Csv {
            reader: reader,
            delimiter: b',',
        }
    }

    /// Sets a new delimiter
    pub fn delimiter(mut self, delimiter: u8) -> Csv<B> {
        self.delimiter = delimiter;
        self
    }

   /// gets first row as Vec<String>
    pub fn header(&mut self) -> Vec<String> {
        self.next().and_then(|r| r.ok().map(|r| r.columns().map(|c| c.to_owned()).collect()))
            .unwrap_or_else(|| Vec::new())
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

/// Iterator on csv returning rows
impl<B: BufRead> Iterator for Csv<B> {
    type Item = Result<Row>;
    fn next(&mut self) -> Option<Result<Row>> {
        let mut buf = String::new();
        let mut cols = Vec::new();
        match append_line_to_string(&mut self.reader, &mut buf, self.delimiter, &mut cols) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with("\n") {
                    buf.pop();
                    if buf.ends_with("\r") {
                        buf.pop();
                    }
                    cols.push(buf.len());
                }
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
    line: String,
    cols: Vec<usize>,
}

impl Row {

    /// Gets an iterator over columns
    pub fn columns<'a>(&'a self) -> Columns<'a> {
        Columns::new(&self)
    }

    /// Decode row into custom decodable type
    pub fn decode<T: Decodable>(&self) -> Result<T> {
        let mut columns = self.columns();
        Decodable::decode(&mut columns)
    }

}

fn read_line<R: BufRead>(r: &mut R, buf: &mut Vec<u8>,
    delimiter: u8, cols: &mut Vec<usize>) -> Result<usize>
{
    let mut read = 0;
    loop {
        let (done, used) = {
            let available = match r.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == ::std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(Error::from(e))
            };
            
            if available.is_empty() { return Ok(read); }
            
            let mut bytes = available.iter().enumerate();
            let (mut done, mut used) = (false, available.len());
            // use a simple loop instead of for loop to allow nested loop
            loop {
                match bytes.next() {
                    Some((_, &b'\"')) => {
                        loop {
                            match bytes.next() {
                                Some((_, &b'\"')) | None => break,
                                _ => (),
                            }
                        }
                    },
                    Some((i, &b'\n')) => {
                        let _ = buf.write(&available[..i + 1]);
                        done = true;
                        used = i + 1;
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

/// Fn inspired by [std::io::BufRead](https://doc.rust-lang.org/src/std/io/mod.rs.html#297)
/// implementation
fn append_line_to_string<R: BufRead>(r: &mut R, buf: &mut String, 
    delimiter: u8, cols: &mut Vec<usize>) -> Result<usize>
{
    struct Guard<'a> { s: &'a mut Vec<u8>, len: usize }
        impl<'a> Drop for Guard<'a> {
        fn drop(&mut self) {
            unsafe { self.s.set_len(self.len); }
        }
    }

    unsafe {
        let mut g = Guard { len: buf.len(), s: buf.as_mut_vec() };
        let ret = read_line(r, g.s, delimiter, cols);
        if ::std::str::from_utf8(&g.s[g.len..]).is_err() {
            ret.and_then(|_| {
                Err(Error::from(::std::io::Error::new(::std::io::ErrorKind::InvalidData,
                               "stream did not contain valid UTF-8")))
            })
        } else {
            g.len = g.s.len();
            ret
        }
    }
}
