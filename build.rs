//! Compiles data/engwebp_vpl.txt (one "GEN 1:1 text" line per verse) into
//! Rust source: the verses as a static array plus a chapter index. The lookup
//! arithmetic in lib.rs relies on the checks here: books arrive whole and in
//! order, and chapters and verses count up from 1 without gaps.

use std::{env, fmt::Write as _, fs, path::Path};

const SOURCE: &str = "data/engwebp_vpl.txt";

fn main() {
    println!("cargo::rerun-if-changed={SOURCE}");
    let text = fs::read_to_string(SOURCE).expect("read the Bible text");

    let mut verses = String::new();
    let mut chapter_starts = Vec::new(); // index in VERSES of each chapter's first verse
    let mut book_starts = Vec::new(); // index in chapter_starts of each book's first chapter
    let mut book = "";
    let (mut last_chapter, mut last_verse) = (0u8, 0u8);
    let mut count = 0;

    for (n, line) in text.lines().enumerate() {
        let fail = |why: &str| -> ! { panic!("{SOURCE}:{}: {why}: {line}", n + 1) };
        let (code, rest) = line.split_once(' ').unwrap_or_else(|| fail("no reference"));
        let (reference, verse_text) = rest.split_once(' ').unwrap_or((rest, ""));
        let (chapter, verse) = reference
            .split_once(':')
            .unwrap_or_else(|| fail("no chapter:verse"));
        let chapter: u8 = chapter.parse().unwrap_or_else(|_| fail("bad chapter"));
        let verse: u8 = verse.parse().unwrap_or_else(|_| fail("bad verse"));

        if code != book {
            book = code;
            book_starts.push(chapter_starts.len());
            (last_chapter, last_verse) = (0, 0);
        }
        if chapter != last_chapter {
            if chapter != last_chapter + 1 || verse != 1 {
                fail("chapters must start at verse 1 and follow each other");
            }
            chapter_starts.push(count);
        } else if verse != last_verse + 1 {
            fail("verses must follow each other");
        }
        (last_chapter, last_verse) = (chapter, verse);

        writeln!(
            verses,
            "Verse {{ book: Book::ALL[{}], chapter: {chapter}, verse: {verse}, text: {:?} }},",
            book_starts.len() - 1,
            verse_text.trim_end(),
        )
        .unwrap();
        count += 1;
    }
    assert_eq!(
        book_starts.len(),
        66,
        "{SOURCE} must hold the 66 books, each once"
    );
    chapter_starts.push(count);
    book_starts.push(chapter_starts.len() - 1);

    let source = format!(
        "/// Every verse of the World English Bible, in canonical order.\n\
         pub static VERSES: [Verse; {count}] = [\n{verses}];\n\
         /// Index in VERSES of each chapter's first verse, then the verse count.\n\
         const CHAPTER_STARTS: [usize; {}] = {chapter_starts:?};\n\
         /// Index in CHAPTER_STARTS of each book's first chapter, then the chapter count.\n\
         const BOOK_STARTS: [usize; 67] = {book_starts:?};\n",
        chapter_starts.len(),
    );
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("bible.rs"),
        source,
    )
    .unwrap();
}
