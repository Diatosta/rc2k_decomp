//! A set of named files, and the plain bytes it crosses the page's boundary as: a count, then
//! for each file the length of its path, the path, the length of its data and the data (all
//! lengths four bytes, little-endian).

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Files(Vec<(String, Vec<u8>)>);

impl Files {
    pub fn get(&self, path: &str) -> Option<&[u8]> {
        self.0.iter().find(|(p, _)| p == path).map(|(_, d)| &d[..])
    }

    /// Sets the file, replacing one of the name.
    pub fn set(&mut self, path: &str, data: Vec<u8>) {
        match self.0.iter_mut().find(|(p, _)| p == path) {
            Some(file) => file.1 = data,
            None => self.0.push((path.to_owned(), data)),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.0.iter().map(|(p, d)| (p.as_str(), &d[..]))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = (self.0.len() as u32).to_le_bytes().to_vec();
        for (path, data) in &self.0 {
            out.extend_from_slice(&(path.len() as u32).to_le_bytes());
            out.extend_from_slice(path.as_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(data);
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        fn take<'a>(b: &mut &'a [u8], n: usize) -> Option<&'a [u8]> {
            let (head, rest) = b.split_at_checked(n)?;
            *b = rest;
            Some(head)
        }
        fn word(b: &mut &[u8]) -> Option<usize> {
            Some(u32::from_le_bytes(take(b, 4)?.try_into().ok()?) as usize)
        }
        let mut rest = bytes;
        let count = word(&mut rest)?;
        let mut files = Self::default();
        for _ in 0..count {
            let n = word(&mut rest)?;
            let path = String::from_utf8(take(&mut rest, n)?.to_vec()).ok()?;
            let n = word(&mut rest)?;
            files.set(&path, take(&mut rest, n)?.to_vec());
        }
        Some(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_cross_as_bytes_and_back() {
        let mut f = Files::default();
        f.set("a/b", vec![1, 2, 3]);
        f.set("c", vec![]);
        f.set("a/b", vec![9]);
        assert_eq!(Files::from_bytes(&f.to_bytes()), Some(f));
        assert_eq!(Files::from_bytes(&[1, 0, 0, 0, 5]), None);
    }
}
