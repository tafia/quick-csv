extern crate quick_csv as csv;

fn main() {
    let huge = ::std::env::args().nth(1).unwrap();
    let rdr = csv::Csv::from_file(huge).unwrap();
    
    let mut count = 0;
    for r in rdr {        
        match r {
            Ok(r) => count += r.len(),
            Err(e) => panic!("{:?}", e),
        }
    }
    println!("count: {}", count);
}
