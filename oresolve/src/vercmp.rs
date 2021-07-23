use std::cmp::Ordering;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Style {
    Native,
    Maven,
}

impl Style {
    pub fn matches(&self, range: &str, version: &str) -> bool {
        match self {
            Style::Native => {
                unimplemented!()
            }
            Style::Maven => {
                // maven version "ranges" are actually always specific versions
                //self.vercmp(range, version) == Some(Ordering::Equal)
                range == version
            }
        }
    }

    pub fn vercmp(&self, v1: &str, v2: &str) -> Option<Ordering> {
        match self {
            Style::Native => {
                unimplemented!()
            }
            Style::Maven => {
                let mut i1 = components(v1);
                let mut i2 = components(v2);
                unimplemented!()
            }
        }
    }
}

fn components<'a>(version: &'a str) -> Components<'a> {
    Components {
        s: version,
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Component<'a> {
    Number(usize),
    Decimal,
    Other(&'a str),
}

struct Components<'a> {
    s: &'a str,
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.s.chars().next() {
            None => None,
            Some(c) => {
                let t = CharType::get(c);
                if t == CharType::Decimal {
                    self.s = &self.s[1..];
                    Some(Component::Decimal)
                } else {
                    let s = self.s.find(|ch| CharType::get(ch) != t).unwrap_or(self.s.len());
                    let r = &self.s[..s];
                    self.s = &self.s[s..];
                    match t {
                        CharType::Digit => Some(Component::Number(r.parse().unwrap())),
                        CharType::Decimal => unreachable!(),
                        CharType::Other => Some(Component::Other(r)),
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CharType {
    Digit,
    Decimal,
    Other,
}

impl CharType {
    fn get(c: char) -> CharType {
        if "0123456789".contains(c) {
            CharType::Digit
        } else if c == '.' {
            CharType::Decimal
        } else {
            CharType::Other
        }
    }
}