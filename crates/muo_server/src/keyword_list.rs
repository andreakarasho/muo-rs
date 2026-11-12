//! The speech keyword ids a client sends with encoded speech.

/// A growing list of speech keywords that empties when taken.
#[derive(Default, Debug)]
pub struct KeywordList {
    keywords: Vec<i32>,
}

impl KeywordList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn count(&self) -> usize {
        self.keywords.len()
    }

    pub fn contains(&self, keyword: i32) -> bool {
        self.keywords.contains(&keyword)
    }

    pub fn add(&mut self, keyword: i32) {
        self.keywords.push(keyword);
    }

    /// `ToArray`: the keywords, and the list is empty afterwards (`Count` is reset).
    pub fn to_array(&mut self) -> Vec<i32> {
        std::mem::take(&mut self.keywords)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_array_empties_the_list() {
        let mut l = KeywordList::new();
        for i in 0..20 {
            l.add(i);
        }
        assert!(l.contains(19) && !l.contains(20));
        assert_eq!(l.to_array().len(), 20);
        assert_eq!(l.count(), 0);
        assert!(l.to_array().is_empty());
    }
}
