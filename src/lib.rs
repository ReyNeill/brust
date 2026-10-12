//! The Bible, compiled: every verse of the World English Bible as typed,
//! static Rust data, with references checked when your program builds.
//!
//! ```
//! let verse = brust::verse!(John 11:35);
//! assert_eq!(verse.text, "Jesus wept.");
//!
//! let psalm = brust::lookup("Psalm 23").unwrap();
//! assert_eq!(psalm.len(), 6);
//! ```

use std::fmt;

include!(concat!(env!("OUT_DIR"), "/bible.rs"));

/// One verse. The few verses the WEB leaves out or moves elsewhere (Luke
/// 17:36, for example) keep their number with empty text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verse {
    pub book: Book,
    pub chapter: u8,
    pub verse: u8,
    pub text: &'static str,
}

impl fmt::Display for Verse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}:{} {}",
            self.book, self.chapter, self.verse, self.text
        )
    }
}

macro_rules! books {
    ($($book:ident $name:literal,)*) => {
        /// The 66 books of the Protestant canon, in order.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Book { $($book),* }

        impl Book {
            pub const ALL: [Book; 66] = [$(Book::$book),*];

            /// The book's name as the WEB writes it, e.g. "1 John".
            pub const fn name(self) -> &'static str {
                match self { $(Book::$book => $name),* }
            }
        }
    };
}

books! {
    Genesis "Genesis",
    Exodus "Exodus",
    Leviticus "Leviticus",
    Numbers "Numbers",
    Deuteronomy "Deuteronomy",
    Joshua "Joshua",
    Judges "Judges",
    Ruth "Ruth",
    FirstSamuel "1 Samuel",
    SecondSamuel "2 Samuel",
    FirstKings "1 Kings",
    SecondKings "2 Kings",
    FirstChronicles "1 Chronicles",
    SecondChronicles "2 Chronicles",
    Ezra "Ezra",
    Nehemiah "Nehemiah",
    Esther "Esther",
    Job "Job",
    Psalms "Psalms",
    Proverbs "Proverbs",
    Ecclesiastes "Ecclesiastes",
    SongOfSolomon "Song of Solomon",
    Isaiah "Isaiah",
    Jeremiah "Jeremiah",
    Lamentations "Lamentations",
    Ezekiel "Ezekiel",
    Daniel "Daniel",
    Hosea "Hosea",
    Joel "Joel",
    Amos "Amos",
    Obadiah "Obadiah",
    Jonah "Jonah",
    Micah "Micah",
    Nahum "Nahum",
    Habakkuk "Habakkuk",
    Zephaniah "Zephaniah",
    Haggai "Haggai",
    Zechariah "Zechariah",
    Malachi "Malachi",
    Matthew "Matthew",
    Mark "Mark",
    Luke "Luke",
    John "John",
    Acts "Acts",
    Romans "Romans",
    FirstCorinthians "1 Corinthians",
    SecondCorinthians "2 Corinthians",
    Galatians "Galatians",
    Ephesians "Ephesians",
    Philippians "Philippians",
    Colossians "Colossians",
    FirstThessalonians "1 Thessalonians",
    SecondThessalonians "2 Thessalonians",
    FirstTimothy "1 Timothy",
    SecondTimothy "2 Timothy",
    Titus "Titus",
    Philemon "Philemon",
    Hebrews "Hebrews",
    James "James",
    FirstPeter "1 Peter",
    SecondPeter "2 Peter",
    FirstJohn "1 John",
    SecondJohn "2 John",
    ThirdJohn "3 John",
    Jude "Jude",
    Revelation "Revelation",
}

impl Book {
    /// Finds a book by name, ignoring case and spaces; "Psalm" also names Psalms.
    const fn named(name: &[u8]) -> Option<Book> {
        if loose_eq(name, b"Psalm") {
            return Some(Book::Psalms);
        }
        let mut i = 0;
        while i < Book::ALL.len() {
            if loose_eq(name, Book::ALL[i].name().as_bytes()) {
                return Some(Book::ALL[i]);
            }
            i += 1;
        }
        None
    }

    const fn chapters(self) -> usize {
        BOOK_STARTS[self as usize + 1] - BOOK_STARTS[self as usize]
    }

    /// Index in CHAPTER_STARTS of one of this book's chapters, counted from 1.
    const fn chapter_index(self, chapter: usize) -> usize {
        BOOK_STARTS[self as usize] + chapter - 1
    }

    const fn verses_in(self, chapter: usize) -> usize {
        let k = self.chapter_index(chapter);
        CHAPTER_STARTS[k + 1] - CHAPTER_STARTS[k]
    }
}

impl fmt::Display for Book {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Why a reference names no verses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Malformed,
    UnknownBook,
    NoSuchChapter,
    NoSuchVerse,
}

impl Error {
    const fn message(self) -> &'static str {
        match self {
            Error::Malformed => "expected a reference like John 3:16, Psalm 23 or Jude",
            Error::UnknownBook => "no such book",
            Error::NoSuchChapter => "no such chapter",
            Error::NoSuchVerse => "no such verse",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for Error {}

/// The verses a reference names: a whole book ("Jude"), a chapter
/// ("Psalm 23") or one verse ("John 3:16"). Book names ignore case and spaces.
pub fn lookup(reference: &str) -> Result<&'static [Verse], Error> {
    let (start, end) = Reference::parse(reference)?.range();
    Ok(&VERSES[start..end])
}

/// One verse, resolved while your program compiles: a reference that does
/// not exist is a build error.
///
/// ```
/// assert!(brust::verse!(1 John 4:8).text.ends_with("God is love."));
/// ```
///
/// ```compile_fail
/// brust::verse!(John 3:99);
/// ```
#[macro_export]
macro_rules! verse {
    ($($reference:tt)+) => {
        &$crate::VERSES[const { $crate::verse_index(stringify!($($reference)+)) }]
    };
}

#[doc(hidden)]
pub const fn verse_index(reference: &str) -> usize {
    match Reference::parse(reference) {
        Ok(reference @ Reference::Verse(..)) => reference.range().0,
        Ok(_) => panic!("verse! names one verse, like John 3:16"),
        Err(error) => panic!("{}", error.message()),
    }
}

/// A reference whose chapter and verse are known to exist.
#[derive(Clone, Copy)]
enum Reference {
    Book(Book),
    Chapter(Book, usize),
    Verse(Book, usize, usize),
}

impl Reference {
    /// Parses "Jude", "Psalm 23" or "1 John 4:8". Const, so `verse!` can run it
    /// at compile time; spacing around the colon is free because `stringify!`
    /// writes "John 3 : 16".
    const fn parse(reference: &str) -> Result<Reference, Error> {
        let s = reference.as_bytes();
        let end = trim_end(s, s.len());
        let Some((last, last_start)) = number_before(s, end) else {
            return match Book::named(s.split_at(end).0) {
                Some(book) => Ok(Reference::Book(book)),
                None => Err(Error::UnknownBook),
            };
        };
        let colon = trim_end(s, last_start);
        let (book_end, chapter, verse) = if colon > 0 && s[colon - 1] == b':' {
            let Some((chapter, start)) = number_before(s, trim_end(s, colon - 1)) else {
                return Err(Error::Malformed);
            };
            (start, chapter, Some(last))
        } else {
            (last_start, last, None)
        };
        let Some(book) = Book::named(s.split_at(book_end).0) else {
            return Err(Error::UnknownBook);
        };
        if chapter == 0 || chapter > book.chapters() {
            return Err(Error::NoSuchChapter);
        }
        match verse {
            None => Ok(Reference::Chapter(book, chapter)),
            Some(verse) if verse == 0 || verse > book.verses_in(chapter) => Err(Error::NoSuchVerse),
            Some(verse) => Ok(Reference::Verse(book, chapter, verse)),
        }
    }

    /// The verses this reference names, as a start..end index range into VERSES.
    const fn range(self) -> (usize, usize) {
        match self {
            Reference::Book(book) => (
                CHAPTER_STARTS[BOOK_STARTS[book as usize]],
                CHAPTER_STARTS[BOOK_STARTS[book as usize + 1]],
            ),
            Reference::Chapter(book, chapter) => {
                let k = book.chapter_index(chapter);
                (CHAPTER_STARTS[k], CHAPTER_STARTS[k + 1])
            }
            Reference::Verse(book, chapter, verse) => {
                let i = CHAPTER_STARTS[book.chapter_index(chapter)] + verse - 1;
                (i, i + 1)
            }
        }
    }
}

const fn trim_end(s: &[u8], mut end: usize) -> usize {
    while end > 0 && s[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    end
}

/// Reads the digits ending at `end`, returning their value (capped well past
/// any real chapter or verse) and where they start.
const fn number_before(s: &[u8], end: usize) -> Option<(usize, usize)> {
    let mut start = end;
    while start > 0 && s[start - 1].is_ascii_digit() {
        start -= 1;
    }
    if start == end {
        return None;
    }
    let (mut value, mut i) = (0, start);
    while i < end {
        value = value * 10 + (s[i] - b'0') as usize;
        if value > 1000 {
            value = 1000;
        }
        i += 1;
    }
    Some((value, start))
}

/// Compares ASCII case-insensitively, ignoring whitespace: "1john" matches "1 John".
const fn loose_eq(a: &[u8], b: &[u8]) -> bool {
    let (mut i, mut j) = (0, 0);
    loop {
        while i < a.len() && a[i].is_ascii_whitespace() {
            i += 1;
        }
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        if i == a.len() || j == b.len() {
            return i == a.len() && j == b.len();
        }
        if !a[i].eq_ignore_ascii_case(&b[j]) {
            return false;
        }
        i += 1;
        j += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Catches the text, the book order and the chapter index drifting apart.
    #[test]
    fn references_land_on_their_text() {
        assert_eq!(
            verse!(Genesis 1:1).text,
            "In the beginning, God created the heavens and the earth."
        );
        assert_eq!(verse!(John 11:35).text, "Jesus wept.");
        assert_eq!(verse!(Jude 1:25).book, Book::Jude);
        assert_eq!(
            verse!(Revelation 22:21).text,
            "The grace of the Lord Jesus Christ be with all the saints. Amen."
        );
    }

    #[test]
    fn lookup_names_books_chapters_and_verses() {
        for book in Book::ALL {
            assert_eq!(lookup(book.name()).unwrap()[0].book, book, "{book}");
        }
        assert_eq!(lookup("Jude").unwrap().len(), 25);
        assert_eq!(lookup("psalm 23").unwrap().len(), 6);
        assert_eq!(lookup("1john 4:8").unwrap()[0].verse, 8);
        assert_eq!(lookup("Hezekiah 1:1"), Err(Error::UnknownBook));
        assert_eq!(lookup("Genesis 51"), Err(Error::NoSuchChapter));
        assert_eq!(lookup("John 3:37"), Err(Error::NoSuchVerse));
        assert_eq!(lookup(":16"), Err(Error::Malformed));
    }
}
