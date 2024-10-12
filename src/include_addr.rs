use cargo_toml::Manifest;
use nutt_conf_parser::NuttConfig;
use proc_macro::TokenStream;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use syn::__private::quote::quote;
use syn::__private::ToTokens;
use walkdir::{DirEntry, WalkDir};

pub fn include_addr(_input: TokenStream) -> TokenStream {
    let crate_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let wd = WalkDir::new(crate_path.parent().unwrap());
    let mut conf_str = "".to_string();
    for dir in wd {
        try_read_conf(&mut conf_str, dir)
    }
    let mut current_dir = Some(crate_path.as_path());
    if conf_str.is_empty() {
        while let Some(dir) = current_dir {
            for dir in WalkDir::new(dir).max_depth(1) {
                try_read_conf(&mut conf_str, dir)
            }
            current_dir = dir.parent();
        }
    }


    let conf = toml::from_str::<NuttConfig>(&conf_str).expect("Parse error");
    let manifest =
        Manifest::from_path(crate_path.join("Cargo.toml")).expect("Cargo.toml not found");
    let package_name = manifest.package.expect("Package not found").name;
    let service_conf = conf
        .get_service_config(&package_name)
        .expect("Service not found in conf");
    let (host, port) = service_conf.get_addr();
    let output = quote! {
        static LOCAL_ADDR: (&str, u16) = (#host, #port);
        static DOCKER_ADDR: (&str, u16) = (#package_name, 80);
    };

    output.into()
}

fn try_read_conf(buf: &mut String, dir: walkdir::Result<DirEntry>) {
    if let Ok(dir) = dir {
        if dir.file_name().to_str().unwrap() == "nutt.conf.toml" {
            File::open(dir.path())
                .unwrap()
                .read_to_string(buf)
                .unwrap();
        }
    }
}