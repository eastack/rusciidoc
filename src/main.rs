use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_until, take_until1, take_while, take_while1},
    character::{
        complete::{alpha1, alphanumeric1, anychar, space0, space1},
        is_alphanumeric, is_newline, is_space,
    },
    combinator::{not, opt},
    error::ParseError,
    multi::{many0, many_m_n, separated_list1},
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

pub fn parse_author_line(i: &str) -> IResult<&str, AuthorInfo> {
    let auth = tuple((
        terminated(alphanumeric1, space0),
        opt(delimited(space0, alphanumeric1, space0)),
        opt(delimited(space0, alphanumeric1, space0)),
    ));
    let email = delimited(tag("<"), is_not(">"), tag(">"));

    let (i, ((firstname, middlename, lastname), email)) =
        terminated(pair(auth, opt(email)), tag("\n"))(i)?;

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
    let (i, title) = preceded(tag("= "), terminated(take_until("\n"), tag("\n")))(i)?;
    let (i, auth_info) = opt(parse_author_line)(i)?;
    let (i, attrs) = many0(terminated(parse_doc_attr, tag("\n")))(i)?;

    Ok((
        i,
        DocHeader {
            title,
            auth_info,
            attrs,
        },
    ))
}

pub fn parse_doc_attr(i: &str) -> IResult<&str, DocAttr> {
    let name = delimited(
        tag(":"),
        pair(opt(tag("!")), take_while1(|c| c != ':')),
        tag(":"),
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
Heng Wang <admin@eastack.me>
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
