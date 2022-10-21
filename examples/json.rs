use std::collections::HashMap;
use nom::branch::alt;

use nom::bytes::complete::{escaped, tag, take_while};
use nom::character::complete::{alphanumeric1 as alphanumeric, one_of};
use nom::combinator::value;
use nom::error::ParseError;
use nom::IResult;

pub enum JsonValue {
    Null,
    Str(String),
    Boolean(bool),
    Num(f64),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

/// 解析器组合是自底向上构建的
/// 首先我们为最小的元素写解析器（这里是空字符）
fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c))(i)
}

/// 一个 nom 解析器有以下签名：
/// `Input -> IResult<Input, Output, Error>`, with `IResult` defined as:
/// `type IResult<I, O, E = error::Error<I>> = Result<(I, O), Err<E>>;`
///
/// 多数情况下你可以省略错误类型并使用默认类型（但本示例稍后将会展示自定义错误）
///
/// 这里我们使用 `&str` 作为输入类型，但 nom 解析器的输入类型可以是范型的
/// 并且可以直接使用 `&[u8]` 或其他任何实现了所需特质的类型。
///
/// 最后，我们可以看到这里输入和输出都是 `&str`
/// 并拥有相同的生命周期标签。这意味着产生的值是输入数据的子集。
/// 并不需要进行内存分配。这是 nom 高性能背后的主要技巧。
fn parser_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    escaped(alphanumeric, '\\', one_of("\"\n\\"))(i)
}

/// `tag(string)` 生成一个识别参数字符串的解析器
///
/// 我们可以将其与另一个函数结合，比如 `value` 它取得另一个解析器，
/// 并且如果解析器没有返回错误，则返回一个给定的常量值。
///
/// `alt` 是另一个组合子其尝试逐个尝试多个解析器，知道一个成功。
fn boolean<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, bool, E> {
    // 这是一个解析器，当它遇到 "true" 时返回 `true`，否则返回错误。
    let parse_true = value(true, tag("true"));

    // 这是一个解析器，当它遇到 "false" 时返回 `false`，否则返回错误。
    let parse_false = value(false, tag("false"));

    // `alt` 组合两个解析器。它返回第一个成功解析器的值，否则返回一个错误。
    alt((parse_true, parse_false))(input)
}

fn null<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    value((), tag("null"))(input)
}

fn main() {
    println!("Hello world!")
}