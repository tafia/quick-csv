To run a micro benchmark using the 1.4MB `examples/data/bench.csv` data:

    go test -bench '.*'

To run similar benchmarks for Rust (on the same data, but will benchmark each
of the four access patterns), run `cargo bench` in the project root directory.

To run the super huge benchmark (3.6GB), you'll need to download the zip from
http://www2.census.gov/acs2010_5yr/pums/csv_pus.zip and put `ss10pusa.csv` in
`../examples/data/ss10pusa.csv`.

Then compile and run:

    go build -o huge-go
    time ./huge-go

To run the huge benchmark for Rust, make sure `ss10pusa.csv` is in the same
location as above and run:

    rustc -C opt-level=3 -C lto -L ../target/release/ -L ../target/release/deps/ huge.rs -o huge-rust
    time ./huge-rust

To get libraries in `../target/release/`, run `cargo build --release` in the
project root directory.

(Please make sure that one CPU is pegged when running this benchmark. If it
isn't, you're probably just testing the speed of your disk.)

