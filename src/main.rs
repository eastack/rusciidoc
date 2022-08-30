use nom::{
    bytes::complete::{is_not, take_until, take_while1},
    character::{
        complete::{char, space0, space1},
    },
    combinator::opt,
    IResult,
    multi::many0,
    sequence::{delimited, pair, preceded, terminated, tuple},
};
use nom::bytes::complete::{is_a, take_till, take_while};
use nom::character::{is_alphabetic, is_space};
use nom::character::complete::{anychar, line_ending};
use nom::combinator::{cut, not};

#[derive(Debug)]
pub struct Document<'a> {
    pub header: Section<'a>,
    pub blocks: Block<'a>,
    pub attrs: Vec<DocAttr<'a>>,
}

#[derive(Debug)]
pub struct DocHeader<'a> {
    pub title: &'a str,
    pub auth_info: Option<AuthorInfo<'a>>,
    pub attrs: Vec<DocAttr<'a>>,
}

#[derive(Debug)]
pub struct DocAttr<'a> {
    pub unset: bool,
    pub name: &'a str,
    pub value: Option<&'a str>,
}

#[derive(Debug, Default, PartialEq)]
pub struct AuthorInfo<'a> {
    pub author: Name<'a>,
    pub email: Option<&'a str>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Name<'a> {
    pub firstname: &'a str,
    pub middle_name: Option<&'a str>,
    pub lastname: Option<&'a str>,
}

#[derive(Debug, Default)]
pub struct Section<'a> {
    pub title: &'a str,
    pub blocks: &'a Vec<Block<'a>>,
}

#[derive(Debug, Default)]
pub struct Block<'a> {
    pub lines: &'a str,
}

//pub fn single_revnumber(input: &str) -> IResult<&str, &str> {
//    delimited(
//        char('v'),
//        take_while1(|c: char| c.is_numeric() || c == '.'),
//        pair(space0, ),
//    )
//}
//
//pub fn parse_revnumber(input: &str) -> IResult<&str, &str> {
//    delimited(
//        space0,
//        take_while1(|c: char| c.is_numeric() || c == '.'),
//        space0,
//    )
//}
//
//pub fn parse_revdata(input: &str) -> IResult<&str, &str> {
//    delimited(
//        space0,
//        take_while1(|c: char| c.is_numeric() || c == '-'),
//        space0,
//    )
//}
//
//pub fn parse_revremark(input: &str) -> IResult<&str, &str> {
//    preceded(pair(char(':'), space0), (is_not('\n'), char('\n')))
//}
//
//pub fn parse_revision(input: &str) -> IResult<&str, &str> {
//    alt((
//        preceded(char('v'), parse_revnumber),
//        tuple((parse_revnumber, char(','), parse_revdata)),
//        tuple((parse_revnumber, char(','), parse_revdata, parse_revremark)),
//    ));
//}

pub fn name(input: &str) -> IResult<&str, &str> {
    // is_not(line_ending)(input)
    // not(line_ending::<&str, &str>)
    is_not("\n\t ")(input)
}

/// 解析文档头部的作者信息
pub fn parse_author_line(i: &str) -> IResult<&str, AuthorInfo> {
    let auth = tuple((
        terminated(name, space0),
        opt(terminated(name, space0)),
        opt(terminated(name, space0)),
    ));
    let email = terminated(delimited(char('<'), is_not(">"), char('>')), space0);

    let (i, ((firstname, middlename, lastname), email)) =
        terminated(pair(auth, opt(email)), line_ending)(i)?;

    Ok((
        i,
        AuthorInfo {
            author: Name {
                firstname,
                middle_name: lastname.and(middlename),
                lastname: lastname.or(middlename),
            },
            email,
        },
    ))
}


pub fn parse_doc_header(i: &str) -> IResult<&str, DocHeader> {
    let (i, title) = preceded(
        pair(char('='), space1),
        terminated(is_not("\r\n"), line_ending),
    )(i)?;

    let (i, auth_info) = opt(parse_author_line)(i)?;
    let (i, attrs) = many0(terminated(parse_doc_attr, line_ending))(i)?;

    Ok((
        i,
        DocHeader {
            title,
            auth_info,
            attrs,
        },
    ))
}

pub fn parse_

pub fn parse_doc_section(i: &str) -> IResult<&str, DocContent> {
    let line = delimited(line_ending, is_not("\r\n"), line_ending)(i)?;
}

/// 解析文档属性
pub fn parse_doc_attr(i: &str) -> IResult<&str, DocAttr> {
    let name = delimited(
        preceded(char(':'), space0),
        pair(opt(char('!')), take_while1(|c| c != ':')),
        char(':'),
    );

    let value = delimited(space1, is_not("\r\n"), line_ending);

    let (i, ((unset, name), value)) = pair(name, opt(value))(i)?;

    Ok((
        i,
        DocAttr {
            unset: unset.is_some(),
            name,
            value,
        },
    ))
}

fn main() {
    let doc = r#"= Rsciidoc
Wang  Yue Heng  <admin@eastack.me>
:hello: world
:!toc:
"#;
    let (_, doc_header) = parse_doc_header(doc).unwrap();
    println!("Doc header: {doc_header:?}")
}

#[cfg(test)]
mod tests {
    use nom::Parser;

    use super::*;

    #[test]
    pub fn test_parse_attr() {
        let (_, attrs) = parse_doc_attr(":hello: world\r\n").unwrap();
        assert_eq!(attrs.name, "hello");
        assert_eq!(attrs.value, Some("world"));
    }

    fn assert_parse_auth_line(input: &str, expected: &(&str, AuthorInfo)) -> Result<(), String> {
        let auth_info = parse_author_line(input).unwrap();
        if auth_info != *expected {
            Err("".to_string())
        } else {
            Ok(())
        }
    }

    #[test]
    pub fn test_parse_auth_line() {
        let test_data = [
            (
                "Wang\n",
                (
                    "",
                    AuthorInfo {
                        author: Name {
                            firstname: "Wang",
                            middle_name: None,
                            lastname: None,
                        },
                        email: None,
                    },
                ),
            ),
            (
                "Wang \n",
                (
                    "",
                    AuthorInfo {
                        author: Name {
                            firstname: "Wang",
                            middle_name: None,
                            lastname: None,
                        },
                        email: None,
                    },
                ),
            ),
            (
                "Wang Heng\n",
                (
                    "",
                    AuthorInfo {
                        author: Name {
                            firstname: "Wang",
                            middle_name: None,
                            lastname: Some("Heng"),
                        },
                        email: None,
                    },
                ),
            ),
            (
                "Wang Heng \n",
                (
                    "",
                    AuthorInfo {
                        author: Name {
                            firstname: "Wang",
                            middle_name: None,
                            lastname: Some("Heng"),
                        },
                        email: None,
                    },
                ),
            ),
            (
                "Wang Yue Heng\n",
                (
                    "",
                    AuthorInfo {
                        author: Name {
                            firstname: "Wang",
                            middle_name: Some("Yue"),
                            lastname: Some("Heng"),
                        },
                        email: None,
                    },
                ),
            ),
            (
                "Wang Yue Heng \n",
                (
                    "",
                    AuthorInfo {
                        author: Name {
                            firstname: "Wang",
                            middle_name: Some("Yue"),
                            lastname: Some("Heng"),
                        },
                        email: None,
                    },
                ),
            ),
            (
                "Wang Yue Heng <admin@eastack.me>\n",
                (
                    "",
                    AuthorInfo {
                        author: Name {
                            firstname: "Wang",
                            middle_name: Some("Yue"),
                            lastname: Some("Heng"),
                        },
                        email: Some("admin@eastack.me"),
                    },
                ),
            ),
            (
                "Wang Yue Heng <admin@eastack.me> \n",
                (
                    "",
                    AuthorInfo {
                        author: Name {
                            firstname: "Wang",
                            middle_name: Some("Yue"),
                            lastname: Some("Heng"),
                        },
                        email: Some("admin@eastack.me"),
                    },
                ),
            ),
        ];

        test_data
            .iter()
            .try_for_each(|(input, expected)| assert_parse_auth_line(input, expected))
            .unwrap();
    }

    #[test]
    fn test() {
        fn alpha(i: &[u8]) -> IResult<&[u8], &[u8]> {
            take_while(is_alphabetic)(i)
        }

        let result = alpha(b"hello123");
        println!("Result: {result:?}");
    }
}
