use clap::app_from_crate;

use common_args::{read_props, AppExt};

fn main() {
    let matches = app_from_crate!().add_javac_common_args().get_matches();

    let props = read_props(&matches);
}
