use crate::calculator::Calculator;

#[test]
fn test_calc() {
    let a = 8;
    let b = 94;
    println!("{}", a as f32 / b as f32);
}

#[test]
fn calculator() -> Result<(), Box<dyn std::error::Error>> {
    let mut calc = Calculator::new();
    let result = calc
        .symbol_in('5')?
        .symbol_in('/')?
        .symbol_in('1')?
        .symbol_in('0')?
        .symbol_in('0')?
        .symbol_in('0')?

        // .erase()
        .erase()

        .symbol_in('+')?

        .symbol_in('1')?
        .symbol_in('0')?

        .symbol_in('=')?;
    dbg!(&result);
    Ok(())
}

#[test]
fn is_normal() {
    let a: f64 = 0.0;
    dbg!(a.is_finite());
}