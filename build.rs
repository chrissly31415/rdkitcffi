//https://rust-lang.github.io/rust-bindgen/tutorial-3.html
//https://michael-f-bryan.github.io/rust-ffi-guide/
//https://medium.com/dwelo-r-d/using-c-libraries-in-rust-13961948c72a
//https://medium.com/dwelo-r-d/wrapping-unsafe-c-libraries-in-rust-d75aeb283c65
//https://github.com/rdkit/rdkit/blob/master/Code/MinimalLib/cffi_test.c

extern crate bindgen;
use std::env;
use std::path::{PathBuf, Path};
use std::process::Command;

use bindgen::CargoCallbacks;

//use std::env;

//compilation for rdkit commands
//cmake -DRDK_BUILD_MINIMAL_LIB=ON -DRDK_BUILD_CFFI_LIB=ON  -DRDK_BUILD_INCHI_SUPPORT=ON -DRDK_BUILD_PYTHON_WRAPPERS=OFF ..
//currently we cannot compile it directory from a submodule approach via a cmake crate because of the additional dependencies e.g. boost
//do not use lib and main both, as -l gets only used for library

//one need to set LD_LIBRARY_PATH manually if binary is called without cargo
//e.g. export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/home/username/calc/rdkitcffi/lib/rdkitcffi_linux/linux-64

fn download_rdkit_artefact() -> String {
    //download rdkit cffi lib from azure
    let out_dir = env::var("OUT_DIR").unwrap();

    let cffi_url = "https://artprodsu6weu.artifacts.visualstudio.com/A85885aaa-9279-4937-b9af-77f592b58cf7/b9a21ad4-0deb-4a21-8386-996d0e642d94/_apis/artifact/cGlwZWxpbmVhcnRpZmFjdDovL3Jka2l0LWJ1aWxkcy9wcm9qZWN0SWQvYjlhMjFhZDQtMGRlYi00YTIxLTgzODYtOTk2ZDBlNjQyZDk0L2J1aWxkSWQvMTMwNy9hcnRpZmFjdE5hbWUvcmRraXRjZmZpX2xpbnV40/content?format=zip";
    let cffi_zip= "rdkitcffi.zip";
    let rdkit_branch = ".1.2021.09.11.0";
    let shared_lib_name = "librdkitcffi.so";
    let shared_lib_dir = format!("{}{}",out_dir,"/lib/rdkitcffi_linux/linux-64/");
    let artefact_name = format!("{}{}", shared_lib_name, rdkit_branch);

    Command::new("wget").args(&["-O", cffi_zip, cffi_url]).status().unwrap();
    Command::new("unzip").args(&[ cffi_zip, "-d", &format!("{}{}",out_dir,"/lib")]).status().unwrap();
    Command::new("ln").args(&[ "-s", &artefact_name, &format!("{}{}", shared_lib_dir, shared_lib_name)]).status().unwrap();
    Command::new("ln").args(&[ "-s", &artefact_name, &format!("{}{}.1", shared_lib_dir, shared_lib_name)]).status().unwrap();

    let shared_lib_path = Path::new(&shared_lib_name).join("shared_lib_dir");
    
    if !shared_lib_path.exists() {
        println!("shared_lib_path: {:?}",shared_lib_path);
    } else {
        eprintln!("Could not download rdkti cffi shared library from:\n {}",cffi_url);
        eprintln!("Consider downloading it manually!")
    }

    return shared_lib_dir.to_string();
}


fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    println!("out_dir: {:?}",out_dir);

    let shared_lib_dir = download_rdkit_artefact();
    //let key = "LD_LIBRARY_PATH";
    //#env::set_var(key, shared_lib_dir);

    //this sets the dynamic lib path only during build
    println!("cargo:rustc-link-search=native={}", shared_lib_dir);
    println!("cargo:rustc-link-lib=dylib=rdkitcffi");
    
    println!("cargo:rerun-if-changed=wrapper.h");

    //use this for dynamic lib path cargo test & run
    println!("cargo:rustc-env=LD_LIBRARY_PATH={}",shared_lib_dir);

    //pkg_config::Config::new().probe("boost").unwrap();

    //pkg_config::Config::new().probe("rdkitcffi").unwrap();

    let bindings = bindgen::Builder::default()
        .header("include/cffiwrapper.h")
        .clang_arg("-Iinclude/boost")
        .clang_arg("-Iinclude")
        .allowlist_function("version")
        .allowlist_function("enable_logging")
        .allowlist_function("disable_logging")
        .allowlist_function("get_smiles")
        .allowlist_function("get_mol")
        .allowlist_function("get_inchikey_for_inchi")
        .allowlist_function("get_inchi")
        .allowlist_function("get_molblock") 
        .allowlist_function("get_v3kmolblock") 
        .allowlist_function("get_json") 
        .allowlist_function("canonical_tautomer") 
        .allowlist_function("get_descriptors") 
        .allowlist_function("add_hs")
        .allowlist_function("set_3d_coords") 
        .allowlist_function("set_2d_coords") 
        .allowlist_function("get_svg") 
        .allowlist_function("remove_all_hs") 
        .allowlist_function("get_substruct_matches") 
        .allowlist_function("get_substruct_match") 
        .allowlist_function("get_cxsmiles")
        .allowlist_function("get_smarts") 
        .allowlist_function("get_qmol") 
        .allowlist_function("cleanup") 
        .allowlist_function("neutralize") 
        .allowlist_function("reionize") 
        .allowlist_function("normalize") 
        .allowlist_function("get_morgan_fp")
        .allowlist_function("get_morgan_fp_as_bytes")
        .allowlist_function("get_rdkit_fp")
        .allowlist_function("get_rdkit_fp_as_bytes")
        .allowlist_function("get_pattern_fp")
        .allowlist_function("get_pattern_fp_as_bytes")
        .allowlist_function("free") 
        .allowlist_function("free_ptr") 
        .allowlist_var("size_t")

//TODO
        .allowlist_function("charge_parent")
        .allowlist_function("fragment_parent")
        .allowlist_function("prefer_coordgen")
        .allowlist_function("set_2d_coords_aligned")

        .parse_callbacks(Box::new(CargoCallbacks::new()))

        .generate()

        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("./src");
    //let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());


    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    
 }