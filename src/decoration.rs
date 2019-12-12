#[derive(Debug, Clone, Copy)]
pub enum Decoration {
    Larger,
    Normal,
    Smaller,
}

impl Decoration {
    pub fn scale_factor(self) -> f32 {
        match self {
            Decoration::Larger => 1.3,
            Decoration::Normal => 1.0,
            Decoration::Smaller => 0.6,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DecoratedString {
    pub decoration: Decoration,
    pub body: String,
}

impl From<&str> for DecoratedString {
    fn from(s: &str) -> DecoratedString {
        let (decoration, s) = if s.chars().nth(0).map(|c| c == '*').unwrap_or(false)
            && s.chars().last().map(|c| c == '*').unwrap_or(false)
        {
            let mut s = s.chars().skip(1).collect::<String>();
            s.pop();
            (Decoration::Larger, s)
        } else if s.chars().nth(0).map(|c| c == '_').unwrap_or(false)
            && s.chars().last().map(|c| c == '_').unwrap_or(false)
        {
            let mut s = s.chars().skip(1).collect::<String>();
            s.pop();
            (Decoration::Smaller, s)
        } else {
            (Decoration::Normal, s.to_owned())
        };
        DecoratedString::new(decoration, &s)
    }
}

impl DecoratedString {
    pub fn new(decoration: Decoration, body: &str) -> DecoratedString {
        DecoratedString {
            decoration,
            body: body.to_owned(),
        }
    }
}
