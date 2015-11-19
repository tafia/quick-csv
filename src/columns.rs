use std::default::Default;
use std::str::FromStr;

use rustc_serialize as serialize;

use error::{Result, Error};
use std::slice::Iter;

pub struct Columns<'a> {
    pos: usize,
    line: &'a str,
    iter: Iter<'a, usize>,
}

impl<'a> Iterator for Columns<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        self.iter.next().map(|p| {
            let s = &self.line[self.pos..*p];
            self.pos = *p + 1;
            s
        })
    }
}

impl<'a> Columns<'a> {

    pub fn new(r: &'a super::Row) -> Columns<'a> {
        Columns {
            pos: 0,
            line: &r.line,
            iter: r.cols.iter()
        }
    }

    fn peek(&self) -> Option<&'a str> {
        self.iter.clone().next().map(|p| {
            &self.line[self.pos..*p]
        })
    }

    fn from_str<T: FromStr + Default>(&mut self) -> Result<T> {
        let col = try!(self.next().ok_or(Error::EOL));
        FromStr::from_str(col).map_err(|_| Error::Decode(format!("Failed converting '{}'", col)))
    }

    pub fn decode<T: serialize::Decodable>(&mut self) -> Result<T> {
        serialize::Decodable::decode(self)
    }

}

impl<'a> serialize::Decoder for Columns<'a> {
    type Error = Error;

    fn error(&mut self, err: &str) -> Error {
        Error::Decode(err.into())
    }
    fn read_nil(&mut self) -> Result<()> { unimplemented!() }
    fn read_usize(&mut self) -> Result<usize> { self.from_str() }
    fn read_u64(&mut self) -> Result<u64> { self.from_str() }
    fn read_u32(&mut self) -> Result<u32> { self.from_str() }
    fn read_u16(&mut self) -> Result<u16> { self.from_str() }
    fn read_u8(&mut self) -> Result<u8> { self.from_str() }
    fn read_isize(&mut self) -> Result<isize> { self.from_str() }
    fn read_i64(&mut self) -> Result<i64> { self.from_str() }
    fn read_i32(&mut self) -> Result<i32> { self.from_str() }
    fn read_i16(&mut self) -> Result<i16> { self.from_str() }
    fn read_i8(&mut self) -> Result<i8> { self.from_str() }
    fn read_bool(&mut self) -> Result<bool> { self.from_str() }
    fn read_f64(&mut self) -> Result<f64> { self.from_str() }
    fn read_f32(&mut self) -> Result<f32> { self.from_str() }
    fn read_char(&mut self) -> Result<char> {
        let col = try!(self.next().ok_or(Error::EOL));
        if col.len() != 1 {
            return Err(Error::Decode(format!("Expected a single char, found {} chars", col.len())));
        }
        Ok(col.chars().next().unwrap())
    }
    fn read_str(&mut self) -> Result<String> {
        match self.next() {
            Some(col) => Ok(col.to_owned()),
            None => Err(Error::EOL)
        }
    }
    fn read_enum<T, F>(&mut self, _: &str, f: F) -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        f(self)
    }
    fn read_enum_variant<T, F>(&mut self, _: &[&str], _: F)
                              -> Result<T>
            where F: FnMut(&mut Columns<'a>, usize) -> Result<T> {
        unimplemented!()
    }
    fn read_enum_variant_arg<T, F>(&mut self, _: usize, f: F) -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        f(self)
    }
    fn read_enum_struct_variant<T, F>(&mut self, names: &[&str], f: F)
                                     -> Result<T>
            where F: FnMut(&mut Columns<'a>, usize) -> Result<T> {
        self.read_enum_variant(names, f)
    }
    fn read_enum_struct_variant_field<T, F>(&mut self, _: &str,
                                            f_idx: usize, f: F)
                                           -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        self.read_enum_variant_arg(f_idx, f)
    }
    fn read_struct<T, F>(&mut self, _: &str, _: usize, f: F)
                        -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        f(self)
    }
    fn read_struct_field<T, F>(&mut self, _: &str, _: usize, f: F)
                              -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        f(self)
    }
    fn read_tuple<T, F>(&mut self, _: usize, f: F) -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        f(self)
    }
    fn read_tuple_arg<T, F>(&mut self, _: usize, f: F) -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        f(self)
    }
    fn read_tuple_struct<T, F>(&mut self, _: &str, _: usize, _: F)
                              -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        unimplemented!()
    }
    fn read_tuple_struct_arg<T, F>(&mut self, _: usize, _: F)
                                  -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        unimplemented!()
    }
    fn read_option<T, F>(&mut self, mut f: F) -> Result<T>
            where F: FnMut(&mut Columns<'a>, bool) -> Result<T> {
        let col = try!(self.peek().ok_or(Error::EOL));
        if col.is_empty() {
            f(self, false)
        } else {
            f(self, true).or_else(|_| f(self, false))
        }
    }
    fn read_seq<T, F>(&mut self, f: F) -> Result<T>
            where F: FnOnce(&mut Columns<'a>, usize) -> Result<T> {
        let len = self.iter.clone().count();
        f(self, len)
    }
    fn read_seq_elt<T, F>(&mut self, _: usize, f: F) -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        f(self)
    }
    fn read_map<T, F>(&mut self, _: F) -> Result<T>
            where F: FnOnce(&mut Columns<'a>, usize) -> Result<T> {
        unimplemented!()
    }
    fn read_map_elt_key<T, F>(&mut self, _: usize, _: F) -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        unimplemented!()
    }
    fn read_map_elt_val<T, F>(&mut self, _: usize, _: F) -> Result<T>
            where F: FnOnce(&mut Columns<'a>) -> Result<T> {
        unimplemented!()
    }
}
