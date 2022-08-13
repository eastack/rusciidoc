use nom::{
    bytes::complete::{tag, take_until},
    sequence::{preceded, terminated, pair},
    IResult, combinator::map, character::complete::digit1,
};

#[derive(Debug)]
pub struct DocHeader<'a> {
    pub title: &'a str,
}

pub fn parse_doc_header(i: &str) -> IResult<&str, DocHeader> {
    let (i, title) = preceded(tag("= "), terminated(take_until("\n"), tag("\n")))(i)?;
    Ok((i, DocHeader { title }))
}

#[derive(Debug)]
pub struct DocAttr<'a> {
}

pub fn parse_doc_attr(i: &str) -> IResult<&str, DocAttr> {
    pair(first, second)
    terminated(first, second)
    digit1(input)

    map()
}




fn main() {
    let doc = r#"= Asciidoctor
"#;

    let res = parse_doc_header(doc).unwrap();
    println!("Res: {res:?}");
}

pub fn parse_doc_attr(i: &str) -> IResult<&str, >
