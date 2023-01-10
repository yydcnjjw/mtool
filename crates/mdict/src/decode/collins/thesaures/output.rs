use std::fmt;

use super::{Sense, Synonym, ThesauresResult};
use crate::decode::collins::output::OutputOrg;

impl OutputOrg for Sense {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write,
    {
        write!(f, "** {} {}\n\n", self.word, self.pos)?;

        if let Some(content) = &self.content {
            write!(f, "{}\n\n", content)?;
        }

        if let Some(example) = &self.example {
            write!(f, "{}\n\n", example)?;
        }

        for synonym in &self.synonyms {
            synonym.output_org(f)?;
        }

        Ok(())
    }
}

impl OutputOrg for Synonym {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write,
    {
        if let Some(query_url) = &self.query_url {
            write!(f, "*** [[{}][{}]]", query_url, self.word)?;
        } else {
            write!(f, "{}", self.word)?;
        }

        write!(f, " ")?;

        if let Some(sound) = &self.sound {
            write!(f, "[[{}][▷]]", sound)?;
        }

        write!(f, "\n")?;

        for example in &self.examples {
            write!(f, "- {}\n", example)?;
        }
        Ok(())
    }
}

impl OutputOrg for ThesauresResult {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write,
    {
        write!(f, "* {}\n", self.source)?;

        for sense in &self.senses {
            sense.output_org(f)?;
        }

        Ok(())
    }
}
