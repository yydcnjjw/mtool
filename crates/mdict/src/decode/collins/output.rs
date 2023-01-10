use std::fmt;

pub trait OutputOrg {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write;

    fn to_string_org(&self) -> Result<String, fmt::Error> {
        let mut s = String::new();
        self.output_org(&mut s)?;
        Ok(s)
    }
}
