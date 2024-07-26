use thiserror::Error;
use std::iter::Peekable;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unexpected end of input")]
    EOF,
    #[error("Unknown character class `\\{0}`")]
    UnknownCharacterType(char),
}

pub type Result<T> = std::result::Result<T, Error>;

enum SingleCharacterMatcher {
    Literal(char),
    Digit,
    Alphanumeric,
    Group(Vec<SingleCharacterMatcher>),
}

impl SingleCharacterMatcher {
    pub fn new(input: &mut Peekable<impl Iterator<Item = char>>) -> Result<Self> {
        match input.next() {
            Some('\\') => Self::new_class(input.next().ok_or(Error::EOF)?),
            Some('[') => Self::new_group(input),
            Some(ch) => Ok(Self::new_literal(ch)),
            None => Err(Error::EOF),
        }
    }

    fn new_in_group(input: &mut impl Iterator<Item = char>) -> Result<Self> {
        match input.next() {
            Some('\\') => Self::new_class(input.next().ok_or(Error::EOF)?),
            Some(ch) => Ok(Self::new_literal(ch)),
            None => Err(Error::EOF),
        }
    }

    pub fn new_literal(ch: char) -> Self {
        SingleCharacterMatcher::Literal(ch)
    }

    pub fn new_class(class: char) -> Result<Self> {
        match class {
            '\\' => Ok(Self::Literal('\\')),
            ']' => Ok(Self::Literal(']')),
            'd' => Ok(Self::Digit),
            'w' => Ok(Self::Alphanumeric),
            ch => Err(Error::UnknownCharacterType(ch)),
        }
    }

    pub fn new_group(input: &mut Peekable<impl Iterator<Item = char>>) -> Result<Self> {
        let mut options = Vec::new();
        while let Some(ch) = input.peek() {
            if *ch == ']' {
                input.next(); // Consume the ']' character
                return Ok(Self::Group(options));
            } else {
                options.push(Self::new_in_group(input)?);
            }
        }

        Err(Error::EOF)
    }

    pub fn test(&self, ch: char) -> bool {
        match self {
            SingleCharacterMatcher::Literal(c) => *c == ch,
            SingleCharacterMatcher::Digit => ch.is_ascii_digit(),
            SingleCharacterMatcher::Alphanumeric => ch.is_ascii_alphanumeric() || ch == '_',
            SingleCharacterMatcher::Group(options) => options.iter().any(|o| o.test(ch)),
        }
    }
}

enum Matcher {
    SingleCharacter(SingleCharacterMatcher),
}

impl Matcher {
    pub fn new(input: &mut Peekable<impl Iterator<Item = char>>) -> Result<Self> {
        match input.peek() {
            Some(_) => Ok(Self::SingleCharacter(SingleCharacterMatcher::new(input)?)),
            None => Err(Error::EOF),
        }
    }

    pub fn test(&self, input: &mut impl Iterator<Item = char>) -> bool {
        match self {
            Matcher::SingleCharacter(c) => input.next().is_some_and(|ch| c.test(ch)),
        }
    }
}
pub struct Pattern {
    matchers: Vec<Matcher>,
}

impl Pattern {
    pub fn new(input: &str) -> Result<Self> {
        let mut input = input.chars().peekable();
        let mut matchers = Vec::new();
        while let Some(_) = input.peek() {
            matchers.push(Matcher::new(&mut input)?);
        }

        Ok(Self { matchers })
    }
    pub fn test(&self, input: &str) -> bool {
        let mut iter = input.chars().peekable();
        while let Some(_) = iter.peek() {
            if self.test_section(iter.clone()) {
                return true;
            }
            iter.next();
        }
        false
    }
    fn test_section(&self, mut input: impl Iterator<Item = char>) -> bool {
        for matcher in &self.matchers {
            if !matcher.test(&mut input) {
                return false;
            }
        }
        true
    }
}
#[cfg(test)]
mod test {
    use crate::Pattern;
    #[test]
    fn single_character_match() {
        let pattern = Pattern::new("x").expect("Pattern is correct");
        assert!(!pattern.test(""));
        assert!(!pattern.test("X"));
        assert!(!pattern.test("abc"));
        assert!(pattern.test("x"));
        assert!(pattern.test("xylophone"));
        assert!(pattern.test("lax"));
        assert!(pattern.test("taxi"));
    }
    #[test]
    fn literal_match() {
        let pattern = Pattern::new("abc").expect("Pattern is correct");
        assert!(pattern.test("abc"));
        assert!(pattern.test("abcde"));
        assert!(pattern.test("012abcde"));
        assert!(!pattern.test("lax"));
        assert!(!pattern.test("abxc"));
    }

    #[test]
    fn digit_match() {
        let pattern = Pattern::new(r"\d").expect("Pattern is correct");
        assert!(pattern.test("1"));
        assert!(pattern.test("a2"));
        assert!(pattern.test("012abcde"));
        assert!(!pattern.test("lax"));
        assert!(!pattern.test("abxc"));
    }

    #[test]
    fn alphanumeric_match() {
        let pattern = Pattern::new(r"\w").expect("Pattern is correct");
        assert!(pattern.test("1"));
        assert!(pattern.test("a"));
        assert!(pattern.test("Z"));
        assert!(pattern.test("_"));
        assert!(!pattern.test("-"));
        assert!(!pattern.test(":"));
    }

    #[test]
    fn group_match() {
        let pattern = Pattern::new(r"[a\d]").expect("Pattern is correct");
        assert!(pattern.test("1"));
        assert!(pattern.test("a"));
        assert!(pattern.test("9"));
        assert!(pattern.test("za"));
        assert!(!pattern.test("b"));
        assert!(!pattern.test(":"));
    }

    #[test]
    fn full_test() {
        let pattern = Pattern::new(r"a\d[\w:]").expect("Pattern is correct");
        assert!(pattern.test("a9c"));
        assert!(pattern.test("da4cg"));
        assert!(!pattern.test("ab9c"));
        assert!(!pattern.test("ab9X"));
        assert!(!pattern.test("ab9_"));
        assert!(!pattern.test("ab9:"));
    }
}
