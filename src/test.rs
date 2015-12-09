use Csv;
use error::Error;

use std::io::{self, Read, Seek};

fn assert_svec_eq<S, T>(got: Vec<Vec<S>>, expected: Vec<Vec<T>>)
        where S: AsRef<str>, T: AsRef<str> {
    let got: Vec<Vec<&str>> =
        got.iter().map(|row| {
            row.iter().map(|f| f.as_ref()).collect()
        }).collect();
    let expected: Vec<Vec<&str>> =
        expected.iter().map(|row| {
            row.iter().map(|f| f.as_ref()).collect()
        }).collect();

    println!("got len: {}, expected len: {}", got.len(), expected.len());
    println!("got lengths: {:?}",
             got.iter().map(|row: &Vec<&str>| row.len())
                       .collect::<Vec<usize>>());
    println!("expected lengths: {:?}",
             expected.iter().map(|row: &Vec<&str>| row.len())
                            .collect::<Vec<usize>>());
    assert_eq!(got, expected);
}

macro_rules! parses_to {
    ($name:ident, $csv:expr, $vec:expr) => (
        parses_to!($name, $csv, $vec, |rdr| rdr);
    );
    ($name:ident, $csv:expr, $vec:expr, $config:expr) => (
        #[test]
        fn $name() {
            let mut rdr = Csv::from_string($csv);
            rdr = $config(rdr);
            let rows = rdr.map(|r| r.and_then(|r| r.decode()).unwrap())
                          .collect::<Vec<Vec<String>>>();
            assert_svec_eq::<String, &str>(rows, $vec);
        }
    );
}

macro_rules! fail_parses_to {
    ($name:ident, $csv:expr, $vec:expr) => (
        fail_parses_to!($name, $csv, $vec, |rdr| rdr);
    );
    ($name:ident, $csv:expr, $vec:expr, $config:expr) => (
        #[test]
        #[should_panic]
        fn $name() {
            let mut rdr = Csv::from_string($csv);
            rdr = $config(rdr);
            let rows = rdr.map(|r| r.and_then(|r| r.decode()).unwrap())
                          .collect::<Vec<Vec<String>>>();
            assert_svec_eq::<String, &str>(rows, $vec);
        }
    );
}

macro_rules! decodes_to {
    ($name:ident, $csv:expr, $ty:ty, $vec:expr) => (
        decodes_to!($name, $csv, $ty, $vec, false);
    );
    ($name:ident, $csv:expr, $ty:ty, $vec:expr, $headers:expr) => (
        #[test]
        fn $name() {
            let mut rdr = Csv::from_string($csv);
            if $headers { rdr.next(); }
            let rows = rdr.map(|r| r.unwrap().decode().unwrap())
                          .collect::<Vec<$ty>>();
            assert_eq!(rows, $vec);
        }
    );
}


parses_to!(one_row_one_field, "a", vec![vec!["a"]]);
parses_to!(one_row_many_fields, "a,b,c", vec![vec!["a", "b", "c"]]);
parses_to!(one_row_trailing_comma, "a,b,", vec![vec!["a", "b", ""]]);
parses_to!(one_row_one_field_lf, "a\n", vec![vec!["a"]]);
parses_to!(one_row_many_fields_lf, "a,b,c\n", vec![vec!["a", "b", "c"]]);
parses_to!(one_row_trailing_comma_lf, "a,b,\n", vec![vec!["a", "b", ""]]);
parses_to!(one_row_one_field_crlf, "a\r\n", vec![vec!["a"]]);
parses_to!(one_row_many_fields_crlf, "a,b,c\r\n", vec![vec!["a", "b", "c"]]);
parses_to!(one_row_trailing_comma_crlf, "a,b,\r\n", vec![vec!["a", "b", ""]]);
parses_to!(one_row_one_field_cr, "a\r", vec![vec!["a"]]);
parses_to!(one_row_many_fields_cr, "a,b,c\r", vec![vec!["a", "b", "c"]]);
parses_to!(one_row_trailing_comma_cr, "a,b,\r", vec![vec!["a", "b", ""]]);

parses_to!(many_rows_one_field, "a\nb", vec![vec!["a"], vec!["b"]]);
parses_to!(many_rows_many_fields,
           "a,b,c\nx,y,z", vec![vec!["a", "b", "c"], vec!["x", "y", "z"]]);
parses_to!(many_rows_trailing_comma,
           "a,b,\nx,y,", vec![vec!["a", "b", ""], vec!["x", "y", ""]]);
parses_to!(many_rows_one_field_lf, "a\nb\n", vec![vec!["a"], vec!["b"]]);
parses_to!(many_rows_many_fields_lf,
           "a,b,c\nx,y,z\n", vec![vec!["a", "b", "c"], vec!["x", "y", "z"]]);
parses_to!(many_rows_trailing_comma_lf,
           "a,b,\nx,y,\n", vec![vec!["a", "b", ""], vec!["x", "y", ""]]);
parses_to!(many_rows_one_field_crlf, "a\r\nb\r\n", vec![vec!["a"], vec!["b"]]);
parses_to!(many_rows_many_fields_crlf,
           "a,b,c\r\nx,y,z\r\n",
           vec![vec!["a", "b", "c"], vec!["x", "y", "z"]]);
parses_to!(many_rows_trailing_comma_crlf,
           "a,b,\r\nx,y,\r\n", vec![vec!["a", "b", ""], vec!["x", "y", ""]]);

// parses_to!(trailing_lines_no_record,
//            "\n\n\na,b,c\nx,y,z\n\n\n",
//            vec![vec!["a", "b", "c"], vec!["x", "y", "z"]]);
// parses_to!(trailing_lines_no_record_crlf,
//            "\r\n\r\n\r\na,b,c\r\nx,y,z\r\n\r\n\r\n",
//            vec![vec!["a", "b", "c"], vec!["x", "y", "z"]]);
parses_to!(empty_string_no_headers, "", vec![]);
parses_to!(empty_string_headers, "", vec![],
            |rdr: Csv<_>| rdr.has_header(true));
parses_to!(empty_lines, "\n\n\n\n", vec![vec![""], vec![""], vec![""], vec![""]]);
// parses_to!(empty_lines_interspersed, "\n\na,b\n\n\nx,y\n\n\nm,n\n",
//            vec![vec!["a", "b"], vec!["x", "y"], vec!["m", "n"]]);
// parses_to!(empty_lines_crlf, "\r\n\r\n\r\n\r\n", vec![]);
// parses_to!(empty_lines_interspersed_crlf,
//            "\r\n\r\na,b\r\n\r\n\r\nx,y\r\n\r\n\r\nm,n\r\n",
//            vec![vec!["a", "b"], vec!["x", "y"], vec!["m", "n"]]);
// parses_to!(empty_lines_mixed, "\r\n\n\r\n\n", vec![]);
// parses_to!(empty_lines_interspersed_mixed,
//            "\n\r\na,b\r\n\n\r\nx,y\r\n\n\r\nm,n\r\n",
//            vec![vec!["a", "b"], vec!["x", "y"], vec!["m", "n"]]);
// parses_to!(empty_lines_cr, "\r\r\r\r", vec![]);
// parses_to!(empty_lines_interspersed_cr, "\r\ra,b\r\r\rx,y\r\r\rm,n\r",
//            vec![vec!["a", "b"], vec!["x", "y"], vec!["m", "n"]]);

parses_to!(quote_empty, "\"\"", vec![vec![""]]);
parses_to!(quote_lf, "\"\"\n", vec![vec![""]]);
parses_to!(quote_space, "\" \"", vec![vec![" "]]);
parses_to!(quote_inner_space, "\" a \"", vec![vec![" a "]]);
fail_parses_to!(quote_outer_space, "  \"a\"  ", vec![vec!["  \"a\"  "]]);

// parses_to!(quote_change, "zaz", vec![vec!["a"]],
//            |rdr: Csv<_>| rdr.quote(b'z'));

parses_to!(delimiter_tabs, "a\tb", vec![vec!["a", "b"]],
           |rdr: Csv<_>| rdr.delimiter(b'\t'));
parses_to!(delimiter_weird, "azb", vec![vec!["a", "b"]],
           |rdr: Csv<_>| rdr.delimiter(b'z'));

parses_to!(headers_absent, "a\nb", vec![vec!["b"]],
           |rdr: Csv<_>| rdr.has_header(true));
 
parses_to!(flexible_rows, "a\nx,y", vec![vec!["a"], vec!["x", "y"]],
           |rdr: Csv<_>| rdr.flexible(true));
parses_to!(flexible_rows2, "a,b\nx", vec![vec!["a", "b"], vec!["x"]],
           |rdr: Csv<_>| rdr.flexible(true));

fail_parses_to!(nonflexible, "a\nx,y", vec![]);
fail_parses_to!(nonflexible2, "a,b\nx", vec![]);

#[derive(Debug, RustcDecodable, RustcEncodable, PartialEq, Eq)]
enum Val { Unsigned(usize), Signed(isize), Bool(bool) }

decodes_to!(decode_int, "1", (usize,), vec![(1usize,)]);
decodes_to!(decode_many_int, "1,2", (usize, i16), vec![(1usize, 2i16)]);
decodes_to!(decode_float, "1,1.0,1.5",
            (f64, f64, f64), vec![(1f64, 1.0, 1.5)]);
decodes_to!(decode_char, "a", (char), vec![('a')]);
decodes_to!(decode_str, "abc", (String,), vec![("abc".into(),)]);

decodes_to!(decode_opt_int, "a", (Option<usize>,), vec![(None,)]);
decodes_to!(decode_opt_float, "a", (Option<f64>,), vec![(None,)]);
decodes_to!(decode_opt_char, "ab", (Option<char>,), vec![(None,)]);
decodes_to!(decode_opt_empty, "\"\"", (Option<String>,), vec![(None,)]);

// decodes_to!(decode_val, "false,-5,5", (Val, Val, Val),
//             vec![(Val::Bool(false), Val::Signed(-5), Val::Unsigned(5))]);
decodes_to!(decode_opt_val, "1.0", (Option<Val>,), vec![(None,)]);

decodes_to!(decode_tail, "abc,1,2,3,4", (String, Vec<usize>),
            vec![("abc".into(), vec![1usize, 2, 3, 4])]);

#[derive(Debug, RustcDecodable, RustcEncodable, PartialEq, Eq)]
enum MyEnum { Enum1, Enum2 }
decodes_to!(decode_myenum, "Enum1,Enum1,Enum2", (MyEnum, MyEnum, MyEnum),
            vec![(MyEnum::Enum1, MyEnum::Enum1, MyEnum::Enum2)]);
#[test]
fn no_headers_no_skip_one_record() {
    let mut d = Csv::from_string("a,b");
    d.headers();
    let rows: Vec<_> = d.collect();
    assert_eq!(rows.len(), 1);
}

// #[test]
// fn no_headers_first_record() {
//     let mut d = Csv::from_string("a,b");
//     let r = d.headers();
//     assert_eq!(r, vec!("a".to_string(), "b".to_string()));
// }

#[test]
fn no_headers_no_skip() {
    let mut d = Csv::from_string("a,b\nc,d");
    d.headers();
    let rows: Vec<_> = d.collect();
    assert_eq!(rows.len(), 2);
}

#[test]
fn byte_strings() {
    let mut d = Csv::from_string("abc,xyz");
    let r = d.next().unwrap().unwrap();
    let c = r.bytes_columns().collect::<Vec<_>>();
    assert_eq!(c, vec![b"abc", b"xyz"]);
}

#[test]
fn byte_strings_invalid_utf8() {
    let mut d = Csv::from_reader(&b"a\xffbc,xyz"[..]);
    let r = d.next().unwrap().unwrap();
    let c = r.bytes_columns().collect::<Vec<_>>();
    assert_eq!(c, vec![&b"a\xffbc"[..], &b"xyz"[..]]);
}

#[test]
#[should_panic]
fn invalid_utf8() {
    let mut d = Csv::from_reader(&b"a\xffbc,xyz"[..]);
    let _ = d.next().unwrap().unwrap().columns().unwrap();
}

#[test]
fn seeking() {
    let data = "1,2\n3,4\n5,6\n";
    let mut buf = io::Cursor::new(data.as_bytes().to_vec());

    {
        let d = Csv::from_reader(Read::by_ref(&mut buf));
        let vals = d.map(|r| r.unwrap().decode::<(usize, usize)>().unwrap()).collect::<Vec<_>>();
        assert_eq!(vals, vec!((1, 2), (3, 4), (5, 6)));
    }

    buf.seek(io::SeekFrom::Start(0)).unwrap();
    {
        let d = Csv::from_reader(Read::by_ref(&mut buf));
        let vals = d.map(|r| r.unwrap().decode::<(usize, usize)>().unwrap()).collect::<Vec<_>>();
        assert_eq!(vals, vec!((1, 2), (3, 4), (5, 6)));
    }
}
