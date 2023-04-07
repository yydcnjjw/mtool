use std::{fs::File, path::PathBuf, str::FromStr, sync::Arc};

use anyhow::Context;
use dashmap::DashMap;
use domain_matcher::{mph::MphMatcher, DomainMatcher, MatchType};
use itertools::Itertools;
use protobuf::Message;
use tracing::{instrument, warn};

use crate::{
    config::{
        protos::geosite,
        routing::{RoutingConfig, RuleConfig},
    },
    proxy::NetLocation,
};

#[derive(Debug)]
pub struct GeositeFile {
    file: PathBuf,
    sgl: geosite::SiteGroupList,
}

impl GeositeFile {
    pub fn new(file: &PathBuf) -> Result<Self, anyhow::Error> {
        Ok(Self {
            file: file.clone(),
            sgl: {
                if file.exists() {
                    let mut f = File::open(&file)?;
                    geosite::SiteGroupList::parse_from_reader(&mut f)
                        .context("Failed to parse geosite")?
                } else {
                    geosite::SiteGroupList::new()
                }
            },
        })
    }

    fn get_site_group(&self, tag: &str) -> Option<&geosite::SiteGroup> {
        self.sgl
            .site_group
            .iter()
            .find(|sg| sg.tag == tag.to_uppercase())
    }

    fn get_site_group_or_create_mut(&mut self, tag: &str) -> &mut geosite::SiteGroup {
        let sg = &mut self.sgl.site_group;
        if let Some(i) = sg.iter().position(|sg| sg.tag == tag.to_uppercase()) {
            &mut sg[i]
        } else {
            sg.push(geosite::SiteGroup {
                tag: tag.to_uppercase(),
                domain: Vec::new(),
                ..Default::default()
            });
            sg.last_mut().unwrap()
        }
    }

    pub fn insert(
        &mut self,
        tag: &str,
        rule_type: RuleType,
        value: &str,
    ) -> Result<(), anyhow::Error> {
        let sg = self.get_site_group_or_create_mut(tag);

        sg.domain.push(geosite::Domain {
            type_: match rule_type {
                RuleType::Domain => geosite::domain::Type::Domain,
                RuleType::Full => geosite::domain::Type::Full,
                RuleType::SubStr => geosite::domain::Type::Plain,
                RuleType::Geosite => anyhow::bail!("geosite rule is not supported"),
            }
            .into(),
            value: value.to_string(),
            ..Default::default()
        });
        Ok(())
    }

    pub fn store(&self) -> Result<(), anyhow::Error> {
        let mut f = File::create(&self.file)?;
        self.sgl.write_to_writer(&mut f)?;
        Ok(())
    }
}

#[derive(Debug)]
struct GeositeResource {
    list: Vec<GeositeFile>,
}

impl GeositeResource {
    fn new(path: Vec<PathBuf>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            list: path
                .iter()
                .map(|file| GeositeFile::new(file))
                .try_collect()?,
        })
    }
}

#[derive(Debug)]
pub struct Resource {
    geosite: GeositeResource,
}

impl Resource {
    fn new(path: Vec<PathBuf>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            geosite: GeositeResource::new(path)?,
        })
    }

    fn get_geosite_tag(&self, tag: &str) -> Result<&geosite::SiteGroup, anyhow::Error> {
        self.geosite
            .list
            .iter()
            .find_map(|file| file.get_site_group(tag))
            .context(format!("geosite:{} isn't exist", tag))
    }
}

#[derive(Debug)]
pub struct Router {
    _res: Arc<Resource>,
    rules: DashMap<String, Rule>,
}

impl Router {
    pub fn new(config: RoutingConfig) -> Result<Self, anyhow::Error> {
        let res = Arc::new(Resource::new(config.resource)?);

        let rules: DashMap<String, Rule> = config
            .rule
            .into_iter()
            .map(|config| {
                Ok::<_, anyhow::Error>((config.id.clone(), Rule::new(config, res.clone())?))
            })
            .try_collect()?;

        Ok(Self { _res: res, rules })
    }

    #[instrument(skip(self))]
    pub fn route(&self, src: &String, remote: &NetLocation) -> Result<String, anyhow::Error> {
        self.rules
            .iter()
            .find_or_first(|rule| {
                rule.src.contains(src) && rule.matcher.reverse_query(&remote.address.to_string())
            })
            .map(|rule| rule.dest.clone())
            .context(format!("{}-{} is not matched", src, remote.to_string()))
    }

    pub fn add_rule_target(&self, id: &str, target: &str) -> Result<(), anyhow::Error> {
        self.rules
            .get_mut(id)
            .context(format!("{} is not exist", id))?
            .add(target)
    }
}

pub struct Rule {
    pub id: String,
    matcher: MphMatcher,
    src: Vec<String>,
    dest: String,

    res: Arc<Resource>,
}

impl std::fmt::Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rule").field("dest", &self.dest).finish()
    }
}

pub enum RuleType {
    Domain,
    Full,
    SubStr,
    Geosite,
}

impl FromStr for RuleType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "f" => Self::Full,
            "s" => Self::SubStr,
            "d" => Self::Domain,
            "geosite" => Self::Geosite,
            _ => anyhow::bail!("{} is not supported", s),
        })
    }
}

pub fn parse_target(value: &str) -> Result<(RuleType, &str), anyhow::Error> {
    let (rule_type, value) = value.split_once(':').context("need ':'")?;
    Ok((RuleType::from_str(rule_type)?, value))
}

impl Rule {
    pub fn new(config: RuleConfig, res: Arc<Resource>) -> Result<Self, anyhow::Error> {
        let matcher = MphMatcher::new(1);
        let mut this = Self {
            id: config.id,
            matcher,
            src: config.src,
            dest: config.dest,

            res,
        };
        for target in config.target {
            if let Err(e) = this.add(&target) {
                warn!("add rule target error: {}", e);
            }
        }
        this.matcher.build();

        Ok(this)
    }

    pub fn add(&mut self, target: &str) -> Result<(), anyhow::Error> {
        let (rule_type, value) = parse_target(&target)?;
        match rule_type {
            RuleType::Domain => self.matcher.reverse_insert(value, MatchType::Domain(true)),
            RuleType::Full => self.matcher.reverse_insert(value, MatchType::Full(true)),
            RuleType::SubStr => self.matcher.reverse_insert(value, MatchType::SubStr(true)),
            RuleType::Geosite => self.insert_geosite(value)?,
        }
        Ok(())
    }

    fn insert_geosite(&mut self, tag: &str) -> Result<(), anyhow::Error> {
        let sg = self.res.get_geosite_tag(tag)?;
        for domain in &sg.domain {
            match domain.type_.unwrap() {
                geosite::domain::Type::Plain => self
                    .matcher
                    .reverse_insert(&domain.value, MatchType::SubStr(true)),
                geosite::domain::Type::Domain => self
                    .matcher
                    .reverse_insert(&domain.value, MatchType::Domain(true)),
                geosite::domain::Type::Full => self
                    .matcher
                    .reverse_insert(&domain.value, MatchType::Full(true)),
                geosite::domain::Type::Regex => {}
            }
        }

        Ok(())
    }
}
