use nom::character::complete::{alpha1, char};
use nom::combinator::recognize;
use nom::sequence::separated_pair;
use nom::IResult;

fn main() {
    let (input, result) = recognize_test("abc,def").unwrap();
    println!("input: {input}, result: {result}");
}

fn recognize_test(input: &str) -> IResult<&str, &str> {
    recognize(separated_pair(alpha1, char(','), alpha1))(input)
}