use nom::{IResult, Parser, error::ParseError};

#[macro_export]
macro_rules! nom_return {
    ($in_:tt, $output_t:ty, $x:expr) => {
        match || -> crate::Result<$output_t> { Ok($x) }() {
            Ok(v) => Ok(($in_, v)),
            Err(e) => Err(nom::Err::Error(e)),
        }
    };
}

pub fn cond_if<I, E, O, F1, F2>(
    cond: bool,
    mut f1: F1,
    mut f2: F2,
) -> impl FnMut(I) -> IResult<I, O, E>
where
    E: ParseError<I>,
    F1: Parser<I, O, E>,
    F2: Parser<I, O, E>,
{
    move |in_: I| {
        if cond {
            f1.parse(in_)
        } else {
            f2.parse(in_)
        }
    }
}
