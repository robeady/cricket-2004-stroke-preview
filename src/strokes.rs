use nom::{
    branch::alt,
    bytes::{complete::tag_no_case, streaming::take_till},
    character::streaming::{alphanumeric1, line_ending, space1},
    combinator::{map_res, opt},
    error::{convert_error, Error, VerboseError},
    multi::many1,
    sequence::{pair, preceded, terminated, tuple},
    IResult, Needed,
};
use std::str::from_utf8;

use anyhow::anyhow;

#[derive(Debug, PartialEq)]
pub struct Stroke {
    pub timings_normal: [StrokeTiming; 5],
    pub timings_hit6: [StrokeTiming; 5],
}

#[derive(Debug, PartialEq)]
pub struct StrokeTiming {
    pub vertical: f64,
    pub direction: f64,
    pub direction_area: f64,
    pub power: f64,
    pub power_area: f64,
}

type CResult<'a, T = &'a [u8]> = IResult<&'a [u8], T, Error<&'a [u8]>>;

fn kv<'a, 'b: 'a>(k: &'b str) -> impl FnMut(&'a [u8]) -> CResult<'a> {
    preceded(pair(tag_no_case(k), space1), alphanumeric1)
}

fn kv_line<'a, 'b: 'a>(k: &'b str) -> impl FnMut(&'a [u8]) -> CResult<'a, f64> {
    map_res(map_res(terminated(kv(k), many1(alt((space1, line_ending)))), from_utf8), str::parse)
}

fn kv2_line<'a, 'b: 'a>(
    k1: &'b str,
    k2: &'b str,
) -> impl FnMut(&'a [u8]) -> CResult<'a, (f64, f64)> {
    terminated(
        pair(
            map_res(map_res(terminated(kv(k1), space1), from_utf8), str::parse),
            map_res(map_res(kv(k2), from_utf8), str::parse),
        ),
        many1(line_ending),
    )
}

fn parse_stroke<'a>(b: &'a [u8]) -> CResult<'a, Option<Stroke>> {
    let (b, _) = match take_till(|c| c == b't')(b) {
        Err(nom::Err::Incomplete(_)) => return Ok((b, None)),
        r => r,
    }?;
    let (b, _) = match tag_no_case("trokeAttributes")(b) {
        Err(_) => return Ok((b, None)),
        r => r,
    }?;

    let (b, _typ) = preceded(many1(line_ending), terminated(kv("Type"), many1(line_ending)))(b)?;
    let (b, _edge_modifier) = kv_line("EdgeModifier")(b)?;
    let (b, _edge_modifier6) = opt(kv_line("EdgeModifier6"))(b)?;
    let (b, (_Difficulty, _Reward)) = kv2_line("Difficulty", "Reward")(b)?;
    let (b, (_Difficulty6, _Reward6)) = kv2_line("Difficulty6", "Reward6")(b)?;
    let (b, (_EdgeProb, _EdgeProb6)) = kv2_line("EdgeProb", "EdgeProb6")(b)?;
    let (b, _BowlerTypes) = kv_line("BowlerTypes")(b)?;
    let (b, _BallStumps) = kv_line("BallStumps")(b)?;
    let (b, _BallBatsman) = kv_line("BallBatsman")(b)?;
    let (b, _BallLength) = kv_line("BallLength")(b)?;

    let keyframe = |mode, idx, b| -> IResult<_, _> {
        let (b, _) = tuple((
            tag_no_case("Mode"),
            space1,
            tag_no_case(mode),
            space1,
            tag_no_case("KeyFrame"),
            space1,
            tag_no_case(idx),
            many1(line_ending),
        ))(b)?;
        let (b, _) = kv_line("Frame")(b)?;
        let (b, vertical) = kv_line("Vertical")(b)?;
        let (b, (direction, direction_area)) = kv2_line("Direction", "DirectionArea")(b)?;
        let (b, (power, power_area)) = kv2_line("Power", "PowerArea")(b)?;
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

    Ok((
        b,
        Some(Stroke {
            timings_normal: [normal_0, normal_1, normal_2, normal_3, normal_4],
            timings_hit6: [hit6_0, hit6_1, hit6_2, hit6_3, hit6_4],
        }),
    ))
}

impl Stroke {
    pub fn parse(b: &[u8]) -> anyhow::Result<Option<Stroke>> {
        let (_remaining, stroke) = parse_stroke(b).map_err(|e| match e {
            nom::Err::Incomplete(Needed::Size(u)) => {
                anyhow!("Error parsing stroke: parsing requires {} bytes", u)
            }
            nom::Err::Incomplete(Needed::Unknown) => {
                anyhow!("Error parsing stroke: parsing requires more data")
            }
            nom::Err::Failure(v) | nom::Err::Error(v) => {
                let input = String::from_utf8_lossy(v.input);
                let input_slice =
                    input.char_indices().nth(10).map(|(i, _)| &input[..i]).unwrap_or(&input);
                anyhow!("Error parsing stroke: {:?} at {:?}", v.code, input_slice)
            }
        })?;
        Ok(stroke)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_stroke_successfully() {
        let stroke = b"\0\0SStrokeAttributes

Type Defensive
EdgeModifier 10
EdgeModifier6 10
Difficulty 0 Reward 0
Difficulty6 0 Reward6 0
EdgeProb 0 EdgeProb6 0
BowlerTypes 15
BallStumps 3
BallBatsman 3
BallLength 2

Mode Normal KeyFrame 0
Frame 10
Vertical 90596966
Direction 242980370 DirectionArea 24298037
Power 144631 PowerArea 361578

Mode Normal KeyFrame 1
Frame 14
Vertical 90596966
Direction 242980370 DirectionArea 24298037
Power 144631 PowerArea 361578

Mode Normal KeyFrame 2
Frame 18
Vertical 90596966
Direction 242980370 DirectionArea 24298037
Power 144631 PowerArea 361578

Mode Normal KeyFrame 3
Frame 21
Vertical 90596966
Direction 242980370 DirectionArea 24298037
Power 144631 PowerArea 361578

Mode Normal KeyFrame 4
Frame 24
Vertical 90596966
Direction 242980370 DirectionArea 24298037
Power 144631 PowerArea 361578

Mode 6Hit KeyFrame 0
Frame 10
Vertical 90596966
Direction 242980370 DirectionArea 24298037
Power 144631 PowerArea 361578

Mode 6Hit KeyFrame 1
Frame 14
Vertical 90596966
Direction 242980370 DirectionArea 24298037
Power 144631 PowerArea 361578

Mode 6Hit KeyFrame 2
Frame 18
Vertical 90596966
Direction 242980370 DirectionArea 24298037
Power 144631 PowerArea 361578

Mode 6Hit KeyFrame 3
Frame 21
Vertical 90596966
Direction 242980370 DirectionArea 24298037
Power 144631 PowerArea 361578

Mode 6Hit KeyFrame 4
Frame 24
Vertical 90596966
Direction 242980370 DirectionArea 24298037
Power 144631 PowerArea 361578
\0\0";

        let expected = Stroke {
            timings_normal: [
                StrokeTiming {
                    vertical: 90596966.0,
                    direction: 242980370.0,
                    direction_area: 24298037.0,
                    power: 144631.0,
                    power_area: 361578.0,
                },
                StrokeTiming {
                    vertical: 90596966.0,
                    direction: 242980370.0,
                    direction_area: 24298037.0,
                    power: 144631.0,
                    power_area: 361578.0,
                },
                StrokeTiming {
                    vertical: 90596966.0,
                    direction: 242980370.0,
                    direction_area: 24298037.0,
                    power: 144631.0,
                    power_area: 361578.0,
                },
                StrokeTiming {
                    vertical: 90596966.0,
                    direction: 242980370.0,
                    direction_area: 24298037.0,
                    power: 144631.0,
                    power_area: 361578.0,
                },
                StrokeTiming {
                    vertical: 90596966.0,
                    direction: 242980370.0,
                    direction_area: 24298037.0,
                    power: 144631.0,
                    power_area: 361578.0,
                },
            ],
            timings_hit6: [
                StrokeTiming {
                    vertical: 90596966.0,
                    direction: 242980370.0,
                    direction_area: 24298037.0,
                    power: 144631.0,
                    power_area: 361578.0,
                },
                StrokeTiming {
                    vertical: 90596966.0,
                    direction: 242980370.0,
                    direction_area: 24298037.0,
                    power: 144631.0,
                    power_area: 361578.0,
                },
                StrokeTiming {
                    vertical: 90596966.0,
                    direction: 242980370.0,
                    direction_area: 24298037.0,
                    power: 144631.0,
                    power_area: 361578.0,
                },
                StrokeTiming {
                    vertical: 90596966.0,
                    direction: 242980370.0,
                    direction_area: 24298037.0,
                    power: 144631.0,
                    power_area: 361578.0,
                },
                StrokeTiming {
                    vertical: 90596966.0,
                    direction: 242980370.0,
                    direction_area: 24298037.0,
                    power: 144631.0,
                    power_area: 361578.0,
                },
            ],
        };

        assert_eq!(Stroke::parse(stroke).unwrap(), Some(expected));
    }

    #[test]
    fn ignores_other_stuff() {
        assert_eq!(Stroke::parse(b"\0; Ball Conditions (Ball) Cricket 2004").unwrap(), None);
    }
}
