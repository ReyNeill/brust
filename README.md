# brust

The Bible in Rust! Every verse of the World English Bible (eBible.org) is built into a
Rust binary as typed, static data, and references are checked when your
program compiles. The build fails if a book is missing, or a chapter or verse number is skipped.

A release build takes about 3 seconds and produces a 5.2 MB binary!

There is also a CLI. ```brust John 11:35``` prints a verse, ```brust Psalm 23``` a chaper, ```brust Jude``` a whole book.

Install with `cargo install --git https://github.com/ReyNeill/brust`.

## As a library

```rust
// Resolved at compile time: a reference that does not exist is a build error.
let verse = brust::verse!(1 John 4:8);
println!("{verse}");

// Resolved at run time.
for verse in brust::lookup("Psalm 23")? {
    println!("{verse}");
}
```

`brust::VERSES` is the whole text as a `[Verse; 31103]`, where each verse
holds its book, chapter, number and text.

## How it works

`build.rs` reads [`data/engwebp_vpl.txt`](data/engwebp_vpl.txt) (one verse per
line), checks that every chapter and verse follows the one before, and writes
the text and a chapter index as Rust source. Reference parsing is a
`const fn`, so `verse!(John 3:99)` stops the build with `no such verse`.

The text is not refactored. Chronicles already retells much of Samuel and
Kings, and Matthew, Mark and Luke share much of their text, but the repetition
is deliberate: they are separate accounts of the same events.

## Text and license

The World English Bible (2020 stable text, from [eBible.org](https://ebible.org/web/))
is in the public domain and is included unchanged. "World English Bible" is a
trademark of eBible.org. The code is MIT licensed.
