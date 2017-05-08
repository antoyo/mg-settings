extern crate mg_settings;
#[macro_use]
extern crate mg_settings_macros;

#[derive(Commands)]
pub enum AppCommand {
    #[help(text="Show the text in the label")]
    Show(String),
    Quit,
}

#[derive(Settings)]
pub struct AppSettings {
    boolean: bool,
}
