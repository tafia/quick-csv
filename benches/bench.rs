#![feature(test)]

extern crate quick_csv as csv;
extern crate rustc_serialize;
extern crate test;

use std::fmt::{Debug, Display};
use std::fs;
use std::io::Read;
use test::Bencher;
use csv::Csv;

static CSV_DATA: &'static str = "./examples/data/bench.csv";

fn ordie<T, E: Debug+Display>(r: Result<T, E>) -> T {
    r.or_else(|e: E| -> Result<T, E> { panic!(format!("{:?}", e)) }).unwrap()
}

fn file_to_mem(fp: &str) -> Vec<u8> {
    let mut f = ordie(fs::File::open(fp));
    let mut bs = vec![];
    ordie(f.read_to_end(&mut bs));
    bs
}

#[bench]
fn str_records(b: &mut Bencher) {
    let data = file_to_mem(CSV_DATA);
    b.bytes = data.len() as u64;
    b.iter(|| {
        let dec = Csv::from_reader(&*data);
        for row in dec.into_iter() {
            for c in row.unwrap().columns() {
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
    let data = file_to_mem(CSV_DATA);
    b.bytes = data.len() as u64;
    b.iter(|| {
        let dec = Csv::from_reader(&*data);
        for row in dec.skip(1).into_iter() {
            let _ = row.unwrap().decode::<Play>().unwrap();
        }
    })
}
