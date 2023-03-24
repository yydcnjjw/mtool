use std::{fs::File, str::FromStr};

use anyhow::Context;
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
pub struct Router {
    rules: Vec<Rule>,
}

impl Router {
    pub fn new(config: RoutingConfig) -> Result<Self, anyhow::Error> {
        let sources: Vec<geosite::SiteGroupList> = config
            .source
            .iter()
            .map(|source| -> Result<geosite::SiteGroupList, anyhow::Error> {
                let mut f = File::open(&source)?;
                geosite::SiteGroupList::parse_from_reader(&mut f).context("Failed to parse geosite")
            })
            .try_collect()?;

        let rules: Vec<Rule> = config
            .rule
            .into_iter()
            .map(|config| Rule::new(config, &sources))
            .try_collect()?;

        Ok(Self { rules })
    }

    #[instrument(skip(self))]
    pub fn route(&self, source: &String, remote: &NetLocation) -> Result<String, anyhow::Error> {
        self.rules
            .iter()
            .find_or_first(|rule| {
                rule.source.contains(source)
                    && rule.matcher.reverse_query(&remote.address.to_string())
            })
            .map(|rule| rule.dest.clone())
            .context(format!("{}-{} is not matched", source, remote.to_string()))
    }
}

pub struct Rule {
    pub id: String,
    matcher: MphMatcher,
    source: Vec<String>,
    dest: String,
}

impl std::fmt::Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rule").field("dest", &self.dest).finish()
    }
}

enum RuleType {
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

impl Rule {
    pub fn new(
        config: RuleConfig,
        sources: &Vec<geosite::SiteGroupList>,
    ) -> Result<Self, anyhow::Error> {
        let mut matcher = MphMatcher::new(1);

        for target in config.target {
            match Self::parse_target(&target) {
                Ok((rule_type, value)) => match rule_type {
                    RuleType::Domain => matcher.reverse_insert(value, MatchType::Domain(true)),
                    RuleType::Full => matcher.reverse_insert(value, MatchType::Full(true)),
                    RuleType::SubStr => matcher.reverse_insert(value, MatchType::SubStr(true)),
                    RuleType::Geosite => Self::insert_geosite(sources, value, &mut matcher)?,
                },
                Err(e) => {
                    warn!("{} format is incorrect: {}", target, e);
                }
            }
        }
        matcher.build();

        Ok(Self {
            id: config.id,
            matcher,
            source: config.source,
            dest: config.dest,
        })
    }

    fn parse_target(value: &str) -> Result<(RuleType, &str), anyhow::Error> {
        let (rule_type, value) = value.split_once(':').context("need ':'")?;
        Ok((RuleType::from_str(rule_type)?, value))
    }

    fn insert_geosite(
        sources: &Vec<geosite::SiteGroupList>,
        tag: &str,
        matcher: &mut MphMatcher,
    ) -> Result<(), anyhow::Error> {
        let sg = sources
            .iter()
            .find_map(|source| {
                source
                    .site_group
                    .iter()
                    .find(|sg| sg.tag == tag.to_uppercase())
            })
            .context(format!("geosite:{} isn't exist", tag))?;

        for domain in &sg.domain {
            match domain.type_.unwrap() {
                geosite::domain::Type::Plain => {
                    matcher.reverse_insert(&domain.value, MatchType::SubStr(true))
                }
                geosite::domain::Type::Domain => {
                    matcher.reverse_insert(&domain.value, MatchType::Domain(true))
                }
                geosite::domain::Type::Full => {
                    matcher.reverse_insert(&domain.value, MatchType::Full(true))
                }
                geosite::domain::Type::Regex => {}
            }
        }

        Ok(())
    }
}
