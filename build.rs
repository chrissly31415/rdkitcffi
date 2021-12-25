//https://rust-lang.github.io/rust-bindgen/tutorial-3.html
//https://michael-f-bryan.github.io/rust-ffi-guide/
//https://medium.com/dwelo-r-d/using-c-libraries-in-rust-13961948c72a

extern crate bindgen;
use std::path::PathBuf;
//use std::env;


//compilation for rdkit commands
//cmake -DRDK_BUILD_MINIMAL_LIB=ON -DRDK_BUILD_CFFI_LIB=ON  -DRDK_BUILD_INCHI_SUPPORT=ON -DRDK_BUILD_PYTHON_WRAPPERS=OFF ..
//currently we cannot compile it directory from a submodule approach via a cmake crate because of the additional dependencies e.g. boost
//do not use lib and main both, as -l gets only used for library

//one need to set LD_LIBRARY_PATH manually if binary is called without cargo
//e.g. export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/home/loschen/calc/rust_cheminf/rdkitcffi/lib/rdkitcffi_linux/linux-64

fn main() {
    let shared_lib_dir = "./lib/rdkitcffi_linux/linux-64/";
    //let key = "LD_LIBRARY_PATH";
    //#env::set_var(key, shared_lib_dir);

    //this sets the dynamic lib path only during build
    println!("cargo:rustc-link-search={}", shared_lib_dir);

    println!("cargo:rustc-link-lib=dylib=rdkitcffi");

    println!("cargo:rerun-if-changed=wrapper.h");

    //use this for dynamic lib path cargo test & run
    println!("cargo:rustc-env=LD_LIBRARY_PATH={}",shared_lib_dir);

    //pkg_config::Config::new().probe("rdkitcffi").unwrap();

    let bindings = bindgen::Builder::default()
        //.trust_clang_mangling(false)
        .header("include/wrapper.h")
        .clang_arg("-I/home/loschen/programs/boost_1_67_0")
      
        .allowlist_function("version")
        .allowlist_function("enable_logging")
        .allowlist_function("get_smiles")
        .allowlist_function("get_mol")
        .allowlist_function("get_inchikey_for_inchi")
        .allowlist_function("get_inchi")
        .allowlist_function("get_molblock") 
        .allowlist_function("get_json") 
        .allowlist_function("canonical_tautomer") 
        .allowlist_function("get_descriptors") 
        .allowlist_function("add_hs")
        .allowlist_function("set_3d_coords") 
        .allowlist_function("remove_all_hs") 
        .allowlist_function("free") 
        .allowlist_function("remove_all_hs") 
        .allowlist_function("free_ptr") 
        .allowlist_var("size_t") 
 
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))

        .generate()

        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("./src");
    //let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());


    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    
 }