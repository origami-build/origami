use clap::app_from_crate;

fn main() {
    let matches = app_from_crate!().get_matches();
}
