//! The lexicon compiler's work, apart from the game: from the text of a language file to
//! the packed string table and the counts the game logs. `strings::compile_lexicon` is the
//! hook; what the original's 25 functions do is here as plain functions, in their order.
//!
//! The language file: one message a line, a label and then the text; `;` starts a comment.
//! The text is cut into words: a run of letters, digits and bytes of 0x80 and more (with
//! `'` and `-` inside it); any other character by itself; `{...}` as one word whatever is
//! in it; `\x` (and `\#hh`) as one word; `~ddd` as one word; and the blanks between two
//! words as one word, a space. Every different word is kept once, and every different
//! message, as the list of its words, once.
//! The table: an index of dwords, one for each message and then one for each word, each
//! an offset into the table with a flag in its top byte (below 0x80: so many bytes of
//! text; else 0x100 less so many 16-bit numbers of index entries). After the index a zero
//! byte, the words' bytes shortest first, a padding byte if that leaves an odd offset, and
//! the messages of more than one word, shortest first, as the index numbers of their words.
//! A message of one word shares that word's entry.
//!
//! Whatever the original would report as an error in the file, or would run into, is
//! refused here, and the original left to say so.

use crate::errors::BadInput;
use std::collections::HashMap;

/// What `compile` makes.
#[derive(Debug, PartialEq, Eq)]
pub struct Compiled {
    pub table: Vec<u8>,
    /// The counts, as the original keeps them (`dword_94EAB8` .. `dword_94F6F4`).
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

/// The table has room from 0x4E07F8 to 0x4F07C8.
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

/// The memory the original works in: when it runs out, the original ends in a fatal error.
struct Arena(u32);

impl Arena {
    fn take(&mut self, bytes: u32) -> Result<(), BadInput> {
        if bytes > self.0 {
            return Err(BadInput("the lexicon's memory ran out"));
        }
        self.0 -= bytes;
        Ok(())
    }
}

/// One word of a message's text, as the original's tokeniser hands it out.
struct Token {
    /// 0 = the text has ended (or an empty `{}` was met).
    length: u32,
    /// Where the word's bytes are; None for the space between two words.
    start: Option<usize>,
    /// What the original leaves in esi when the length is 0.
    left: usize,
    after: usize,
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte >= 0x80
}

struct Source<'a>(&'a [u8]);

impl Source<'_> {
    /// The byte at a place; past the text the original finds the zeros of fresh memory.
    fn at(&self, place: usize) -> u8 {
        self.0.get(place).copied().unwrap_or(0)
    }

    /// For the loops that do not stop at a zero byte: past the text they would run on.
    fn within(&self, place: usize) -> Result<u8, BadInput> {
        if place > self.0.len() {
            return Err(BadInput(
                "the language file ends in the middle of a comment or a group",
            ));
        }
        Ok(self.at(place))
    }

    /// `sub_403889`: the next word from `scan` on.
    fn next_word(&self, scan: usize) -> Result<Token, BadInput> {
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
        let escaped = |place: &mut usize, length: &mut u32| -> Result<(), BadInput> {
            let kind = self.at(*place);
            *place += 1;
            if kind <= 0x20 {
                return Err(BadInput("language file: Invalid special symbol type"));
            }
            if kind == b'#' {
                if self.at(*place) <= 0x20 || self.at(*place + 1) <= 0x20 {
                    return Err(BadInput("language file: Invalid special symbol type"));
                }
                *length += 2;
                *place += 2;
            }
            Ok(())
        };
        match first {
            b'{' => {
                let content = place;
                // Counted like the original: one for every byte read, the closing brace
                // too, which is then not part of the word.
                let mut read = 0u32;
                loop {
                    read += 1;
                    let byte = self.within(place)?;
                    place += 1;
                    match byte {
                        b'{' | b';' | 0x0A | 0x0D => {
                            return Err(BadInput("language file: Missing start or end of group"));
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
            b'}' => Err(BadInput("language file: Missing start or end of group")),
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
                        return Err(BadInput("language file: Invalid DBCS type"));
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

/// `sub_403B85` after its `#`: two hex digits.
fn hex_pair(digits: &[u8]) -> Result<u8, BadInput> {
    let mut value = 0u8;
    for n in 0..2 {
        let digit = match digits.get(n).copied().unwrap_or(0) {
            byte @ b'0'..=b'9' => byte - b'0',
            byte @ b'A'..=b'F' => byte - b'A' + 10,
            byte @ b'a'..=b'f' => byte - b'a' + 10,
            _ => return Err(BadInput("language file: Invalid hex string")),
        };
        value = value << 4 | digit;
    }
    Ok(value)
}

/// `sub_403AC2` for one word: the escapes of the file become the bytes of the table.
fn unescaped(raw: &[u8]) -> Result<Vec<u8>, BadInput> {
    let mut text = raw.to_vec();
    let mut place = 0;
    while place < text.len() {
        let byte = text[place];
        place += 1;
        // The original takes a zero byte for the word's end.
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
                    return Err(BadInput("language file: Invalid decimal string"));
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

/// The original's bubble sort: it leaves a list of one or two as it is, and otherwise
/// sorts by length, things of one length staying in their order.
fn sort_by_length<T>(list: &mut [T], length: impl Fn(&T) -> usize) {
    if list.len() > 2 {
        list.sort_by_key(length);
    }
}

/// `arena_bytes` is the memory the original gives itself (`dword_519554` * 0x400).
pub fn compile(text: &[u8], arena_bytes: u32) -> Result<Compiled, BadInput> {
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

    // `sub_403682`: a message a line.
    let mut place = 0usize;
    loop {
        // `sub_403CD2`: comments and line ends.
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
                return Err(BadInput("language file: Message body not defined"));
            }
            if byte <= 0x20 {
                break;
            }
            counted += 1;
            place += 1;
        }
        if counted == 0 {
            return Err(BadInput("language file: Label not defined"));
        }
        loop {
            counted += 1;
            let byte = source.at(place);
            place += 1;
            if byte == b';' {
                return Err(BadInput("language file: Message body not defined"));
            }
            if byte != 9 && byte != 0x20 {
                break;
            }
        }
        arena.take(counted)?;
        let body = label + counted as usize - 1;

        // `sub_403745`: the message's words.
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
                    return Err(BadInput("language file: Zero length phrase"));
                }
                break token.after;
            }
            if token.length > 0x7F {
                return Err(BadInput("language file: Word too long"));
            }
            // `sub_403A10`: the word, kept once.
            let raw: &[u8] = match token.start {
                Some(start) => text
                    .get(start..start + token.length as usize)
                    .ok_or(BadInput("the language file ends in the middle of a word"))?,
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
                return Err(BadInput("language file: Phrase too long"));
            }
            message.push(word);
            scan = token.after;
        };

        // `sub_40379F`: the message as a phrase, kept once.
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

    // `sub_403AC2`.
    for word in &mut words {
        word.text = unescaped(&word.raw)?;
    }

    // `sub_403BDC`, `sub_403C54`: one list each, by first byte and then as met; sorted.
    let mut word_order: Vec<usize> = word_buckets.into_iter().flatten().collect();
    sort_by_length(&mut word_order, |&word| words[word].text.len());
    let mut phrase_order: Vec<usize> = phrase_buckets.into_iter().flatten().collect();
    sort_by_length(&mut phrase_order, |&phrase| phrases[phrase].words.len());
    if word_order.is_empty() || phrase_order.is_empty() {
        return Err(BadInput("a language file without words"));
    }

    // `sub_403DA7`: offsets and numbers.
    let entries = messages.len() + words.len();
    if entries > 0x10000 {
        return Err(BadInput("language file: More than 64K indices in bundle"));
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
        return Err(BadInput("language file: Token range overflow"));
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
        return Err(BadInput("language file: Token range overflow"));
    }
    let total = offset as usize;
    if total > TABLE_ROOM {
        return Err(BadInput("the compiled lexicon is larger than its table"));
    }

    // The index, and `sub_403F0B`: the table.
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
    // `sub_403FC1`: an entry's number is that of the first index entry like it.
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
        return Err(BadInput(
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
