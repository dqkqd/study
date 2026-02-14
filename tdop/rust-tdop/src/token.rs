use anyhow::Result;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Literal(u32),
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    LParen,
    RParen,
    Eof,
}

#[derive(Debug)]
pub struct Tokenizer {
    tokens: Vec<Token>,
}

impl Tokenizer {
    pub fn new(program: &str) -> Tokenizer {
        let re = Regex::new(r"\d+|[+\-*/^()]").unwrap();

        let mut tokens: Vec<Token> = re
            .find_iter(program)
            .map(|m| match m.as_str() {
                "+" => Token::Add,
                "-" => Token::Sub,
                "*" => Token::Mul,
                "/" => Token::Div,
                "^" => Token::Pow,
                "(" => Token::LParen,
                ")" => Token::RParen,
                s => match s.parse::<u32>() {
                    Ok(value) => Token::Literal(value),
                    Err(_) => todo!("Unreacognize token: {s}"),
                },
            })
            .collect();

        tokens.push(Token::Eof);

        Tokenizer {
            tokens: tokens.into_iter().rev().collect(),
        }
    }

    pub fn next(&mut self) -> Result<Token> {
        match self.tokens.pop() {
            Some(token) => Ok(token),
            None => anyhow::bail!("Expected to see a token"),
        }
    }

    pub fn peek(&mut self) -> Option<Token> {
        self.tokens.last().cloned()
    }

    pub fn next_expect(&mut self, expected: Token) -> Result<()> {
        match self.peek() {
            Some(token) => {
                if token != expected {
                    anyhow::bail!("Expected {:?}, got {:?}", expected, token)
                }
                self.next()?;
                Ok(())
            }
            None => anyhow::bail!("Exhausted, expected {:?}", expected),
        }
    }
}
