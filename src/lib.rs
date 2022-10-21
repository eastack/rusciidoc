use nom::{IResult, Parser};
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_till, take_until};
use nom::character::{is_newline, is_space};
use nom::character::complete::{char, multispace0, multispace1, one_of, space1};
use nom::combinator::{eof, map, opt, value};
use nom::multi::{many0, many1_count};
use nom::sequence::{delimited, pair, terminated, tuple};

// Formatting pair
#[derive(Eq, PartialEq, Debug)]
pub enum FormattedText<'a> {
    Strong(&'a str)
}

#[derive(Eq, PartialEq, Debug)]
pub struct Title<'a> {
    pub level: usize,
    pub content: &'a str,
}

pub fn parse_strong_formatting_pair(i: &str) -> IResult<&str, FormattedText> {
    let parse_strong_formatting = delimited(
        multispace1,
        // space1,
        delimited(char('*'), is_not("*"), char('*')),
        one_of(",;\".?! \t"),
    );

    map(parse_strong_formatting, |text| FormattedText::Strong(text))(i)
}

pub fn parse_comment_line(i: &str) -> IResult<&str, ()> {
    value(
        (),
        pair(tag("//"), is_not("\n\r")),
    )(i)
}

pub fn parse_comment_block(i: &str) -> IResult<&str, ()> {
    value(
        (),
        tuple((tag("////"), take_until("////"), tag("////"))),
    )(i)
}

pub fn parse_title(i: &str) -> IResult<&str, Title> {
    let (i, (level, content)) = pair(many1_count(char('=')), is_not("\n\r"))(i)?;
    Ok((i, Title { level, content }))
}

pub fn parse_block(i: &str) -> IResult<&str, Vec<&str>> {
    terminated(many0(parse_line), tag("\n"))(i)
}

pub fn parse_line(i: &str) -> IResult<&str, &str> {
    terminated(is_not("\n"), tag("\n"))(i)
}

#[cfg(test)]
mod tests {
    use nom::multi::many0;

    use crate::{FormattedText, parse_comment_block, parse_comment_line, parse_line, parse_block, parse_strong_formatting_pair, parse_title, Title};

    #[test]
    pub fn test_parse_strong_text() {
        let text = "strong";
        let doc = format!(" *{}* ", text);
        let (i, strong_text) = parse_strong_formatting_pair(&doc).unwrap();
        assert_eq!(i, "");
        assert_eq!(strong_text, FormattedText::Strong(text));

        let text = "Hello World";
        let doc = format!(" *{}*!", text);
        let (i, strong_text) = parse_strong_formatting_pair(&doc).unwrap();
        assert_eq!(i, "");
        assert_eq!(strong_text, FormattedText::Strong(text));
    }

    #[test]
    pub fn test_comment_line() {
        let text = "// I'm comment \n\
                         I'm not comment.\n\
                         I'm content.";
        let (i, r) = parse_comment_line(text).unwrap();
        assert_eq!(i, "\nI'm not comment.\nI'm content.");
        assert_eq!(r, ());
    }

    #[test]
    pub fn test_comment_block() {
        let text = "//// \n\
                         I'm comment \n\
                         I'm not comment. \n\
                         I'm content. \n\
                         ////\n\
                         I'm not comment.\n\
                         I'm content.";
        let (i, r) = parse_comment_block(text).unwrap();
        assert_eq!(i, "\nI'm not comment.\nI'm content.");
        assert_eq!(r, ());
    }

    #[test]
    pub fn test_parse_title() {
        let text = "===== Hello Asciidoctor\nHello World!";
        let (result, title) = parse_title(text).unwrap();
        assert_eq!(result, "\nHello World!");
        assert_eq!(title, Title { level: 5, content: " Hello Asciidoctor" });
    }

    #[test]
    pub fn test_parse_block() {
        let text = "Asciidoctor is a marklanguage.\n\nIs powerful.\nIs simple.\nIs elegent.\n\nThe End.";
        let (result, section) = many0(parse_block)(text).unwrap();
        println!("parse_section result: {result}, section: {section:?}")
        // let v: Vec<&str> = vec![];
        // assert_eq!(result, "");
        // assert_eq!(section, v)
    }

    #[test]
    pub fn test_parse_line() {
        // let text = "Hello World\n\n你好世界";
        let text = "Asciidoctor is a marklanguage.\n\nIs powerful.\nIs simple.\nIs elegent.\n\nThe End.";
        let (result, section) = parse_line(text).unwrap();
        println!("result: {result}, line: {section:?}")
    }
}