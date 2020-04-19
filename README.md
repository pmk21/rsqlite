# RSQLite

![](https://github.com/pmk21/rsqlite/workflows/rsqlite/badge.svg)

A simple SQLite clone in Rust. This is basically a translation of the C code present on [this](https://cstack.github.io/db_tutorial/) brilliant tutorial into Rust(not fully idiomatic). This code contains implementation only upto Part 5 in the tutorial.

This a very simple database and is a small project I took up to gain experience with Rust.

## Requirements

Having [`rustup`](https://www.rust-lang.org/tools/install) and [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html) should be enough to get this up and running.

## Usage

* In the base directory of the repository type the command `$cargo run <filename>`, the database will be stored in the given file and will also load values(if present) from the given file.

* Once the program is up and running a prompt `db >` will appear, there you can execute database commands.

* Supported commands are(which are only a few!) -

  * `.exit` - To exit the program.
  
  * `insert <id> <username> <email>` - Inserts the given values into the database. The values are persisted on the disk.
  
  * `select` - Displays all the rows present in the database.

## Documentation

Documentation of the various modules and functions can be seen by typing `$cargo doc --open` in the base directory of the repository.

## Tests

A few simple tests can be run with `$cargo test -- --test-threads=1`.

## License

This project is licensed under the MIT License.

MIT © Prithvi MK
