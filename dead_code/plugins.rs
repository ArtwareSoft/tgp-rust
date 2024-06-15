use super::tgp::{Comp, StaticString, StdHashMap};

#[derive(Debug)]
pub struct Plugin {
    comps: Vec<Comp>,
    base_dir: StaticString,
    dsl: StaticString,
    files: Vec<String>,
    dependent: Vec<StaticString>
}

#[derive(Debug)]
struct RawPluginDir {
    base_dir: String,
    files: Vec<File>,
}
#[derive(Debug)]
struct RawFile {
    path: String,
    content: String,
}
#[derive(Debug)]
struct File {
    path: String,
    content: String,
    using: Vec<StaticString>,
    dsl: StaticString,
    plugin_dsl: StaticString
}

#[derive(Debug)]
pub struct TgpModel {
    pub comps: StdHashMap<StaticString, Comp>,
    pub plugins: Vec<Plugin>
}