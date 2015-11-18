# quick-csv

Fast Csv reader which performs surprisingly **very** well.

The implementation is largely inspired on [rust-csv](https://github.com/BurntSushi/rust-csv).
The main change is to directly rely on `BufReader` and `Split`

## Example

First, create a `Csv`, either from a file or from a `BufRead` reader.
`Csv` is nothing but a wrapper over a `Lines` iterator so you can use any iterator function
to get directly the row you want (or do nothing but just counting records count).


```rust
extern crate quick_csv;

fn main() {
    let csv = quick_csv::Csv::from_file("test.csv");
    for row in csv.into_iter() {
        // work on csv row ...
    }
}
```

`Row` is on the other hand provides 2 methods to access csv columns:
- `columns`: 
  - iterator over columns, implemented as a simple `Split` iterator.
  - Iterator item is a `&str`, which means you only have to `parse()` it to the needed type and you're done
  - achieves better performance than rust-csv *raw* method, while providing a `&str` instead of `[u8]`!

  ```rust
  let mut cols = row.columns();
  let fifth = cols.nth(5).unwrap().parse::<f64>();
  println!("Doubled fifth column: {}", fifth * 2.0);
  ```

- `decode`:
  - deserialize into you `Decodable` struct, a-la rust-csv. Most convenient way to deal with your csv data
  - performance penalty compared to `columns` is very low (less than 30% on my test)

  ```rust
  if let Ok((col1, col2, col3)) = rust::decode::<(String, u64, f64)>() {
      println!("col1: '{}',, col2: {}, col3: {}", col1, col2, col3);
  }
  ``` 

## Limitation

This is really a WIP so many funtionalities of rust-csv are not implemented.
