use nom::bytes::complete::is_not;
use nom::character::complete::multispace0;
use nom::IResult;
use nom::multi::many0;
use nom::sequence::delimited;

fn main() {
    let (input, result) = parser(" Hello World\n\n你好世界.").unwrap();
    println!("input: '{input}', result: '{result:?}'");
}

fn parser(input: &str) -> IResult<&str, Vec<&str>> {
    many0(delimited(multispace0, is_not("\n"), multispace0))(input)
}
