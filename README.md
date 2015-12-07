# quick-csv

Fast Csv reader which performs **very** well.

## Example

First, create a `Csv`, either from a file or from a `BufRead` reader.

```rust
extern crate quick_csv;

fn main() {
    let csv = quick_csv::Csv::from_file("test.csv").unwrap();
    for row in csv.into_iter() {
        // work on csv row ...
    }
}
```

`Row` is on the other hand provides 3 methods to access csv columns:
- `columns`: 
  - iterator over columns.
  - Iterator item is a `&str`, which means you only have to `parse()` it to the needed type and you're done

  ```rust
  let mut cols = row.columns();
  let fifth = cols.nth(5).unwrap().parse::<f64>();
  println!("Doubled fifth column: {}", fifth * 2.0);
  ```

- `decode`:
  - deserialize into you `Decodable` struct, a-la rust-csv.
  - most convenient way to deal with your csv data

  ```rust
  if let Ok((col1, col2, col3)) = rust::decode::<(String, u64, f64)>() {
      println!("col1: '{}',, col2: {}, col3: {}", col1, col2, col3);
  }
  ``` 

- `bytes_columns`:
  - similar to `columns` but columns are of type `&[u8]`, which means you may want to convert it to &str first
  - performance gain compared to `columns` is minimal, use it only if you *really* need to as it is less convenient

## Benchmarks

I mainly benchmarked this to [rust-csv](https://github.com/BurntSushi/rust-csv), which is supposed to be already very fast.
I tried to provide similar methods even if I don't have `raw` version.

- quick-csv
```
test bytes_records   ... bench:   4,799,100 ns/iter (+/- 119,638) = 283 MB/s
test decoded_records ... bench:  10,813,981 ns/iter (+/- 423,758) = 125 MB/s
test str_records     ... bench:   5,318,667 ns/iter (+/- 133,288) = 256 MB/s
```

- rust-csv
```
test byte_records    ... bench:  10,528,780 ns/iter (+/- 2,080,735) = 128 MB/s
test decoded_records ... bench:  18,458,365 ns/iter (+/- 2,415,059) = 73 MB/s
test raw_records     ... bench:   6,555,447 ns/iter (+/- 830,423) = 207 MB/s
test string_records  ... bench:  12,813,284 ns/iter (+/- 2,324,424) = 106 MB/s
```

## License

MIT
