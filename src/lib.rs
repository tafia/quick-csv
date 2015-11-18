extern crate rustc_serialize;

pub mod columns;
pub mod error;

use self::columns::Columns;
use std::io::{BufRead, BufReader, Lines};
use std::fs::File;
use std::path::Path;

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
///    
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
    /// columns are simply a split on the delimiter
    lines: Lines<B>,
    /// delimiter
    delimiter: char
}

impl<B: BufRead> Csv<B> {

    /// Creates a Csv from a generic BufReader
    /// 
    /// Note: default delimiter = ','
    pub fn from_reader(reader: B) -> Csv<B> {
        Csv {
            lines: reader.lines(),
            delimiter: ',',
        }
    }

    /// Sets a new delimiter
    pub fn delimiter(mut self, delimiter: char) -> Csv<B> {
        self.delimiter = delimiter;
        self
    }

    /// gets first row as Vec<String>
    pub fn header(&mut self) -> Vec<String> {
        self.next().map(|r| r.columns().map(|c| c.to_owned()).collect())
            .unwrap_or_else(|| Vec::new())
    }
}

impl Csv<BufReader<File>> {

    /// Creates a csv from a file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> ::std::io::Result<Csv<BufReader<File>>>
    {
        let reader = BufReader::new(try!(File::open(path)));
        Ok(Csv::from_reader(reader))
    }

}

/// Iterator on csv returning rows
impl<B: BufRead> Iterator for Csv<B> {
    type Item = Row;
    fn next(&mut self) -> Option<Row> {
        self.lines.next().map(|l| Row {
            line: l.unwrap(),
            delimiter: self.delimiter,
        })
    }
}

/// Row struct used as Csv iterator Item
///
/// Row can be decoded into a Result<T: Decodable>
pub struct Row {
    delimiter: char,
    line: String,
}

impl Row {

    /// Gets an iterator over columns
    pub fn columns<'a>(&'a self) -> Columns<'a> {
        Columns::new(&self.line, &self.delimiter)
    }

    /// Decode row into custom decodable type
    pub fn decode<T: Decodable>(&self) -> error::Result<T> {
        let mut columns = self.columns();
        Decodable::decode(&mut columns)
    }

}
