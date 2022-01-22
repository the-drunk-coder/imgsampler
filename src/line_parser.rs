use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::is_alphanumeric,
    combinator::{map, map_res, recognize},
    error::VerboseError,
    multi::{separated_list0, separated_list1},
    number::complete::float,
    sequence::{delimited, separated_pair},
    IResult,
};

pub enum ParserResult {
    String(String),
    Scalar(f32),
    Bounce(Vec<f32>),
    Ramp(Vec<f32>),
    Choose(Vec<f32>),
    Cycle(Vec<f32>),
}

fn parse_param<'a>(i: &'a str) -> IResult<&'a str, ParserResult, VerboseError<&'a str>> {
    alt((
        map(parse_float, |f| ParserResult::Scalar(f)),
        delimited(
            tag("["),
            alt((
                map(parse_float, |f| ParserResult::Scalar(f)),
                map(
                    separated_pair(
                        tag("ramp"),
                        tag(" "),
                        separated_list0(tag(" "), parse_float),
                    ),
                    |v| ParserResult::Ramp(v.1),
                ),
                map(
                    separated_pair(
                        tag("bounce"),
                        tag(" "),
                        separated_list0(tag(" "), parse_float),
                    ),
                    |v| ParserResult::Bounce(v.1),
                ),
                map(
                    separated_pair(
                        tag("choose"),
                        tag(" "),
                        separated_list0(tag(" "), parse_float),
                    ),
                    |v| ParserResult::Choose(v.1),
                ),
                map(
                    separated_pair(
                        tag("cycle"),
                        tag(" "),
                        separated_list0(tag(" "), parse_float),
                    ),
                    |v| ParserResult::Cycle(v.1),
                ),
            )),
            tag("]"),
        ),
    ))(i)
}

fn parse_float<'a>(i: &'a str) -> IResult<&'a str, f32, VerboseError<&'a str>> {
    map_res(recognize(float), |digit_str: &str| digit_str.parse::<f32>())(i)
}

/// valid chars for a function name
fn valid_char(chr: char) -> bool {
    chr == '_' || chr == '.' || chr == '-' || is_alphanumeric(chr as u8)
}

fn parse_string<'a>(i: &'a str) -> IResult<&'a str, ParserResult, VerboseError<&'a str>> {
    map(take_while(valid_char), |desc_str: &str| {
        ParserResult::String(desc_str.to_string())
    })(i)
}

pub fn parse_line<'a>(i: &'a str) -> IResult<&'a str, Vec<ParserResult>, VerboseError<&'a str>> {
    separated_list1(tag(" "), alt((parse_param, parse_string)))(i)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_line_parser() {
        let result = parse_line(
            "img forest.jpg pos 0.2 [0.2] [bounce 0.2 0.3] [ramp 0.2 0.3] [choose 0.2 0.3 400]",
        )
        .unwrap();
        println!("{}", result.0);
        assert!(matches!(result.1[0], ParserResult::String(_)));
        assert!(matches!(result.1[1], ParserResult::String(_)));
        assert!(matches!(result.1[2], ParserResult::String(_)));
        assert!(matches!(result.1[3], ParserResult::Scalar(_)));
        assert!(matches!(result.1[4], ParserResult::Scalar(_)));
        assert!(matches!(result.1[5], ParserResult::Bounce(_)));
        assert!(matches!(result.1[6], ParserResult::Ramp(_)));
        assert!(matches!(result.1[7], ParserResult::Choose(_)));
    }
}
