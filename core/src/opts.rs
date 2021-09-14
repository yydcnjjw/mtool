use clap::Clap;

pub trait AppOpts: Clap {
    fn config_path(&self) -> &str;
    fn exec_cmd(&self) -> anyhow::Result<()>;
}
