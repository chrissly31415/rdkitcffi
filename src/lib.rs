//! This is an experimental rust wrapper for some functionality of the great open source cheminformatics [RDKit](https://www.rdkit.org/) library.
//!
//! It makes use of its new (and also still experimental) C Foreign Function Interface (cffi) functionality, see also this [blog post](https://greglandrum.github.io/rdkit-blog/technical/2021/05/01/rdkit-cffi-part1.html).
//!
//! Use it at your own risk, its not yet recommended for productive use and only available for linux :-)
//!
//! Please note, that only a limited functionality is being exposed via cffi by RDKit. Structured data is
//! transferred from the backend via the cffi interface as string types. Addiitional arguments can be passed as json strings.  
//! This also means that the structure of objects is different from the C/C++ and python APIs.  
//!
//! [github repository](https://github.com/chrissly31415/rdkitcffi).
//!
//! Please have a look at the examples below and the test functions.  
//!
//! # Examples
//!
//!
//! ```
//! use rdkitcffi::Molecule;
//!
//! let smiles = "OCCC#CO";
//! let mol = Molecule::new(smiles, "").unwrap();
//!
//! let natoms = mol.get_numatoms();
//! ```
//!
//! Additional arguments can be passed via json
//!
//! ```
//! use rdkitcffi::Molecule;
//!
//! let json_args = "{\"removeHs\":false,\"canonical\":false}";
//! let mol = Molecule::new("c1cc(O[H])ccc1", json_args).unwrap();
//! ```
//!
//! Working with SD files and filtering invalid molecules (=None):
//!
//! ```
//!use rdkitcffi::{Molecule,read_sdfile};
//!
//! let mut mol_opt_list : Vec<Option<Molecule>>= read_sdfile("data/test.sdf");
//! let mut mol_list: Vec<Molecule> = mol_opt_list.into_iter().filter_map(|m| m).collect();
//! mol_list.iter_mut().for_each(|m| m.remove_all_hs());
//!
//! ```
//!
//! Dealing with invalid molecules (=None)
//!
//! ```
//! use rdkitcffi::Molecule;
//!
//! let result = Molecule::new("OCCO", "");
//! match result {
//!    Some(m) => println!("Result: {:?}", m),
//!    None => println!("Could not get molecule!"),
//!};
//! ```
//!
//!
//! Getting a JSON represenation (via serde_json):
//!
//! ```
//! use rdkitcffi::Molecule;
//!
//! let mol = Molecule::new("OCCO", "").unwrap();
//! println!("json: {:?}", mol.get_json(""));
//!
//! ```
//!
//! Neutralizing a zwitterion
//!
//! ```
//! use rdkitcffi::Molecule;
//!
//! let mut mol = Molecule::new("C(C(=O)[O-])[NH3+]", "").unwrap();
//! mol.neutralize("");
//! println!("{:?}", mol.get_smiles(""));
//!
//! ```
//!
//! Computing RDKit descriptors
//!
//! ```
//! use rdkitcffi::Molecule;
//!
//! let mol = Molecule::new("CCCN", "").unwrap();
//! let desc = mol.get_descriptors_as_dict();
//! let nrot = desc.get("NumRotatableBonds");
//! let logp = desc.get("CrippenClogP");
//!
//! ```
//!
//! Creating a polars dataframe:
//!
//! ```
//! use rdkitcffi::Molecule;
//! use polars::prelude::*;
//! use polars::df;
//!
//! let mut mol_list : Vec<Molecule> = rdkitcffi::read_smifile_unwrap("data/test.smi");
//! let a: Vec<_> = mol_list.iter().map(|m| m.get_smiles("")).collect();
//! let df = df!( "smiles" => a).unwrap();
//!
//! ```
//!

use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use std::collections::HashMap;

use std::ffi::{CStr, CString};
use std::fmt::Debug;
use std::fs::read_to_string;
use std::mem;
use std::os::raw::{c_char, c_void};

pub mod examples;

pub mod bindings;

use bindings::{add_hs, remove_all_hs, set_3d_coords};
use bindings::{
    canonical_tautomer, charge_parent, cleanup, fragment_parent, neutralize, normalize, reionize,
};
use bindings::{free, free_ptr};
use bindings::{
    get_cxsmiles, get_descriptors, get_inchi, get_inchikey_for_inchi, get_json, get_mol,
    get_molblock, get_qmol, get_smarts, get_smiles, get_substruct_match, get_substruct_matches,
    get_svg, get_v3kmolblock,
};
use bindings::{
    get_morgan_fp, get_morgan_fp_as_bytes, get_pattern_fp, get_pattern_fp_as_bytes, get_rdkit_fp,
    get_rdkit_fp_as_bytes,
};

/// Basic class, implementing most functionality as member functions of a molecule object

pub struct Molecule {
    pkl_mol: *mut c_char,   // Pointer to the molecule data in C format
    pkl_size: *mut usize,   // Pointer to size of molecule data
}

impl Drop for Molecule {
    fn drop(&mut self) {
        self.free_memory();
    }
}


impl std::fmt::Debug for Molecule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let smiles = self.get_smiles("");
        write!(f, "{}", smiles)
    }
}

impl Molecule {
    /// Constructor returning an optional molecule
    pub fn new(input: &str, json_info: &str) -> Option<Molecule> {
        unsafe {
            // Convert Rust strings to C strings, handling NULL bytes
            let input_cstr = match CString::new(input) {
                Ok(s) => s,
                Err(_) => return None,  // Return None if string contains NULL bytes
            };
            let json_cstr = match CString::new(json_info) {
                Ok(s) => s,
                Err(_) => return None,
            };

            // Allocate memory for size with proper alignment
            let pkl_size = libc::malloc(mem::size_of::<usize>()) as *mut usize;
            if pkl_size.is_null() {
                return None;  // Return None if allocation fails
            }

            // Get molecule pointer from RDKit
            let pkl_mol = get_mol(
                input_cstr.as_ptr(),    // Convert CString to raw pointer
                pkl_size,               // Pass pointer to size
                json_cstr.as_ptr(),     // Convert CString to raw pointer
            );
            
            // Comprehensive error checking
            if pkl_mol.is_null() || unsafe { *pkl_size == 0 } {
                // Clean up allocated memory if molecule creation fails
                if !pkl_size.is_null() {
                    free(pkl_size as *mut c_void);
                }
                return None;
            }

            Some(Molecule { pkl_mol, pkl_size })
        }
    }

    /// Constructor returning Molecule, panics if None
    pub fn get_mol(input: &str, json_info: &str) -> Molecule {
        let input_cstr = CString::new(input).unwrap();
        let json_info = CString::new(json_info).unwrap();
        let pkl_size: *mut usize = unsafe { libc::malloc(mem::size_of::<u64>()) as *mut usize };
        let pkl_mol = unsafe { get_mol(input_cstr.as_ptr(), pkl_size, json_info.as_ptr()) };
        if pkl_mol.is_null() {
            panic!("Could not create molecule!");
        }
        Molecule { pkl_size, pkl_mol }
    }

    /// Gets a commonchem representation as JSON string
    pub fn get_json(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let rdkit_json_cchar = get_json(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let mol_json_str = CStr::from_ptr(rdkit_json_cchar)
                .to_string_lossy()
                .into_owned();
            free_ptr(rdkit_json_cchar);
            mol_json_str
        }
    }

    /// Gets a fully typed common chem like json object
    pub fn get_commonchem(&self) -> JsonBase {
        let json_repr = self.get_json("");
        let res: JsonBase = serde_json::from_str(&json_repr).expect("Wrong JSON format!?");
        res
    }

    /// Gets the underlying Molecule object of the common chem structure
    pub fn get_json_molecule(&self) -> JsonMolecule {
        let json_repr = self.get_json("");
        JsonMolecule::json_mol_from_json(&json_repr)
    }

    pub fn get_atoms(&self) -> Vec<JsonAtom> {
        let json_mol = self.get_json_molecule();
        json_mol.atoms
    }

    pub fn get_numatoms(&self) -> usize {
        let json_mol = self.get_json_molecule();
        json_mol.atoms.len()
    }

    pub fn get_bonds(&self) -> Vec<JsonBond> {
        let json_mol = self.get_json_molecule();
        json_mol.bonds
    }

    pub fn get_numbonds(&self) -> usize {
        let json_mol = self.get_json_molecule();
        json_mol.bonds.len()
    }

    /// Get a 2 dimensional vector with atomic coordinates
    pub fn get_coords(&self) -> Vec<Vec<f32>> {
        let json_mol = self.get_json_molecule();
        let conf: &JsonConformer = json_mol.conformers.get(0).unwrap();
        conf.coords.to_owned()
    }

    /// Get the SMILES string from a molecule
    pub fn get_smiles(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let a: *mut c_char = get_smiles(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let can_smiles: String = CStr::from_ptr(a).to_string_lossy().into_owned();
            free_ptr(a);
            can_smiles
        }
    }

    /// get SMARTS
    pub fn get_smarts(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let a: *mut c_char = get_smarts(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let smarts: String = CStr::from_ptr(a).to_string_lossy().into_owned();
            free_ptr(a);
            smarts
        }
    }

    /// get CXSMILES
    pub fn get_cxsmiles(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let a: *mut c_char = get_cxsmiles(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let cxsmiles: String = CStr::from_ptr(a).to_string_lossy().into_owned();
            free_ptr(a);
            cxsmiles
        }
    }

    /// find a substructure match via query molecule
    pub fn get_substruct_match(&self, query: &Molecule, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let a: *mut c_char = get_substruct_match(
                self.pkl_mol,
                *self.pkl_size,
                query.pkl_mol,
                *query.pkl_size,
                json_info.as_ptr(),
            );
            let res: String = CStr::from_ptr(a).to_string_lossy().into_owned();
            free_ptr(a);
            res
        }
    }

    /// find several substructure matches via query molecule
    pub fn get_substruct_matches(&self, query: &Molecule, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let a: *mut c_char = get_substruct_matches(
                self.pkl_mol,
                *self.pkl_size,
                query.pkl_mol,
                *query.pkl_size,
                json_info.as_ptr(),
            );
            let res: String = CStr::from_ptr(a).to_string_lossy().into_owned();
            free_ptr(a);
            res
        }
    }

    /// get svg image
    pub fn get_svg(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let a: *mut c_char = get_svg(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let svg: String = CStr::from_ptr(a).to_string_lossy().into_owned();
            free_ptr(a);
            svg
        }
    }

    /// Normalize  molecule
    pub fn normalize(&mut self, json_info: &str) {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            normalize(
                &mut self.pkl_mol as *mut _,
                self.pkl_size,
                json_info.as_ptr(),
            );
        }
    }

    /// Neutralize charged species
    pub fn neutralize(&mut self, json_info: &str) {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            neutralize(
                &mut self.pkl_mol as *mut _,
                self.pkl_size,
                json_info.as_ptr(),
            );
        }
    }

    /// celanup molecule
    pub fn cleanup(&mut self, json_info: &str) {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            cleanup(
                &mut self.pkl_mol as *mut _,
                self.pkl_size,
                json_info.as_ptr(),
            );
        }
    }

    /// reionize molecule
    pub fn reionize(&mut self, json_info: &str) {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            reionize(
                &mut self.pkl_mol as *mut _,
                self.pkl_size,
                json_info.as_ptr(),
            );
        }
    }

    /// get a the canonical tautomer
    pub fn canonical_tautomer(&mut self, json_info: &str) {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            canonical_tautomer(
                &mut self.pkl_mol as *mut _,
                self.pkl_size,
                json_info.as_ptr(),
            );
        }
    }

    /// gets the larger fragment
    pub fn fragment_parent(&mut self, json_info: &str) {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            fragment_parent(
                &mut self.pkl_mol as *mut _,
                self.pkl_size,
                json_info.as_ptr(),
            );
        }
    }

    /// gets the charge fragment
    pub fn charge_parent(&mut self, json_info: &str) {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            charge_parent(
                &mut self.pkl_mol as *mut _,
                self.pkl_size,
                json_info.as_ptr(),
            );
        }
    }

    /// get the inchi as a String
    pub fn get_inchi(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let inchi_cchar: *mut c_char =
                get_inchi(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let inchi: String = CStr::from_ptr(inchi_cchar).to_string_lossy().into_owned();
            free_ptr(inchi_cchar);
            inchi
        }
    }

    /// get the inchi key
    pub fn get_inchikey(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let a: *mut c_char = get_inchi(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let b: *mut c_char = get_inchikey_for_inchi(a);
            let inchikey: String = CStr::from_ptr(b).to_string_lossy().into_owned();
            free_ptr(a);
            free_ptr(b);
            inchikey
        }
    }

    /// add the hydrogens
    pub fn add_hs(&mut self) {
        unsafe {
            add_hs(&mut self.pkl_mol as *mut _, self.pkl_size);
        }
    }

    /// remove hydrogens
    pub fn remove_all_hs(&mut self) {
        unsafe {
            remove_all_hs(&mut self.pkl_mol as *mut _, self.pkl_size);
        }
    }

    /// creates 3D coordinates
    pub fn set_3d_coords(&mut self, json_info: &str) {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            set_3d_coords(
                &mut self.pkl_mol as *mut _,
                self.pkl_size,
                json_info.as_ptr(),
            );
        }
    }
    /// Gets a [MDL molfile](https://en.wikipedia.org/wiki/Chemical_table_file) content as a string.
    pub fn get_molblock(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let mblock_cchar: *mut c_char =
                get_molblock(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let res = CStr::from_ptr(mblock_cchar).to_string_lossy().into_owned();
            free_ptr(mblock_cchar);
            res
        }
    }

    /// Gets a v3000 MDL molblock content as a string.
    pub fn get_v3kmolblock(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let mblock_cchar: *mut c_char =
                get_v3kmolblock(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let res = CStr::from_ptr(mblock_cchar).to_string_lossy().into_owned();
            free_ptr(mblock_cchar);
            res
        }
    }

    /// get descriptors as hashmap
    pub fn get_descriptors_as_dict(&self) -> HashMap<String, f32> {
        let desc_string = self.get_descriptors();
        let desc_json: HashMap<String, f32> =
            serde_json::from_str(&desc_string).expect("Wrong JSON format!?");
        desc_json
    }

    /// get descriptors as string
    pub fn get_descriptors(&self) -> String {
        let desc_cchar: *mut c_char = unsafe { get_descriptors(self.pkl_mol, *self.pkl_size) };
        let desc_string: &str = unsafe { CStr::from_ptr(desc_cchar).to_str().unwrap() };
        let res = desc_string.to_owned();
        unsafe { free_ptr(desc_cchar) };
        res
    }

    pub fn get_morgan_fp(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let fp_cchar: *mut c_char =
                get_morgan_fp(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let fp_string: &str = CStr::from_ptr(fp_cchar).to_str().unwrap();
            let res = fp_string.to_owned();
            free_ptr(fp_cchar);
            res
        }
    }

    pub fn get_morgan_fp_as_bytes(&self, json_info: &str) -> Vec<i8> {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let n_bytes: *mut usize = libc::malloc(mem::size_of::<u64>()) as *mut usize;
            let fp_cchar: *mut c_char =
                get_morgan_fp_as_bytes(self.pkl_mol, *self.pkl_size, n_bytes, json_info.as_ptr());
            let mut fp_bytes: Vec<i8> = Vec::new();
            for pos in 0..*n_bytes {
                let nb: i8 = *fp_cchar.offset(pos as _);
                fp_bytes.push(nb);
            }
            let res = fp_bytes.to_owned();
            free(n_bytes as *mut c_void);
            free_ptr(fp_cchar);
            res
        }
    }

    pub fn get_rdkit_fp(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let fp_cchar: *mut c_char =
                get_rdkit_fp(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let fp_string: &str = CStr::from_ptr(fp_cchar).to_str().unwrap();
            let res = fp_string.to_owned();
            free_ptr(fp_cchar);
            res
        }
    }

    pub fn get_rdkit_fp_as_bytes(&self, json_info: &str) -> Vec<i8> {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let n_bytes: *mut usize = libc::malloc(mem::size_of::<u64>()) as *mut usize;
            let fp_cchar: *mut c_char =
                get_rdkit_fp_as_bytes(self.pkl_mol, *self.pkl_size, n_bytes, json_info.as_ptr());
            let mut fp_bytes: Vec<i8> = Vec::new();
            for pos in 0..*n_bytes {
                let nb: i8 = *fp_cchar.offset(pos as _);
                fp_bytes.push(nb);
            }
            let res = fp_bytes.to_owned();
            free(n_bytes as *mut c_void);
            free_ptr(fp_cchar);
            res
        }
    }

    pub fn get_pattern_fp(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let fp_cchar: *mut c_char =
                get_pattern_fp(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let fp_string: &str = CStr::from_ptr(fp_cchar).to_str().unwrap();
            let res = fp_string.to_owned();
            free_ptr(fp_cchar);
            res
        }
    }

    pub fn get_pattern_fp_as_bytes(&self, json_info: &str) -> Vec<i8> {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let n_bytes: *mut usize = libc::malloc(mem::size_of::<u64>()) as *mut usize;
            let fp_cchar: *mut c_char =
                get_pattern_fp_as_bytes(self.pkl_mol, *self.pkl_size, n_bytes, json_info.as_ptr());
            let mut fp_bytes: Vec<i8> = Vec::new();
            for pos in 0..*n_bytes {
                let nb: i8 = *fp_cchar.offset(pos as _);
                fp_bytes.push(nb);
            }
            let res = fp_bytes.to_owned();
            free(n_bytes as *mut c_void);
            free_ptr(fp_cchar);
            res
        }
    }

    ///Gets a query molecule from a SMARTS
    pub fn get_qmol(input: &str, json_info: &str) -> Option<Molecule> {
        let input_cstr = CString::new(input).unwrap();
        let json_info = CString::new(json_info).unwrap();
        let pkl_size: *mut usize = unsafe { libc::malloc(mem::size_of::<u64>()) as *mut usize };
        let pkl_mol = unsafe { get_qmol(input_cstr.as_ptr(), pkl_size, json_info.as_ptr()) };
        if pkl_mol.is_null() {
            return None;
        }
        return Some(Molecule { pkl_size, pkl_mol });
    }

    fn free_memory(&mut self) {
        unsafe {
            // Free molecule data if pointer is not null
            if !self.pkl_mol.is_null() {
                free_ptr(self.pkl_mol);             // Use RDKit's free function
                self.pkl_mol = std::ptr::null_mut(); // Null the pointer to prevent use-after-free
            }
            // Free size data if pointer is not null
            if !self.pkl_size.is_null() {
                free(self.pkl_size as *mut c_void);  // Use libc free
                self.pkl_size = std::ptr::null_mut(); // Null the pointer
            }
        }
    }
}

/// read a classical .smi file
pub fn read_smifile(smi_file: &str) -> Vec<Option<Molecule>> {
    let smi_file = read_to_string(smi_file).expect("Could not load file.");
    let mut mol_list: Vec<Option<Molecule>> = Vec::new();
    let smiles_list: Vec<&str> = smi_file.split("\n").collect();
    for s in smiles_list.iter() {
        let s_mod = s.trim();
        if s_mod.len() == 0 {
            mol_list.push(None);
            continue;
        };
        let mol_opt = Molecule::new(s_mod, "");
        mol_list.push(mol_opt);
    }
    mol_list
}

/// read a classical .smi file, filter molecules which are none
pub fn read_smifile_unwrap(smi_file: &str) -> Vec<Molecule> {
    let mut mol_opt_list: Vec<Option<Molecule>> = crate::read_smifile(smi_file);
    let mol_list: Vec<Molecule> = mol_opt_list.into_iter().filter_map(|m| m).collect();
    mol_list
}

/// read a classical .sdf file
pub fn read_sdfile(sd_file: &str) -> Vec<Option<Molecule>> {
    let sd_file = read_to_string(sd_file).expect("Could not load file.");
    let mut mol_list: Vec<Option<Molecule>> = Vec::new();
    let molblock_list: Vec<&str> = sd_file.split("$$$$").collect();
    for (i, s) in molblock_list.iter().enumerate() {
        let s_mod = s.trim();
        if s_mod.len() == 0 {
            mol_list.push(None);
            continue;
        };
        let mut mol_opt = Molecule::new(s_mod, "");

        // this avoids hard to catch exceptions later on...
        //match mol_opt.as_mut() {
        //    Some(mut mol_opt) => {mol_opt.cleanup(""); Some(mol_opt)},
        //    None => None,
        //};
        mol_list.push(mol_opt);
    }
    mol_list
}

/// read a classical .sdf file, filter molecules which are none
pub fn read_sdfile_unwrap(sd_file: &str) -> Vec<Molecule> {
    let mut mol_opt_list: Vec<Option<Molecule>> = crate::read_sdfile(sd_file);
    let mol_list: Vec<Molecule> = mol_opt_list.into_iter().filter_map(|m| m).collect();
    mol_list
}

pub struct SDIterator {
    molblock_iterator: std::vec::IntoIter<String>,
}

impl SDIterator {
    pub fn new(pathname: &str) -> Self {
        let sd_file = read_to_string(pathname).expect("Could not load file.");
        let molblock_list: Vec<String> = sd_file.split("$$$$").map(|x| x.to_string()).collect();
        let molblock_iterator = molblock_list.into_iter();
        SDIterator { molblock_iterator }
    }
}

impl Iterator for SDIterator {
    type Item = Option<Molecule>;
    fn next(&mut self) -> Option<Self::Item> {
        let molblock = self.molblock_iterator.next();
        let s_mod = match molblock {
            Some(molblock) => molblock.trim().to_string(),
            None => return None,
        };
        if s_mod.len() == 0 {
            self.next(); //last line
        };
        let mol_opt = Molecule::new(&s_mod, "");
        Some(mol_opt)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonBase {
    pub rdkitjson: VersionInfo,
    pub defaults: RdkitDefaults,
    pub molecules: Vec<JsonMolecule>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionInfo {
    pub version: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RdkitDefaults {
    pub atom: AtomDefaults,
    pub bond: BondDefaults,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AtomDefaults {
    pub z: i32,
    pub impHs: i32,
    pub chg: i32,
    pub nRad: i32,
    pub isotope: i32,
    pub stereo: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BondDefaults {
    pub bo: i32,
    pub stereo: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonMolecule {
    #[serde(default)]
    pub name: String,
    pub atoms: Vec<JsonAtom>,
    pub bonds: Vec<JsonBond>,
    #[serde(default)]
    pub conformers: Vec<JsonConformer>,
    pub extensions: Vec<Extensions>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonConformer {
    pub coords: Vec<Vec<f32>>,
    dim: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Extensions {
    name: String,
    formatVersion: i32,
    toolkitVersion: String,
    #[serde(default)]
    aromaticAtoms: Vec<i32>,
    #[serde(default)]
    aromaticBonds: Vec<i32>,
    #[serde(default)]
    atomRings: Vec<Vec<i32>>,
    #[serde(default)]
    cipCodes: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonAtom {
    #[serde(default)]
    chg: i32,
    #[serde(default)]
    impHs: i32,
    #[serde(default)]
    isotope: i32,
    #[serde(default)]
    nRad: i32,
    #[serde(default)]
    stereo: String,
    #[serde(default = "z_default")]
    z: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonBond {
    #[serde(default)]
    atoms: Vec<i32>,
    #[serde(default)]
    bo: i32,
    #[serde(default = "stereo_default")]
    stereo: String,
}

const Z_DEFAULT: i32 = 6;
fn z_default() -> i32 {
    Z_DEFAULT
}

fn stereo_default() -> String {
    String::from("unspecified")
}

impl JsonMolecule {
    ///Create new molecule from smiles, SD file or json
    pub fn new(self, molstring: &str) -> JsonMolecule {
        JsonMolecule::json_mol_from_string(molstring, "")
    }

    pub fn json_mol_from_string(molstring: &str, json_info: &str) -> JsonMolecule {
        let json_str = jsonfrom_string(molstring, json_info);
        JsonMolecule::json_mol_from_json(&json_str)
    }

    pub fn json_mol_from_smiles(smiles: &str) -> JsonMolecule {
        JsonMolecule::json_mol_from_string(smiles, "")
    }

    pub fn json_mol_from_json(json_str: &str) -> JsonMolecule {
        let rdkit_json: JsonBase = serde_json::from_str(&json_str).expect("Wrong JSON format!?");
        let mol = serde_json::to_string(&rdkit_json.molecules[0]).unwrap();
        serde_json::from_str(&mol).expect("Wrong JSON format!?")
    }
}

pub fn jsonfrom_string(input: &str, json_info: &str) -> String {
    //let (pkl_mol, pkl_size) = Molecule::PklFromString(input, json_info);
    let pkl_mol = Molecule::new(input, json_info).unwrap();
    let mol_json_str = pkl_mol.get_json(json_info);
    mol_json_str.to_owned()
}
