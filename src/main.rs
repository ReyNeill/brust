use std::io::Write;
use std::process::ExitCode;

/// `brust John 3:16` prints a verse, `brust Psalm 23` a chapter, `brust Jude` a book.
fn main() -> ExitCode {
    let reference = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    if reference.is_empty() {
        eprintln!("usage: brust <reference>, e.g. brust John 3:16, brust Psalm 23, brust Jude");
        return ExitCode::from(2);
    }
    match brust::lookup(&reference) {
        Ok(verses) => {
            let mut out = std::io::stdout().lock();
            // Stop quietly when the reader goes away (brust Psalms | head).
            for verse in verses {
                if writeln!(out, "{verse}").is_err() {
                    break;
                }
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("brust: {error}: {reference}");
            ExitCode::FAILURE
        }
    }
}
