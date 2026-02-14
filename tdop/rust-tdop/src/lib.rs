use anyhow::Result;

use crate::{expr::Expr, token::Tokenizer};

mod expr;
mod token;

pub fn parse(program: &str) -> Result<Expr> {
    let mut tokenizer = Tokenizer::new(program);
    let e = Expr::parse(&mut tokenizer, 0)?;
    Ok(e)
}

#[cfg(test)]
mod test {
    use crate::parse;
    use rstest::rstest;

    #[rstest]
    #[case("1", "1", 1.0)]
    #[case("+1", "1", 1.0)]
    #[case("-1", "(- 1)", -1.0)]
    #[case("1 + 2", "(+ 1 2)", 3.0)]
    #[case("1 - 2", "(- 1 2)", -1.0)]
    #[case("2 * 3", "(* 2 3)", 6.0)]
    #[case("3 / 4", "(/ 3 4)", 0.75)]
    #[case("1 + 3 + 2", "(+ (+ 1 3) 2)", 6.0)]
    #[case("1 + 3 * 2", "(+ 1 (* 3 2))", 7.0)]
    #[case("1 + 3 - 2", "(- (+ 1 3) 2)", 2.0)]
    #[case("1 + 3 / 2", "(+ 1 (/ 3 2))", 2.5)]
    #[case("1 + 2 * 3 * 4 + 5", "(+ (+ 1 (* (* 2 3) 4)) 5)", 30.0)]
    #[case("1 + 2 * 3 * 4 * 5", "(+ 1 (* (* (* 2 3) 4) 5))", 121.0)]
    #[case("(1)", "1", 1.0)]
    #[case("(-1)","(- 1)",-1.0)]
    #[case("(+1)", "1", 1.0)]
    #[case("-(+1)","(- 1)",-1.0)]
    #[case("-(2 + 3)","(- (+ 2 3))",-5.0)]
    #[case("2 * (3 + 4)", "(* 2 (+ 3 4))", 14.0)]
    #[case("(3 + 4) * 2", "(* (+ 3 4) 2)", 14.0)]
    #[case("2^3", "(^ 2 3)", 8.0)]
    #[case("2^(3 + 1)", "(^ 2 (+ 3 1))", 16.0)]
    #[case("(2 + 1)^(3 + 1)", "(^ (+ 2 1) (+ 3 1))", 81.0)]
    fn test(
        #[case] program: &str,
        #[case] expected_parsed_expr: &str,
        #[case] expected_eval_value: f64,
    ) {
        let expr = parse(program).unwrap();
        assert_eq!(expr.as_str(), expected_parsed_expr);
        assert_eq!(expr.eval(), expected_eval_value);
    }
}
