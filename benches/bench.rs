#![feature(test)]

extern crate quick_csv as csv;
extern crate rustc_serialize;
extern crate test;

use test::Bencher;
use csv::Csv;

static CSV_DATA: &'static str = "./examples/data/bench.csv";

#[bench]
fn str_records(b: &mut Bencher) {
    b.iter(|| {
        let dec = Csv::from_file(CSV_DATA).unwrap();
        for row in dec.into_iter() {
            for c in row.columns() {
                let _ = c;    
            }
        }
    })
}
//
//#[bench]
//fn byte_records(b: &mut Bencher) {
//    let mut data = file_to_mem(CSV_DATA);
//    b.bytes = data.get_ref().len() as u64;
//    b.iter(|| {
//        let mut dec = reader(&mut data);
//        for r in dec.byte_records() { let _ = r.unwrap(); }
//    })
//}
//
//#[bench]
//fn string_records(b: &mut Bencher) {
//    let mut data = file_to_mem(CSV_DATA);
//    b.bytes = data.get_ref().len() as u64;
//    b.iter(|| {
//        let mut dec = reader(&mut data);
//        for r in dec.records() { let _ = r.unwrap(); }
//    })
//}

#[allow(dead_code)]
#[derive(RustcDecodable)]
struct Play {
    gameid: String,
    qtr: i32,
    min: Option<i32>,
    sec: Option<i32>,
    team_off: String,
    team_def: String,
    down: Option<i32>,
    togo: Option<i32>,
    ydline: Option<i32>,
    description: String,
    offscore: i32,
    defscore: i32,
    season: i32,
}

#[bench]
fn decoded_records(b: &mut Bencher) {
    b.iter(|| {
        let dec = Csv::from_file(CSV_DATA).unwrap();
        for row in dec.into_iter() {
            if let Ok(p) = row.decode::<Play>() {
                let _ = p;
            }
        }
    })
}
