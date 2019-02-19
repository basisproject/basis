extern crate exonum_build;

use exonum_build::{get_exonum_protobuf_files_path, protobuf_generate};

fn main() {
    let exonum_protos = get_exonum_protobuf_files_path();
    protobuf_generate(
        "src/block/models/proto",
        &["src/block/models/proto", &exonum_protos],
        "protobuf_mod.rs",
    );
}

