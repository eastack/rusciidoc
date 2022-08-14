use nom::{
    branch::alt,
    bytes::complete::{is_not, take_until, take_while1},
    character::complete::{char, space0, space1},
    combinator::opt,
    multi::many0,
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

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

#[derive(Debug, Default)]
pub struct AuthorInfo<'a> {
    pub author: Name<'a>,
    pub email: Option<&'a str>,
}

#[derive(Debug, Default)]
pub struct Name<'a> {
    pub firstname: &'a str,
    pub middlename: Option<&'a str>,
    pub lastname: Option<&'a str>,
}

#[derive(Debug, Default)]
pub struct DocContent<'a> {
    pub firstname: &'a str,
    pub middlename: Option<&'a str>,
    pub lastname: Option<&'a str>,
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
    is_not("\n\t ")(input)
}

pub fn parse_author_line(i: &str) -> IResult<&str, AuthorInfo> {
    let auth = tuple((
        terminated(name, space0),
        opt(delimited(space0, name, space0)),
        opt(delimited(space0, name, space0)),
    ));
    let email = delimited(char('<'), is_not(">"), char('>'));

    let (i, ((firstname, middlename, lastname), email)) =
        terminated(pair(auth, opt(email)), char('\n'))(i)?;

    Ok((
        i,
        AuthorInfo {
            author: Name {
                firstname,
                middlename: lastname.and(middlename),
                lastname: lastname.or(middlename),
            },
            email,
        },
    ))
}

pub fn parse_doc_header(i: &str) -> IResult<&str, DocHeader> {
    let (i, title) = preceded(
        pair(char('='), space1),
        terminated(take_until("\n"), char('\n')),
    )(i)?;
    let (i, auth_info) = opt(parse_author_line)(i)?;
    let (i, attrs) = many0(terminated(parse_doc_attr, char('\n')))(i)?;

    Ok((
        i,
        DocHeader {
            title,
            auth_info,
            attrs,
        },
    ))
}

//pub fn parse_doc_content(i: &str) -> IResult<&str, DocContent> {}

pub fn parse_doc_attr(i: &str) -> IResult<&str, DocAttr> {
    let name = delimited(
        char(':'),
        pair(opt(char('!')), take_while1(|c| c != ':')),
        char(':'),
    );
    let value = preceded(space1, take_until("\n"));

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
Heng Yue Wang <admin@eastack.me>
:hello: world
:!toc:
"#;
    let (_, doc_header) = parse_doc_header(doc).unwrap();
    println!("Doc header: {doc_header:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse_attr() {
        let (_, attrs) = parse_doc_attr(":hello: world\n").unwrap();
        assert_eq!(attrs.name, "hello");
        assert_eq!(attrs.value, Some("world"));
    }
}
