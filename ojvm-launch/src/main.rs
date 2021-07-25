use clap::{app_from_crate, Arg, ArgGroup};

fn main() {
    let matches = app_from_crate!()
        .args(&[Arg::new("shell"), Arg::new("powershell"), Arg::new("json")])
        .group(ArgGroup::new("Select the output format").args(&["shell", "powershell", "json"]))
        .get_matches();
}
