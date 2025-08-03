//Some useful links:
//https://rust-lang.github.io/rust-bindgen/tutorial-3.html
//https://michael-f-bryan.github.io/rust-ffi-guide/
//https://medium.com/dwelo-r-d/using-c-libraries-in-rust-13961948c72a
//https://medium.com/dwelo-r-d/wrapping-unsafe-c-libraries-in-rust-d75aeb283c65
//https://github.com/rdkit/rdkit/blob/master/Code/MinimalLib/cffi_test.c

extern crate bindgen;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use bindgen::CargoCallbacks;

//one may need to set LD_LIBRARY_PATH manually if binary is called without cargo
//e.g. export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/home/username/calc/rdkitcffi/lib/rdkitcffi_linux/linux-64

fn download_rdkit_artifact() -> Option<String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    
    // Detect target OS
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else {
            "linux".to_string()
        }
    });
    
    let (lib_dir, artifact_name, release_tag) = if target_os == "windows" {
        (
            format!("{}/rdkitcffi_windows/windows-64", manifest_dir),
            "rdkitcffi_windows.zip".to_string(),
            "rdkit-windows-latest".to_string()
        )
    } else {
        (
            format!("{}/rdkitcffi_linux/linux-64", manifest_dir),
            "rdkitcffi_linux.tar.gz".to_string(),
            "rdkit-latest".to_string()
        )
    };

    // Create directories if they don't exist
    std::fs::create_dir_all(&lib_dir).ok()?;

    // Download from GitHub release
    let repo_owner = "chrissly31415"; // Replace with your GitHub username
    let repo_name = "rdkitcffi"; // Replace with your repo name

    let download_url = format!(
        "https://github.com/{}/{}/releases/download/{}/{}",
        repo_owner, repo_name, release_tag, artifact_name
    );

    // Download the artifact
    if Command::new("wget")
        .args(["-O", &artifact_name, &download_url])
        .status()
        .ok()?
        .success()
    {
        // Extract the artifact
        if target_os == "windows" {
            // Extract zip file for Windows
            Command::new("unzip")
                .args([&artifact_name, "-d", &manifest_dir])
                .status()
                .ok()?;
        } else {
            // Extract tarball for Linux
            Command::new("tar")
                .args(["xzf", &artifact_name, "-C", &manifest_dir])
                .status()
                .ok()?;
        }

        // Clean up downloaded artifact
        std::fs::remove_file(&artifact_name).ok()?;

        // Create symlinks (only for Linux)
        if target_os != "windows" {
            let lib_files = std::fs::read_dir(&lib_dir).ok()?;
            for entry in lib_files {
                if let Ok(entry) = entry {
                    let filename = entry.file_name();
                    if let Some(name) = filename.to_str() {
                        if name.starts_with("librdkitcffi.so.1.") {
                            Command::new("ln")
                                .args(["-sf", name, &format!("{}/librdkitcffi.so", lib_dir)])
                                .status()
                                .ok()?;
                            Command::new("ln")
                                .args(["-sf", name, &format!("{}/librdkitcffi.so.1", lib_dir)])
                                .status()
                                .ok()?;
                            break;
                        }
                    }
                }
            }
        }

        return Some(lib_dir);
    }
    None
}

fn build_rdkit() -> Option<String> {
    // Don't build on Windows, only download
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else {
            "linux".to_string()
        }
    });
    
    if target_os == "windows" {
        println!("cargo:warning=Local RDKit compilation not supported on Windows. Please ensure the Windows artifact is available for download.");
        return None;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    let lib_dir = format!("{}/rdkitcffi_linux/linux-64", manifest_dir);

    // Clone RDKit
    if !Command::new("git")
        .args([
            "clone",
            "https://github.com/rdkit/rdkit.git",
            "--branch",
            "Release_2021_09",
            "--single-branch",
            "--depth=1",
        ])
        .status()
        .ok()?
        .success()
    {
        return None;
    }

    // Create build directory and run cmake
    std::fs::create_dir_all("rdkit/build").ok()?;
    if !Command::new("cmake")
        .current_dir("rdkit/build")
        .args([
            "..",
            "-DCMAKE_BUILD_TYPE=Release",
            "-DRDK_BUILD_CPP_TESTS=OFF",
            "-DRDK_BUILD_PYTHON_WRAPPERS=OFF",
            "-DRDK_BUILD_COORDGEN_SUPPORT=ON",
            "-DRDK_BUILD_MAEPARSER_SUPPORT=ON",
            "-DRDK_OPTIMIZE_POPCNT=ON",
            "-DRDK_BUILD_INCHI_SUPPORT=ON",
            "-DRDK_BUILD_THREADSAFE_SSS=ON",
            "-DRDK_TEST_MULTITHREADED=ON",
            "-DRDK_BUILD_CFFI_LIB=ON",
        ])
        .status()
        .ok()?
        .success()
    {
        return None;
    }

    // Build
    if !Command::new("make")
        .current_dir("rdkit/build")
        .args(["-j2", "cffi_test"])
        .status()
        .ok()?
        .success()
    {
        return None;
    }

    // Create lib directory and copy files
    std::fs::create_dir_all(&lib_dir).ok()?;
    let source_lib = std::fs::read_dir("rdkit/build/lib")
        .ok()?
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|s| s.starts_with("librdkitcffi.so.1."))
                .unwrap_or(false)
        })?;

    std::fs::copy(
        source_lib.path(),
        format!("{}/{}", lib_dir, source_lib.file_name().to_str()?),
    )
    .ok()?;

    // Create symlinks
    let lib_name = source_lib.file_name();
    Command::new("ln")
        .current_dir(&lib_dir)
        .args(["-sf", lib_name.to_str()?, "librdkitcffi.so"])
        .status()
        .ok()?;
    Command::new("ln")
        .current_dir(&lib_dir)
        .args(["-sf", lib_name.to_str()?, "librdkitcffi.so.1"])
        .status()
        .ok()?;

    Some(lib_dir)
}

fn get_rdkit_lib_path() -> String {
    // First try downloading
    println!("cargo:warning=Attempting to download pre-built RDKit artifact...");
    if let Some(path) = download_rdkit_artifact() {
        println!("cargo:warning=Successfully downloaded RDKit artifact");
        return path;
    }

    // If download fails, try building (only on Linux)
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else {
            "linux".to_string()
        }
    });
    
    if target_os != "windows" {
        println!("cargo:warning=Download failed, attempting to build RDKit...");
        if let Some(path) = build_rdkit() {
            println!("cargo:warning=Successfully built RDKit");
            return path;
        }
    }

    // If both fail, panic with helpful message
    if target_os == "windows" {
        panic!("Failed to download Windows RDKit artifact. Please ensure the Windows artifact is available in the GitHub release.");
    } else {
        panic!("Failed to either download or build RDKit. Please ensure you have internet connection or the required dependencies installed.");
    }
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    println!("out_dir: {:?}", out_dir);

    let shared_lib_dir = get_rdkit_lib_path();
    println!("shared_lib_dir: {}", shared_lib_dir);

    // Detect target OS for linking
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else {
            "linux".to_string()
        }
    });

    // Link configuration
    println!("cargo:rustc-link-search=native={}", shared_lib_dir);
    
    if target_os == "windows" {
        println!("cargo:rustc-link-lib=dylib=rdkitcffi");
        // Set PATH for Windows DLL loading
        println!("cargo:rustc-env=PATH={};{}", shared_lib_dir, env::var("PATH").unwrap_or_default());
    } else {
        println!("cargo:rustc-link-lib=dylib=rdkitcffi");
        // Set LD_LIBRARY_PATH for Linux
        println!("cargo:rustc-env=LD_LIBRARY_PATH={}", shared_lib_dir);
    }

    // Rebuild if header changes
    println!("cargo:rerun-if-changed=include/cffiwrapper.h");

    // Generate bindings
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
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
