use std::fmt;

use crate::decode::collins::output::OutputOrg;

use super::{
    dict::{Define, Example, Hom, Sense, Text, WordForm, XR},
    DictResult,
};

impl OutputOrg for WordForm {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write,
    {
        write!(f, "{}", self.form)?;
        if let Some(content) = &self.content {
            write!(f, ": {}", content)?;
        }

        if let Some(sound) = &self.sound {
            write!(f, " [[{}][▷]]", sound)?;
        }

        Ok(())
    }
}

impl OutputOrg for Text {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write,
    {
        if self.bold {
            write!(f, "*{}*", self.content)?;
        } else if let Some(url) = &self.query_url {
            write!(f, "[[{}][{}]]", url, self.content)?;
        } else {
            write!(f, "{}", self.content)?;
        }
        Ok(())
    }
}

impl OutputOrg for Define {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write,
    {
        for text in &self.content {
            text.output_org(f)?;
        }
        Ok(())
    }
}

impl OutputOrg for Example {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write,
    {
        write!(f, "{}", self.content)?;

        if let Some(syntax) = &self.syntax {
            write!(f, "*{}*", syntax)?;
        }

        if let Some(sound) = &self.sound {
            write!(f, " [[{}][▷]]", sound)?;
        }

        Ok(())
    }
}

impl OutputOrg for XR {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write,
    {
        write!(f, "[[{}][{}]]", self.query_url, self.phrase)?;
        Ok(())
    }
}

impl OutputOrg for Sense {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write,
    {
        if let Some(gram) = &self.gram {
            write!(f, "*[{}]* ", gram)?;
        }

        if let Some(define) = &self.define {
            define.output_org(f)?;
            write!(f, "\n")?;
        }

        for example in &self.examples {
            write!(f, "  - ")?;
            example.output_org(f)?;
            write!(f, "\n")?;
        }

        if !self.xrs.is_empty() {
            if self.define.is_some() {
                write!(f, "  - ")?;
            }

            if let Some(hint) = &self.xr_hint {
                write!(f, "{}:", hint)?;
            }

            for xr in &self.xrs {
                write!(f, " ")?;
                xr.output_org(f)?;
            }

            write!(f, "\n")?;
        }

        if !self.sub_sense.is_empty() {
            if self.define.is_none() && self.gram.is_none() {
                write!(f, "[-]\n")?;
            }

            for sense in &self.sub_sense {
                write!(f, "  - ")?;
                sense.output_org(f)?;
                write!(f, "\n")?;
            }
        }

        Ok(())
    }
}

impl OutputOrg for Hom {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write,
    {
        write!(f, "** ")?;

        if let Some(pos) = &self.pos {
            write!(f, "{}", pos)?;
        } else {
            write!(f, "[-]")?;
        }

        if let Some(syntax) = &self.syntax {
            write!(f, "{}", syntax)?;
        }
        write!(f, "\n")?;

        for sense in &self.senses {
            write!(f, "- ")?;
            sense.output_org(f)?;
        }

        Ok(())
    }
}

impl OutputOrg for DictResult {
    fn output_org<Fmt>(&self, f: &mut Fmt) -> fmt::Result
    where
        Fmt: fmt::Write,
    {
        write!(f, "* {}", self.word)?;
        if let Some(source) = &self.source {
            write!(f, " {}", source)?;
        }

        write!(f, "\n")?;

        if let Some(pron) = &self.pronounce {
            write!(f, "{}", pron)?;
        }

        if let Some(sound) = &self.sound {
            write!(f, " [[{}][▷]]", sound)?;
        }

        write!(f, "\n")?;

        if !self.wfs.is_empty() {
            write!(f, "- word form:\n")?;
            for wf in &self.wfs {
                write!(f, "  - ")?;
                wf.output_org(f)?;
                write!(f, "\n")?;
            }
        }

        for hom in &self.homs {
            hom.output_org(f)?;
        }

        Ok(())
    }
}
