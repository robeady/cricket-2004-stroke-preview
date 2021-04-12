use std::{
    num::ParseFloatError,
    str::{from_utf8, Utf8Error},
    string::FromUtf8Error,
};

use nom::{
    bytes::complete::tag_no_case,
    character::complete::{alphanumeric1, line_ending, space1},
    combinator::map_res,
    error::{ContextError, Error, FromExternalError, ParseError},
    multi::many1,
    sequence::{pair, preceded, terminated, tuple},
    IResult, Parser,
};

pub struct Stroke {
    pub timings_normal: [StrokeTiming; 5],
    pub timings_hit6: [StrokeTiming; 5],
}

/* impl FnMut(&'a [u8]) -> */
fn parse_kv_line<'a, E>(k: &str, b: &'a [u8]) -> IResult<&'a [u8], f64, E>
where
    E: ParseError<&'a [u8]>
        + FromExternalError<&'a [u8], ParseFloatError>
        + FromExternalError<&'a [u8], Utf8Error>,
{
    map_res(
        map_res(
            terminated(preceded(pair(tag_no_case(k), space1), alphanumeric1), many1(line_ending)),
            from_utf8,
        ),
        |s| s.parse::<f64>(),
    )(b)
}

impl Stroke {
    pub fn parse<'a>(b: &'a [u8]) -> anyhow::Result<Stroke> {
        let kv_line = |k, b: &'a _| -> IResult<_, _, Error<&'a _>> {
            map_res(
                map_res(
                    terminated(
                        preceded(pair(tag_no_case(k), space1), alphanumeric1),
                        many1(line_ending),
                    ),
                    from_utf8,
                ),
                str::parse,
            )(b)
        };

        let kv2_line = |k1, k2, b| -> IResult<_, _> {
            terminated(
                pair(
                    map_res(
                        map_res(
                            terminated(
                                preceded(pair(tag_no_case(k1), space1), alphanumeric1),
                                space1,
                            ),
                            from_utf8,
                        ),
                        str::parse,
                    ),
                    map_res(
                        map_res(preceded(pair(tag_no_case(k2), space1), alphanumeric1), from_utf8),
                        str::parse,
                    ),
                ),
                many1(line_ending),
            )(b)
        };

        let ke2_line = |k1, e1, k2, e2, b| -> IResult<_, _> {
            tuple((
                tag_no_case(k1),
                space1,
                tag_no_case(e1),
                space1,
                tag_no_case(k2),
                space1,
                tag_no_case(e2),
                many1(line_ending),
            ))(b)
        };

        let (b, typ) = kv_line("Type", b)?;
        let (b, edge_modifier) = kv_line("EdgeModifier", b)?;
        let (b, edge_modifier6) = kv_line("EdgeModifier6", b)?;
        let (b, (Difficulty, Reward)) = kv2_line("Difficulty", "Reward", b)?;
        let (b, (Difficulty6, Reward6)) = kv2_line("Difficulty6", "Reward6", b)?;
        let (b, (EdgeProb, EdgeProb6)) = kv2_line("EdgeProb", "EdgeProb6", b)?;
        let (b, BowlerTypes) = kv_line("BowlerTypes", b)?;
        let (b, BallStumps) = kv_line("BallStumps", b)?;
        let (b, BallBatsman) = kv_line("BallBatsman", b)?;
        let (b, BallLength) = kv_line("BallLength", b)?;

        let keyframe = |mode, idx, b| -> IResult<_, _> {
            let (b, _) = ke2_line("Mode", mode, "KeyFrame", idx, b)?;
            let (b, _) = kv_line("Frame", b)?;
            let (b, vertical) = kv_line("Vertical", b)?;
            let (b, (direction, direction_area)) = kv2_line("Direction", "DirectionArea", b)?;
            let (b, (power, power_area)) = kv2_line("Power", "PowerArea", b)?;
            Ok((b, StrokeTiming { vertical, direction, direction_area, power, power_area }))
        };

        let (b, normal_0) = keyframe("Normal", "0", b)?;
        let (b, normal_1) = keyframe("Normal", "1", b)?;
        let (b, normal_2) = keyframe("Normal", "2", b)?;
        let (b, normal_3) = keyframe("Normal", "3", b)?;
        let (b, normal_4) = keyframe("Normal", "4", b)?;

        let (b, hit6_0) = keyframe("6Hit", "0", b)?;
        let (b, hit6_1) = keyframe("6Hit", "1", b)?;
        let (b, hit6_2) = keyframe("6Hit", "2", b)?;
        let (b, hit6_3) = keyframe("6Hit", "3", b)?;
        let (b, hit6_4) = keyframe("6Hit", "4", b)?;

        Ok(Stroke {
            timings_normal: [normal_0, normal_1, normal_2, normal_3, normal_4],
            timings_hit6: [hit6_0, hit6_1, hit6_2, hit6_3, hit6_4],
        })
    }
}

pub struct StrokeTiming {
    pub vertical: f64,
    pub direction: f64,
    pub direction_area: f64,
    pub power: f64,
    pub power_area: f64,
}
