//! The lexicon compiler: from the text of a language file to the packed string table
//! (`strings::StringTable` reads it) and the counts the game logs.
//!
//! The language file: one message a line, a label and then the text; `;` starts a comment.
//! The text is cut into words: a run of letters, digits and bytes of 0x80 and more (with `'`
//! and `-` inside it); any other character by itself; `{...}`, `\x` (and `\#hh`) and `~ddd`
//! each as one word; the blanks between two words as one word, a space. Every different word
//! is kept once, and every different message, as the list of its words, once.
//! The table is read by `strings::table`: an index of dwords (one per message, then one per
//! word), then a zero byte, the words' bytes shortest first, a padding byte if that leaves an
//! odd offset, and the messages of more than one word, shortest first, as the index numbers of
//! their words. A message of one word shares that word's entry. Input that would be an error
//! or run on is refused.

use crate::Invalid;
use std::collections::HashMap;

/// What `compile` makes.
#[derive(Debug, PartialEq, Eq)]
pub struct Compiled {
    pub table: Vec<u8>,
    /// The counts the game logs.
    pub padded: u32,
    pub lines: u32,
    pub messages: u32,
    pub source_size: u32,
    pub words_encountered: u32,
    pub words_stored: u32,
    pub highest_word: u32,
    pub phrases_encountered: u32,
    pub phrases_stored: u32,
    pub highest_phrase: u32,
    pub index_size: u32,
    /// Messages with nothing in them (`{}`): they share one record of the game's own.
    pub empty_messages: u32,
}

/// The table has room for 0xFFD0 bytes.
pub const TABLE_ROOM: usize = 0xFFD0;
/// The word the blanks between two words become, before it is unescaped to a space.
const SPACE: &[u8] = b"\\s";

struct Word {
    /// As written in the file, which is what words are told apart by.
    raw: Vec<u8>,
    /// The bytes that go into the table.
    text: Vec<u8>,
    id: u32,
    /// Offset and flag, as in the index.
    entry: u32,
}

struct Phrase {
    words: Vec<usize>,
    entry: u32,
}

/// The memory the compiler may use; running out is refused.
struct Arena(u32);

impl Arena {
    fn take(&mut self, bytes: u32) -> Result<(), Invalid> {
        if bytes > self.0 {
            return Err(Invalid("the lexicon's memory ran out"));
        }
        self.0 -= bytes;
        Ok(())
    }
}

/// One word of a message's text, as the tokeniser hands it out.
struct Token {
    /// 0 = the text has ended (or an empty `{}` was met).
    length: u32,
    /// Where the word's bytes are; None for the space between two words.
    start: Option<usize>,
    /// Where the scan stands when the length is 0.
    left: usize,
    after: usize,
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte >= 0x80
}

struct Source<'a>(&'a [u8]);

impl Source<'_> {
    /// The byte at a place; past the text, 0.
    fn at(&self, place: usize) -> u8 {
        self.0.get(place).copied().unwrap_or(0)
    }

    /// For loops that do not stop at a zero byte: past the text they would run on (refused).
    fn within(&self, place: usize) -> Result<u8, Invalid> {
        if place > self.0.len() {
            return Err(Invalid(
                "the language file ends in the middle of a comment or a group",
            ));
        }
        Ok(self.at(place))
    }

    /// The next word from `scan` on.
    fn next_word(&self, scan: usize) -> Result<Token, Invalid> {
        let mut place = scan;
        loop {
            let byte = self.at(place);
            if matches!(byte, 0 | 0x0A | 0x0D | b';') {
                return Ok(Token {
                    length: 0,
                    start: None,
                    left: scan,
                    after: place,
                });
            }
            if byte > 0x20 {
                break;
            }
            place += 1;
        }
        if place != scan {
            return Ok(Token {
                length: 2,
                start: None,
                left: scan,
                after: place,
            });
        }
        let start = place;
        let first = self.at(place);
        place += 1;
        let escaped = |place: &mut usize, length: &mut u32| -> Result<(), Invalid> {
            let kind = self.at(*place);
            *place += 1;
            if kind <= 0x20 {
                return Err(Invalid("language file: Invalid special symbol type"));
            }
            if kind == b'#' {
                if self.at(*place) <= 0x20 || self.at(*place + 1) <= 0x20 {
                    return Err(Invalid("language file: Invalid special symbol type"));
                }
                *length += 2;
                *place += 2;
            }
            Ok(())
        };
        match first {
            b'{' => {
                let content = place;
                // One for every byte read, the closing brace too (not part of the word).
                let mut read = 0u32;
                loop {
                    read += 1;
                    let byte = self.within(place)?;
                    place += 1;
                    match byte {
                        b'{' | b';' | 0x0A | 0x0D => {
                            return Err(Invalid("language file: Missing start or end of group"));
                        }
                        b'\\' => {
                            read += 1;
                            escaped(&mut place, &mut read)?;
                        }
                        b'}' => break,
                        _ => {}
                    }
                }
                Ok(Token {
                    length: read - 1,
                    start: Some(content),
                    left: content,
                    after: place,
                })
            }
            b'}' => Err(Invalid("language file: Missing start or end of group")),
            b'\\' => {
                let mut length = 2;
                escaped(&mut place, &mut length)?;
                Ok(Token {
                    length,
                    start: Some(start),
                    left: start,
                    after: place,
                })
            }
            b'~' => {
                for _ in 0..3 {
                    if self.at(place) <= 0x20 {
                        return Err(Invalid("language file: Invalid DBCS type"));
                    }
                    place += 1;
                }
                Ok(Token {
                    length: 4,
                    start: Some(start),
                    left: start,
                    after: place,
                })
            }
            byte if !is_word_byte(byte) => Ok(Token {
                length: 1,
                start: Some(start),
                left: start,
                after: place,
            }),
            _ => {
                let mut length = 0;
                loop {
                    length += 1;
                    let byte = self.at(place);
                    place += 1;
                    if !(is_word_byte(byte) || byte == b'\'' || byte == b'-') {
                        place -= 1;
                        break;
                    }
                }
                Ok(Token {
                    length,
                    start: Some(start),
                    left: start,
                    after: place,
                })
            }
        }
    }
}

/// After its `#`: two hex digits.
fn hex_pair(digits: &[u8]) -> Result<u8, Invalid> {
    let mut value = 0u8;
    for n in 0..2 {
        let digit = match digits.get(n).copied().unwrap_or(0) {
            byte @ b'0'..=b'9' => byte - b'0',
            byte @ b'A'..=b'F' => byte - b'A' + 10,
            byte @ b'a'..=b'f' => byte - b'a' + 10,
            _ => return Err(Invalid("language file: Invalid hex string")),
        };
        value = value << 4 | digit;
    }
    Ok(value)
}

/// For one word: the escapes of the file become the bytes of the table.
fn unescaped(raw: &[u8]) -> Result<Vec<u8>, Invalid> {
    let mut text = raw.to_vec();
    let mut place = 0;
    while place < text.len() {
        let byte = text[place];
        place += 1;
        // A zero byte ends the word.
        if byte == 0 {
            break;
        }
        if byte == b'\\' {
            let from = place - 1;
            let kind = text.get(place).copied().unwrap_or(0);
            place += 1;
            let value = match kind {
                b'n' => 0x0A,
                b's' => 0x20,
                b't' => 9,
                b'1'..=b'9' => kind - b'0',
                b'#' => {
                    let value = hex_pair(text.get(place..).unwrap_or(&[]))?;
                    place += 2;
                    value
                }
                other => other,
            };
            text.splice(from..place.min(text.len()), [value]);
            place = from + 1;
        } else if byte == b'~' {
            let from = place - 1;
            let mut number = 0u32;
            for n in 0..3 {
                let digit = text.get(place + n).copied().unwrap_or(0).wrapping_sub(b'0');
                if digit >= 10 {
                    return Err(Invalid("language file: Invalid decimal string"));
                }
                number = number * 10 + digit as u32;
            }
            let code = (number * 2 + 0x1001) as u16;
            text.splice(from..place + 3, code.to_be_bytes());
            place = from + 2;
        }
    }
    Ok(text)
}

/// A stable sort by length; a list of one or two is left as it is.
fn sort_by_length<T>(list: &mut [T], length: impl Fn(&T) -> usize) {
    if list.len() > 2 {
        list.sort_by_key(length);
    }
}

/// `arena_bytes` is the memory the compiler may use.
pub fn compile(text: &[u8], arena_bytes: u32) -> Result<Compiled, Invalid> {
    let source = Source(text);
    let mut arena = Arena(arena_bytes);
    arena.take((text.len() as u32 + 4) & !3)?;

    let mut words: Vec<Word> = Vec::new();
    let mut word_by_raw: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut word_buckets: Vec<Vec<usize>> = vec![Vec::new(); 256];
    let mut phrases: Vec<Phrase> = Vec::new();
    let mut phrase_by_words: HashMap<Vec<usize>, usize> = HashMap::new();
    let mut phrase_buckets: Vec<Vec<usize>> = vec![Vec::new(); 256];
    // For each message its phrase, or None for an empty one.
    let mut messages: Vec<Option<usize>> = Vec::new();
    let (mut lines, mut source_size, mut words_encountered, mut phrases_encountered) =
        (1u32, 0u32, 0u32, 0u32);

    // A message a line.
    let mut place = 0usize;
    loop {
        // Comments and line ends.
        let at_end = loop {
            match source.at(place) {
                0 => break true,
                b';' | 0x0D => {
                    while source.within(place)? != 0x0A {
                        place += 1;
                    }
                    place += 1;
                    lines += 1;
                }
                0x0A => {
                    place += 1;
                    lines += 1;
                }
                _ => break false,
            }
        };
        if at_end {
            break;
        }
        arena.take(0xC)?;

        let label = place;
        let mut counted = 0u32;
        loop {
            let byte = source.at(place);
            if byte == b';' {
                return Err(Invalid("language file: Message body not defined"));
            }
            if byte <= 0x20 {
                break;
            }
            counted += 1;
            place += 1;
        }
        if counted == 0 {
            return Err(Invalid("language file: Label not defined"));
        }
        loop {
            counted += 1;
            let byte = source.at(place);
            place += 1;
            if byte == b';' {
                return Err(Invalid("language file: Message body not defined"));
            }
            if byte != 9 && byte != 0x20 {
                break;
            }
        }
        arena.take(counted)?;
        let body = label + counted as usize - 1;

        // The message's words.
        let mut message: Vec<usize> = Vec::new();
        let mut scan = body;
        let end = loop {
            let token = source.next_word(scan)?;
            if token.length == 0 {
                if message.is_empty()
                    && (token.left == 0
                        || source.at(token.left - 1) != b'{'
                        || source.at(token.left) != b'}')
                {
                    return Err(Invalid("language file: Zero length phrase"));
                }
                break token.after;
            }
            if token.length > 0x7F {
                return Err(Invalid("language file: Word too long"));
            }
            // The word, kept once.
            let raw: &[u8] = match token.start {
                Some(start) => text
                    .get(start..start + token.length as usize)
                    .ok_or(Invalid("the language file ends in the middle of a word"))?,
                None => SPACE,
            };
            source_size += if token.start.is_some() {
                token.length
            } else {
                token.length >> 1
            };
            words_encountered += 1;
            let word = match word_by_raw.get(raw) {
                Some(&word) => word,
                None => {
                    arena.take(token.length + 0x19)?;
                    words.push(Word {
                        raw: raw.to_vec(),
                        text: Vec::new(),
                        id: 0,
                        entry: 0,
                    });
                    word_by_raw.insert(raw.to_vec(), words.len() - 1);
                    word_buckets[raw[0] as usize].push(words.len() - 1);
                    words.len() - 1
                }
            };
            if message.len() == 0x80 {
                return Err(Invalid("language file: Phrase too long"));
            }
            message.push(word);
            scan = token.after;
        };

        // The message as a phrase, kept once.
        if message.is_empty() {
            messages.push(None);
        } else {
            phrases_encountered += 1;
            let phrase = match phrase_by_words.get(&message) {
                Some(&phrase) => phrase,
                None => {
                    arena.take(0x18 + 4 * message.len() as u32)?;
                    phrases.push(Phrase {
                        words: message.clone(),
                        entry: 0,
                    });
                    phrase_buckets[words[message[0]].raw[0] as usize].push(phrases.len() - 1);
                    phrase_by_words.insert(message, phrases.len() - 1);
                    phrases.len() - 1
                }
            };
            messages.push(Some(phrase));
        }
        place = end;
    }

    // The words' escapes become bytes.
    for word in &mut words {
        word.text = unescaped(&word.raw)?;
    }

    // One list each, by first byte and then as met; sorted.
    let mut word_order: Vec<usize> = word_buckets.into_iter().flatten().collect();
    sort_by_length(&mut word_order, |&word| words[word].text.len());
    let mut phrase_order: Vec<usize> = phrase_buckets.into_iter().flatten().collect();
    sort_by_length(&mut phrase_order, |&phrase| phrases[phrase].words.len());
    if word_order.is_empty() || phrase_order.is_empty() {
        return Err(Invalid("a language file without words"));
    }

    // Offsets and numbers.
    let entries = messages.len() + words.len();
    if entries > 0x10000 {
        return Err(Invalid("language file: More than 64K indices in bundle"));
    }
    let index_size = entries as u32 * 4;
    arena.take(index_size)?;
    let mut offset = index_size + 1;
    for (n, &word) in word_order.iter().enumerate() {
        let length = words[word].text.len() as u32;
        words[word].entry = offset & 0x00FF_FFFF | length << 24;
        words[word].id = n as u32 + 1;
        offset += length;
    }
    let highest_word = word_order.len() as u32;
    if highest_word as u16 >= 0x8000 {
        return Err(Invalid("language file: Token range overflow"));
    }
    let padded = offset & 1;
    offset += padded;
    let mut highest_phrase = 0x8000u32;
    for &phrase in &phrase_order {
        let length = phrases[phrase].words.len() as u32;
        if length == 1 {
            phrases[phrase].entry = words[phrases[phrase].words[0]].entry;
        } else {
            phrases[phrase].entry =
                offset & 0x00FF_FFFF | ((length as u8).wrapping_neg() as u32) << 24;
            offset += 2 * length;
            highest_phrase += 1;
        }
    }
    if highest_phrase as u16 & 0x8000 == 0 {
        return Err(Invalid("language file: Token range overflow"));
    }
    let total = offset as usize;
    if total > TABLE_ROOM {
        return Err(Invalid("the compiled lexicon is larger than its table"));
    }

    // The index, and the table.
    let mut index: Vec<u32> = Vec::with_capacity(entries);
    let mut empty_messages = 0;
    for message in &messages {
        index.push(match message {
            Some(phrase) => phrases[*phrase].entry,
            None => {
                empty_messages += 1;
                index_size
            }
        });
    }
    index.extend(word_order.iter().map(|&word| words[word].entry));
    // An entry's number is that of the first index entry like it.
    let mut number_of: HashMap<u32, u16> = HashMap::new();
    for (n, &entry) in index.iter().enumerate() {
        number_of.entry(entry).or_insert(n as u16);
    }

    let mut table: Vec<u8> = index.iter().flat_map(|entry| entry.to_le_bytes()).collect();
    table.push(0);
    for &word in &word_order {
        table.extend(&words[word].text);
    }
    if padded != 0 {
        table.push(0);
    }
    for &phrase in &phrase_order {
        if phrases[phrase].words.len() > 1 {
            for &word in &phrases[phrase].words {
                table.extend(number_of[&words[word].entry].to_le_bytes());
            }
        }
    }
    if table.len() != total {
        return Err(Invalid(
            "the compiled lexicon is not the size its offsets say",
        ));
    }

    Ok(Compiled {
        table,
        padded,
        lines,
        messages: messages.len() as u32,
        source_size,
        words_encountered,
        words_stored: words.len() as u32,
        highest_word,
        phrases_encountered,
        phrases_stored: phrases.len() as u32,
        highest_phrase,
        index_size,
        empty_messages,
    })
}

/// A word of a string as the corrections meet it: the index entry and the place of the
/// phrase's number that names it (none for a string that is one word).
struct WordAt {
    entry: usize,
    named_at: Option<usize>,
}

/// Every word of string `id`, in order.
fn words_of(
    table: &[u8],
    id: u32,
    named_at: Option<usize>,
    depth: u32,
    out: &mut Vec<WordAt>,
) -> Result<(), Invalid> {
    if depth > 32 {
        return Err(Invalid("a string of the lexicon that contains itself"));
    }
    let entry = id as usize * 4;
    let head = read_u32(table, entry)?;
    let flag = (head >> 24) as usize;
    if flag < 0x80 {
        out.push(WordAt { entry, named_at });
        return Ok(());
    }
    let at = (head & 0x00FF_FFFF) as usize;
    for child in 0..0x100 - flag {
        let name = at + child * 2;
        let id = u16::from_le_bytes([byte(table, name)?, byte(table, name + 1)?]);
        words_of(table, id as u32, Some(name), depth + 1, out)?;
    }
    Ok(())
}

fn byte(table: &[u8], at: usize) -> Result<u8, Invalid> {
    table
        .get(at)
        .copied()
        .ok_or(Invalid("a lexicon entry past the table"))
}

fn read_u32(table: &[u8], at: usize) -> Result<u32, Invalid> {
    let b = table
        .get(at..at + 4)
        .ok_or(Invalid("a lexicon entry past the table"))?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// A word's bytes as the table holds them now.
fn word_text(table: &[u8], entry: usize) -> Result<&[u8], Invalid> {
    let head = read_u32(table, entry)?;
    let (at, len) = ((head & 0x00FF_FFFF) as usize, (head >> 24) as usize);
    table
        .get(at..at + len)
        .ok_or(Invalid("a lexicon word past the table"))
}

/// The four corrections the game makes in its lexicon after loading it: in string 0x6CA "Bowland"
/// loses its `w`, in 0x6AB "Laukennen" becomes "Laukkanen", and in 0x23B the word "A7" or "(A7)"
/// names string 0x23A's "A8" or "(A8)" instead. A correction meeting its word where the string is
/// that word alone is refused.
pub fn correct(table: &mut [u8]) -> Result<(), Invalid> {
    const ALONE: Invalid = Invalid("a correction's word is its whole string");
    let words = |table: &[u8], id| -> Result<Vec<WordAt>, Invalid> {
        let mut out = Vec::new();
        words_of(table, id, None, 0, &mut out)?;
        Ok(out)
    };
    let place = |table: &[u8], entry| -> Result<usize, Invalid> {
        Ok((read_u32(table, entry)? & 0x00FF_FFFF) as usize)
    };

    for w in words(table, 0x6CA)? {
        let text = word_text(table, w.entry)?;
        if text.len() == 7 && text.starts_with(b"Bowl") {
            let at = place(table, w.entry)?;
            table.copy_within(at + 3..at + 7, at + 2);
            table[w.entry + 3] = 6;
        }
    }
    for w in words(table, 0x6AB)? {
        let text = word_text(table, w.entry)?;
        if text.len() == 9 && text.starts_with(b"Lauk") {
            let at = place(table, w.entry)?;
            table[at + 4] = b'k';
            table[at + 5] = b'a';
        }
    }
    let mut a8: Option<usize> = None;
    for w in words(table, 0x23A)? {
        let text = word_text(table, w.entry)?;
        if text == b"A8" || text == b"(A8)" {
            a8 = Some(w.named_at.ok_or(ALONE)?);
        }
    }
    if let Some(a8) = a8 {
        for w in words(table, 0x23B)? {
            let text = word_text(table, w.entry)?;
            if text == b"A7" || text == b"(A7)" {
                let at = w.named_at.ok_or(ALONE)?;
                table.copy_within(a8..a8 + 2, at);
            }
        }
    }
    Ok(())
}

/// Whether the string table is made again from the language file: the setting (signed) 1 or more
/// and the key's state not 0, a developer's way to see a changed language file.
pub fn recompile_wanted(level: u32, key: u8) -> bool {
    level as i32 >= 1 && key != 0
}
