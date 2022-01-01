//! Some more involved examples for using the cffi interface functions
//!
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


use libc;

use std::mem;

use std::ffi::CStr;
use std::ffi::CString;
use std::fs::read_to_string;
use std::os::raw::c_char;
use std::str;

use crate::bindings::{add_hs, enable_logging, remove_all_hs, set_3d_coords, version};
use crate::bindings::{
    canonical_tautomer, get_descriptors, get_inchi, get_json, get_mol, get_molblock, get_smiles,
};
use crate::bindings::{free, free_ptr, size_t};

use super::{JsonBase, Molecule};

use std::io::Cursor;


    #[cfg(test)]
    mod tests {
        use polars::df;
        use polars::prelude::*;

        use super::*;
        #[test]
        fn polars_df1() {
            let mut mol_list: Vec<Molecule> = crate::read_sdfile("data/test.sdf");
            mol_list.iter_mut().for_each(|m| m.remove_all_hs());
            let a: Vec<_> = mol_list.iter().map(|m| m.get_smiles("")).collect();
            let b: Vec<_> = mol_list
                .iter()
                .map(|m| m.get_smiles("").len() as u32)
                .collect();

            mol_list.iter_mut().for_each(|m| m.canonical_tautomer(""));
            let c: Vec<_> = mol_list.iter().map(|m| m.get_smiles("")).collect();
            let mut df = df!( "smiles" => a, "len" => b, "can_tautomer" => c).unwrap();

            let vala = df.column("smiles").unwrap();
            let valb = df.column("can_tautomer").unwrap();
            let mask = vala.neq(valb);
            df = df.filter(&mask).unwrap();
            println!("{}", df);
        }

        #[test]
        #[ignore]
        fn polars_doc() {
            use polars::df;
            use polars::prelude::*;

            unsafe {
                enable_logging();
            }

            let mut mol_list: Vec<Molecule> = crate::read_sdfile("data/large.sdf"); // ist failing....
            let a: Vec<_> = mol_list.iter().map(|m| m.get_smiles("")).collect();
            println!("here");
            mol_list.iter_mut().for_each(|m| m.canonical_tautomer(""));
            println!("here2");
            let b: Vec<_> = mol_list.iter().map(|m| m.get_smiles("")).collect();

            let mut df = df!( "smiles" => a, "can_tautomer" => b).unwrap();

            let vala = df.column("smiles").unwrap();
            let valb = df.column("can_tautomer").unwrap();
            let mask = vala.neq(valb);
            df = df.filter(&mask).unwrap();
        }

        #[test]
        fn polars_df2() {
            let mol_list: Vec<Molecule> = crate::read_sdfile("data/test.sdf");
            // we are using json to deal with dimension
            let basic_json = mol_list
                .iter()
                .map(|m| m.get_descriptors_as_string())
                .collect::<Vec<String>>()
                .iter()
                .map(|s| format!("{}\n", s))
                .collect::<String>();
            let file = Cursor::new(basic_json);
            let df2 = JsonReader::new(file)
                .infer_schema(Some(3))
                .with_batch_size(4)
                .finish()
                .unwrap();
            println!("{}", df2);
        }

        
        #[test]
        fn sdf2inchi() {
            unsafe {
                let orig_sdf = CString::new("\n     RDKit          2D\n\n  7  6  0  0  0  0  0  0  0  0999 V2000\n    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    1.2990    0.7500    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0\n    2.5981   -0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    3.8971    0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    5.1962   -0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    6.4952    0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    7.7942    1.5000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n  1  2  1  0\n  2  3  1  0\n  3  4  1  0\n  4  5  2  3\n  5  6  1  0\n  6  7  3  0\nM  END\n").unwrap();
                let rdkit_json = CString::new("").unwrap();
                let pkl_size: *mut size_t = libc::malloc(mem::size_of::<u64>()) as *mut u64;
                let pkl_mol: *mut c_char =
                    get_mol(orig_sdf.as_ptr(), pkl_size, rdkit_json.as_ptr());
                let inchi_cchar = get_inchi(pkl_mol, *pkl_size, rdkit_json.as_ptr());
                println!("InChi: {:#?}", CStr::from_ptr(inchi_cchar));
                free(pkl_size as *mut libc::c_void);
                free_ptr(pkl_mol);
                free_ptr(inchi_cchar);
            }
        }
        #[test]
        fn json2inchi() {
            unsafe {
                let orig_json = CString::new(r#"{"commonchem":{"version":10},"defaults":{"atom":{"z":6,"impHs":0,"chg":0,"nRad":0,"isotope":0,"stereo":"unspecified"},"bond":{"bo":1,"stereo":"unspecified"}},"molecules":[{"atoms":[{"impHs":3},{"z":8},{"impHs":2},{"impHs":1},{"impHs":1},{},{"impHs":1}],"bonds":[{"atoms":[0,1]},{"atoms":[1,2]},{"atoms":[2,3]},{"bo":2,"atoms":[3,4]},{"atoms":[4,5]},{"bo":3,"atoms":[5,6]}],"conformers":[{"dim":3,"coords":[[-1.923,0.6284,-0.2289],[-1.5806,-0.5257,-0.956],[-0.2954,-0.8968,-0.8422],[0.2046,-1.2165,0.5395],[1.1511,-0.4958,1.0587],[1.8054,0.5537,0.5447],[2.3445,1.3997,0.1438]]}],"extensions":[{"name":"rdkitRepresentation","formatVersion":2,"toolkitVersion":"2021.09.1pre"}]}]}"#).unwrap();
                let add_json = CString::new("").unwrap();
                let pkl_size: *mut size_t = libc::malloc(mem::size_of::<u64>()) as *mut u64;
                let pkl_mol: *mut c_char = get_mol(orig_json.as_ptr(), pkl_size, add_json.as_ptr());
                let inchi_cchar = get_inchi(pkl_mol, *pkl_size, add_json.as_ptr());
                println!("InChi: {:#?}", CStr::from_ptr(inchi_cchar));
                free(pkl_size as *mut libc::c_void);
                free_ptr(pkl_mol);
                free_ptr(inchi_cchar);
            }
        }
        #[test]
        fn sdfile2inchi() {
            unsafe {
                //only reading first file currently
                let sdf_string =
                    read_to_string("./data/test.sdf").expect("Could not load SD file.");
                let sdf_string: CString = CString::new(sdf_string).unwrap();
                let add_json = CString::new("").unwrap();

                let pkl_size: *mut size_t = libc::malloc(mem::size_of::<u64>()) as *mut u64;
                let pkl_mol: *mut c_char =
                    get_mol(sdf_string.as_ptr(), pkl_size, add_json.as_ptr());

                //get molecule as json object
                let rdkit_json_cchar = get_json(pkl_mol, *pkl_size, add_json.as_ptr());
                let mol_json_str = CStr::from_ptr(rdkit_json_cchar).to_str().unwrap();
                let rdkit_json_object: JsonBase =
                    serde_json::from_str(mol_json_str).expect("Wrong JSON format!");

                println!("{}", mol_json_str);

                for k in rdkit_json_object.molecules.iter() {
                    println!("Name: {:?}\n\n", k.name);
                    println!("Name: {:?}\n\n", k.extensions);
                }

                free(pkl_size as *mut libc::c_void);
                free_ptr(pkl_mol);
                free_ptr(rdkit_json_cchar);
            }
        }
    }

