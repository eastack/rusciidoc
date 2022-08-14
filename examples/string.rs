use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while_m_n},
    character::{complete::multispace1, streaming::char},
    combinator::{map, map_opt, map_res, value, verify},
    error::{FromExternalError, ParseError},
    multi::fold_many0,
    sequence::{delimited, preceded},
    IResult,
};

// 解析器组合器是自低向上构建的：
// 我们先从最小的元素（转义字符）开始写解析器，
// 然后将它们组合为一个大的解析器。

/// 当 XXXX 是 1 到 6 个十六进制数时解析一个 u{XXXX} 这种格式的Unicode 序列。
/// 我们会把它和后边的 parse_escaped_char 组合来解析像 \u{00AC} 这样的序列。
fn parse_unicode<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    // `take_while_m_n` 解析 `m` 和 `n` 个（包括）匹配条件的字节。
    // 这里 `parse_hex` 解析 1 到 6 个十六进制数。
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());

    // `preceded` 取一个前缀解析器，并且如果解析成功时返回主体解析器。
    // 在当前情况下，用来解析 u{XXXX}。
    let parse_delimited_hex = preceded(
        char('u'),
        // `delimited` 和 `preceded` 类似，但它同时解析前缀和后缀。
        // 他返回中间解析器的结果。在当前示例中，它解析 {XXXX}，
        // 当 XXXX 是 1 到 6 个十六进制数时返回 XXXX。
        delimited(char('{'), parse_hex, char('}')),
    );

    // `map_res` 从解析器获取一个结果并将一个函数应用到返回的结果上然后返回一个 Result。
    // 在当前示例中我们从 parse_hex 获取十六进制字节并尝试将其转换为 u32。
    let parse_u32 = map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16));

    // map_opt 和 map_res 类似，但它取一个 Option 而不是 Result。
    // 如果这个函数返回 None，map_opt 则会返回一个错误。
    // 在当前示例中，因为不是所有 u32 值是合法的 Unicode 码点，
    // 我们必须可失败的使用 from_u32 将其转换为 char。
    map_opt(parse_u32, |value| std::char::from_u32(value))(input)
}

/// 解析一个转义字符：\n，\t，\r，\u{00AC}等。
fn parse_escaped_char<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    preceded(
        char('\\'),
        // `alt` 尝试序列中的每一个解析器，
        // 返回第一个成功匹配产生的结果
        alt((
            parse_unicode,
            // `value` 解析器返回一个固定的值（其第一个参数）如果它的解析器（第二个参数）成功的话。
            // 在当前示例中，它会查看标记字符（n，r，t等）并返回匹配的字符（\n，\r，\t等）。
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )(input)
}

/// 解析一个反斜线，跟随任意数量的空字符。
/// 这在后边用来丢弃任意转义的空字符。
fn parse_escaped_whitespace<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    preceded(char('\\'), multispace1)(input)
}

/// 解析一个非空文本块，其中不包含 \ 或 "
fn parse_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    // `is_not` 从 0 或多个字符中解析一个字符串当其中没有给定的字符时。
    let not_quote_slash = is_not("\"\\");

    // `verify` 运行一个解析器，然后在解析器返回结果上执行一个校验函数。
    // 校验函数只有在其返回 true 时才允许输出返回。
    // 在当前示例中，我们希望确保 is_not 的输出不是空的。
    verify(not_quote_slash, |s: &str| !s.is_empty())(input)
}

/// 一个字符串片段包含一个已被解析的字符串：
/// 要么是一个非空 Literal（一系列未转义的字符串）,
/// 一个单一被解析的转义字符，
/// 或一个被转义的空字符块。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

/// 组合 parse_literal、 parse_escaped_whitespace 和 parse_escaped_char
/// 到一个 StringFragment 中。
fn parse_fragment<'a, E>(input: &'a str) -> IResult<&'a str, StringFragment<'a>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    alt((
        // `map` 将一个解析器的执行和一个函数应用到输出上相结合。
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ))(input)
}

/// 解析一个字符串。使用 parse_fragment 循环将所有片段放到一个输出字符串中。
fn parse_string<'a, E>(input: &'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    // fold_many0 和 iterator::fold 是等价的。
    // 它在一个循环中运行一个解析器并对每个输出值调用一个折叠函数。
    let build_string = fold_many0(
        // 我们的解析器函数解析一个单一的字符串片段
        parse_fragment,
        // 我们的初始值，一个空字符串
        String::new,
        // 我们的折叠函数。对每个片段，添加片段到字符串中。
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(s) => string.push(s),
                StringFragment::EscapedWS => {}
            }
            string
        },
    );

    // 最后，解析字符串。注意，如果 `build_string` 可以接受一个原始
    // " character, the closing delimiter" 将永远不会匹配。
    // 当与一个循环解析器（比如 fold_many0）使用 `delimited` 时，确保
    // 循环不会意外的匹配你的结束分隔符！
    delimited(char('"'), build_string, char('"'))(input)
}

fn main() {
    let data = "\"abc\"";
    println!("EXAMPLE 1:\nParsing a simple input string: {}", data);
    let result = parse_string::<()>(data);
    assert_eq!(result, Ok(("", String::from("abc"))));

    let data = "\"tab:\\tafter tab, newline:\\nnew line, quote: \\\", emoji: \\u{1F602}, newline:\\nescaped whitespace: \\    abc\"";
    println!(
    "EXAMPLE 2:\nParsing a string with escape sequences, newline literal, and escaped whitespace:\n\n{}\n",
    data
  );
    let result = parse_string::<()>(data);
    assert_eq!(
    result,
    Ok((
      "",
      String::from("tab:\tafter tab, newline:\nnew line, quote: \", emoji: 😂, newline:\nescaped whitespace: abc")
    ))
  );
    println!("Result:\n\n{}", result.unwrap().1);
}
