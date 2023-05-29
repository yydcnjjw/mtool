mod cmd;
mod llama;
mod openai;
mod tencent;
mod translator;

use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule, CreateOnceTaskDescriptor};
use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_core::{
    config::{is_startup_mode, StartupMode},
    AppStage, CmdlineStage, ConfigStore,
};
use mtool_system::keybinding::Keybinding;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector()
            .construct_once(tencent::Translator::construct);
        app.injector().construct_once(openai::Translator::construct);
        app.injector().construct_once(llama::Translator::construct);

        app.schedule()
            .add_once_task(CmdlineStage::AfterInit, register_command)
            .add_once_task(
                AppStage::Init,
                register_keybinding.cond(is_startup_mode(StartupMode::WGui)),
            );
        Ok(())
    }
}

async fn register_command(cmder: Res<Cmder>, cs: Res<ConfigStore>) -> Result<(), anyhow::Error> {
    if cs.startup_mode() == StartupMode::Cli {
        cmder
            .add_command(
                cmd::text_translate_into_english
                    .name("text_translate_into_english")
                    .add_alias("te")
                    .desc("Translate into English"),
            )
            .add_command(
                cmd::text_translate_into_chinese
                    .name("text_translate_into_chinese")
                    .add_alias("tz")
                    .desc("Translate into Chinese"),
            )
            .add_command(
                cmd::text_translate_into_japanese
                    .name("text_translate_into_japanese")
                    .add_alias("tj")
                    .desc("Translate into Japanese"),
            );
    } else {
        cmder
            .add_command(
                cmd::text_translate_into_english_wgui
                    .name("text_translate_into_english")
                    .add_alias("te")
                    .desc("Translate into English"),
            )
            .add_command(
                cmd::text_translate_into_chinese_wgui
                    .name("text_translate_into_chinese")
                    .add_alias("tz")
                    .desc("Translate into Chinese"),
            )
            .add_command(
                cmd::text_translate_into_japanese_wgui
                    .name("text_translate_into_japanese")
                    .add_alias("tj")
                    .desc("Translate into Japanese"),
            );
    }

    Ok(())
}

async fn register_keybinding(_keybinding: Res<Keybinding>) -> Result<(), anyhow::Error> {
    // keybinding
    //     .define_global("M-A-e", cmd::te_interactive)
    //     .await?;
    // keybinding
    //     .define_global("M-A-z", cmd::tz_interactive)
    //     .await?;
    Ok(())
}
