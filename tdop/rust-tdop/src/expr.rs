use crate::token::{Token, Tokenizer};

use anyhow::Result;

#[derive(Debug)]
pub enum Expr {
    Prefix(PrefixExpr),
    Infix(InfixExpr),
}

#[derive(Debug)]
pub enum PrefixExpr {
    Literal(u32),
    Add { inner: Box<Expr> },
    Sub { inner: Box<Expr> },
    Paren { inner: Box<Expr> },
}

#[derive(Debug)]
pub enum InfixExpr {
    Add {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Sub {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Mul {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Div {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Pow {
        base: Box<Expr>,
        exponent: Box<Expr>,
    },
}

impl Expr {
    pub(crate) fn parse(tokenizer: &mut Tokenizer, binding_power: u8) -> Result<Expr> {
        let token = tokenizer.next()?;
        let mut lhs = Expr::Prefix(PrefixExpr::parse(tokenizer, token)?);

        loop {
            let current_token = match tokenizer.peek() {
                Some(Token::Eof) | None => break,
                Some(token) => token,
            };

            if binding_power >= InfixExpr::binding_power(current_token) {
                break;
            }

            tokenizer.next()?;
            lhs = Expr::Infix(InfixExpr::parse(tokenizer, lhs, current_token)?);
        }

        // prefix parse
        Ok(lhs)
    }

    pub fn as_str(&self) -> String {
        match self {
            Expr::Prefix(prefix_expr) => prefix_expr.as_str(),
            Expr::Infix(infix_expr) => infix_expr.as_str(),
        }
    }

    pub fn eval(&self) -> f64 {
        match self {
            Expr::Prefix(prefix_expr) => prefix_expr.eval(),
            Expr::Infix(infix_expr) => infix_expr.eval(),
        }
    }
}

impl PrefixExpr {
    fn binding_power(token: Token) -> u8 {
        match token {
            Token::Literal(_) => 10,
            Token::Add | Token::Sub => 100,
            Token::LParen => 0,
            token => unimplemented!("No binding power for {:?}", token),
        }
    }

    pub fn parse(tokenizer: &mut Tokenizer, token: Token) -> Result<PrefixExpr> {
        let binding_power = Self::binding_power(token);
        let res = match token {
            Token::Literal(value) => PrefixExpr::Literal(value),
            Token::Add => {
                let inner = Expr::parse(tokenizer, binding_power)?;
                PrefixExpr::Add {
                    inner: Box::new(inner),
                }
            }
            Token::Sub => {
                let inner = Expr::parse(tokenizer, binding_power)?;
                PrefixExpr::Sub {
                    inner: Box::new(inner),
                }
            }
            Token::LParen => {
                let inner = Expr::parse(tokenizer, binding_power)?;
                tokenizer.next_expect(Token::RParen)?;
                PrefixExpr::Paren {
                    inner: Box::new(inner),
                }
            }
            token => unimplemented!("unsupported prefix for {:?}", token),
        };

        Ok(res)
    }

    fn as_str(&self) -> String {
        match self {
            PrefixExpr::Literal(value) => format!("{}", value),
            PrefixExpr::Add { inner } => inner.as_str(),
            PrefixExpr::Sub { inner } => format!("(- {})", inner.as_str()),
            PrefixExpr::Paren { inner } => inner.as_str(),
        }
    }

    pub(crate) fn eval(&self) -> f64 {
        match self {
            PrefixExpr::Literal(value) => *value as f64,
            PrefixExpr::Add { inner } => inner.eval(),
            PrefixExpr::Sub { inner } => -inner.eval(),
            PrefixExpr::Paren { inner } => inner.eval(),
        }
    }
}

impl InfixExpr {
    fn binding_power(token: Token) -> u8 {
        match token {
            Token::RParen => 0,
            Token::Literal(_) => 10,
            Token::Add | Token::Sub => 20,
            Token::Mul | Token::Div => 30,
            Token::Pow => 40,
            token => unimplemented!("No binding power for {:?}", token),
        }
    }
    pub fn parse(tokenizer: &mut Tokenizer, lhs: Expr, token: Token) -> Result<InfixExpr> {
        let rhs = Expr::parse(tokenizer, Self::binding_power(token))?;
        let res = match token {
            Token::Literal(_) => todo!(),
            Token::Add => InfixExpr::Add {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            Token::Sub => InfixExpr::Sub {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            Token::Mul => InfixExpr::Mul {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            Token::Div => InfixExpr::Div {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            Token::Pow => InfixExpr::Pow {
                base: Box::new(lhs),
                exponent: Box::new(rhs),
            },
            token => unimplemented!("unsupported infix for {:?}", token),
        };

        Ok(res)
    }

    fn as_str(&self) -> String {
        match self {
            InfixExpr::Add { lhs, rhs } => {
                format!("(+ {} {})", lhs.as_str(), rhs.as_str())
            }
            InfixExpr::Sub { lhs, rhs } => {
                format!("(- {} {})", lhs.as_str(), rhs.as_str())
            }
            InfixExpr::Mul { lhs, rhs } => {
                format!("(* {} {})", lhs.as_str(), rhs.as_str())
            }
            InfixExpr::Div { lhs, rhs } => {
                format!("(/ {} {})", lhs.as_str(), rhs.as_str())
            }
            InfixExpr::Pow { base, exponent } => {
                format!("(^ {} {})", base.as_str(), exponent.as_str())
            }
        }
    }

    pub(crate) fn eval(&self) -> f64 {
        match self {
            InfixExpr::Add { lhs, rhs } => lhs.eval() + rhs.eval(),
            InfixExpr::Sub { lhs, rhs } => lhs.eval() - rhs.eval(),
            InfixExpr::Mul { lhs, rhs } => lhs.eval() * rhs.eval(),
            InfixExpr::Div { lhs, rhs } => lhs.eval() / rhs.eval(),
            InfixExpr::Pow { base, exponent } => {
                let base = base.eval();
                let exponent = exponent.eval();
                base.powf(exponent)
            }
        }
    }
}
