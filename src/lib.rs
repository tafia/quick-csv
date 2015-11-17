extern crate rustc_serialize;

pub mod columns;
pub mod error;

use self::columns::Columns;
use std::io::{BufRead, BufReader, Lines};
use std::fs::File;
use std::path::Path;

pub struct Csv {
    /// columns are simply a split on the separator
    lines: Lines<BufReader<File>>,
    /// separator
    separator: char
}

impl Csv {

    pub fn from_file<P: AsRef<Path>>(path: P, separator: char) -> ::std::io::Result<Csv> {
        let f = try!(File::open(path));
        let lines = BufReader::new(f).lines();
        Ok(Csv {
            lines: lines,
            separator: separator
        })
    }
}

/// Iterator on csv returning rows
impl Iterator for Csv {
    
    type Item = Row;

    fn next(&mut self) -> Option<Row> {
        self.lines.next().map(|l| Row {
            line: l.unwrap(),
            separator: self.separator,
        })
    }

}

pub struct Row {
    separator: char,
    line: String,
}

impl Row {

    pub fn columns<'a>(&'a self) -> Columns<'a> {
        Columns::new(&self.line, &self.separator)
    }

}

impl<T: rustc_serialize::Decodable> Into<error::Result<T>> for Row {
    fn into(self) -> error::Result<T> {
        let mut columns = self.columns();
        rustc_serialize::Decodable::decode(&mut columns)
    }
}
