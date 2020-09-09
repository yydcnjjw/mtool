use std::collections::HashMap;

use super::Command;

use anyhow::Context;

type Cmd = Box<dyn Command>;

pub struct Commander {
    cmds: HashMap<String, Cmd>,
}

impl Commander {
    pub fn new() -> Self {
        Self {
            cmds: HashMap::new(),
        }
    }

    pub async fn exec(&mut self, name: &String, args: &[String]) -> anyhow::Result<()> {
        let cmd = self
            .get(name)
            .with_context(|| format!("Command `{}` not found", name))?;
        cmd.exec(args.to_vec()).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn list_command_name(&self) -> Vec<&String> {
        self.cmds.keys().collect::<_>()
    }

    pub fn get(&mut self, name: &String) -> Option<&mut Cmd> {
        self.cmds.get_mut(name)
    }

    pub fn insert(&mut self, name: String, cmd: Cmd) {
        self.cmds.insert(name, cmd);
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, name: String) {
        self.cmds.remove(&name);
    }
}
