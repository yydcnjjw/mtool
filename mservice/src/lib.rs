#[mrpc::service(message(serde, debug))]
pub enum Service {
    Sysev(sysev_mod::Service),
    Config(config_mod::Service),
    Keybinding(keybinding_mod::Service),
    Cmder(cmder_mod::Service),
}
