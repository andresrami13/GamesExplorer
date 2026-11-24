//! Parser del formato KeyValues de Valve en texto (`.vdf`, `.acf`).
//!
//! Soporta cadenas entre comillas con escapes, tokens sin comillas, comentarios `//`
//! y condicionales `[$WIN32]`, que se ignoran. Las claves se comparan sin distinguir
//! mayúsculas, porque Steam no es consistente (`"Software"` frente a `"software"`).

use crate::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Vdf {
    Str(String),
    Obj(Vec<(String, Vdf)>),
}

impl Vdf {
    /// Primer hijo cuya clave coincide sin distinguir mayúsculas.
    pub fn get(&self, key: &str) -> Option<&Vdf> {
        self.entries()
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v)
    }

    /// Recorre una ruta de claves: `root.path(&["a", "b"])`.
    pub fn path(&self, keys: &[&str]) -> Option<&Vdf> {
        keys.iter().try_fold(self, |node, key| node.get(key))
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Vdf::Str(s) => Some(s),
            Vdf::Obj(_) => None,
        }
    }

    pub fn num<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.str(key)?.trim().parse().ok()
    }

    pub fn entries(&self) -> &[(String, Vdf)] {
        match self {
            Vdf::Obj(entries) => entries,
            Vdf::Str(_) => &[],
        }
    }
}

/// Parsea un documento completo. Devuelve un objeto raíz con las claves de primer nivel.
pub fn parse(input: &str) -> Result<Vdf, Error> {
    let mut lexer = Lexer { chars: input.char_indices().peekable(), src: input };
    let root = parse_object(&mut lexer, false)?;
    Ok(Vdf::Obj(root))
}

fn parse_object(lexer: &mut Lexer, nested: bool) -> Result<Vec<(String, Vdf)>, Error> {
    let mut entries = Vec::new();
    loop {
        let key = match lexer.next_token()? {
            None if nested => return Err(Error::Parse("falta '}' de cierre".into())),
            None => return Ok(entries),
            Some(Token::Close) if nested => return Ok(entries),
            Some(Token::Close) => return Err(Error::Parse("'}' inesperado".into())),
            Some(Token::Open) => return Err(Error::Parse("'{' sin clave".into())),
            Some(Token::Text(key)) => key,
        };
        let value = match lexer.next_token()? {
            Some(Token::Text(value)) => Vdf::Str(value),
            Some(Token::Open) => Vdf::Obj(parse_object(lexer, true)?),
            _ => return Err(Error::Parse(format!("la clave '{key}' no tiene valor"))),
        };
        entries.push((key, value));
    }
}

enum Token {
    Text(String),
    Open,
    Close,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    src: &'a str,
}

impl Lexer<'_> {
    fn next_token(&mut self) -> Result<Option<Token>, Error> {
        loop {
            let Some(&(start, c)) = self.chars.peek() else { return Ok(None) };
            match c {
                c if c.is_whitespace() => {
                    self.chars.next();
                }
                '/' if self.src[start..].starts_with("//") => self.skip_line(),
                '[' => self.skip_conditional(),
                '{' => {
                    self.chars.next();
                    return Ok(Some(Token::Open));
                }
                '}' => {
                    self.chars.next();
                    return Ok(Some(Token::Close));
                }
                '"' => {
                    self.chars.next();
                    return self.quoted().map(|s| Some(Token::Text(s)));
                }
                _ => return Ok(Some(Token::Text(self.bare()))),
            }
        }
    }

    fn skip_line(&mut self) {
        for (_, c) in self.chars.by_ref() {
            if c == '\n' {
                break;
            }
        }
    }

    fn skip_conditional(&mut self) {
        for (_, c) in self.chars.by_ref() {
            if c == ']' {
                break;
            }
        }
    }

    fn quoted(&mut self) -> Result<String, Error> {
        let mut out = String::new();
        while let Some((_, c)) = self.chars.next() {
            match c {
                '"' => return Ok(out),
                '\\' => match self.chars.next().map(|(_, c)| c) {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(other) => out.push(other),
                    None => break,
                },
                _ => out.push(c),
            }
        }
        Err(Error::Parse("cadena sin cerrar".into()))
    }

    fn bare(&mut self) -> String {
        let mut out = String::new();
        while let Some(&(_, c)) = self.chars.peek() {
            if c.is_whitespace() || matches!(c, '"' | '{' | '}') {
                break;
            }
            out.push(c);
            self.chars.next();
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_objects_escapes_comments_and_conditionals() {
        let doc = parse(
            r#"
            // comentario
            "Root"
            {
                "path"   "C:\\Program Files (x86)\\Steam"
                "quote"  "dice \"hola\""
                "flag"   "1" [$WIN32]
                bare     value
                "Child" { "N" "42" }
            }
            "#,
        )
        .unwrap();
        let root = doc.get("root").unwrap();
        assert_eq!(root.str("PATH"), Some(r"C:\Program Files (x86)\Steam"));
        assert_eq!(root.str("quote"), Some(r#"dice "hola""#));
        assert_eq!(root.str("flag"), Some("1"));
        assert_eq!(root.str("bare"), Some("value"));
        assert_eq!(root.path(&["child"]).unwrap().num::<u32>("n"), Some(42));
    }

    #[test]
    fn rejects_unclosed_object() {
        assert!(parse(r#""a" { "b" "c" "#).is_err());
    }
}
