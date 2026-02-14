use rust_tdop::parse;

fn main() -> anyhow::Result<()> {
    let program = "1 + 2 + 3";
    let expr = parse(program)?;
    println!("parsed: {}", expr.as_str());
    println!("eval: {} = {}", program, expr.eval());

    Ok(())
}
