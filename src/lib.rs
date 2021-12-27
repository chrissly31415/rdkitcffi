//! This is an experimental (and not official) rust wrapper for some functionaltiy of the great [RDKit](https://www.rdkit.org/) library.
//!
//! It makes use of its new (and also still experimental) C Foreign Function Interface (CFFI), see also this [blog post](https://greglandrum.github.io/rdkit-blog/technical/2021/05/01/rdkit-cffi-part1.html).
//!
//! Use it at your own risk, its not recommended yet for productive use and only available for linux :-)
//! 
//! Please note, that only a limited functionality is being exposed via cffi by RDKit. Structured data is
//! transferred from the backend via the cffi interface as string types.
//! This also means that the structure of objects is significantly different from the C/C++ and python APIs.
//!
//! Please have a look at the examples below and the test functions.  
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! use rdkitcffi::Molecule;
//!
//! let orig_smiles = "OCCC#CO";
//! let pkl_mol = Molecule::new(orig_smiles, "");
//! let desc = pkl_mol.get_descriptors();
//! ```
//!
//! Working with SD files:
//!
//! ```
//! use rdkitcffi::Molecule;
//!
//! let mut mol_list : Vec<Molecule> = Molecule::read_sdfile("examples/test.sdf");
//! mol_list.iter_mut().for_each(|m| m.remove_all_hs());
//!
//! ```
//!
//! Getting a JSON version of the molecule (via serde_json):
//!
//! ```
//! use rdkitcffi::Molecule;
//!
//! let orig_smiles = "OCCC#CO";
//! let pkl_mol = Molecule::new(orig_smiles, "");
//! println!("json molecule:    {:?}", pkl_mol.get_JsonMolecule(""));
//!
//! ```
//!
//! Neutralizing a zwitterion
//!
//! ```
//! use rdkitcffi::Molecule;
//!
//! let orig_smiles = "C(C(=O)[O-])[NH3+]";
//! let mut pkl_mol = Molecule::new(orig_smiles, "");
//! pkl_mol.neutralize("");
//!
//!
//! ```
//!
//! Computing RDKit descriptors
//! 
//! ```
//! use rdkitcffi::Molecule;
//! 
//! let orig_smiles = "CCCN";
//! let pkl_mol = Molecule::new(orig_smiles, "");
//! let desc = pkl_mol.get_descriptors();
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
//! let mut mol_list : Vec<Molecule> = Molecule::read_sdfile("examples/test.sdf");
//!
//! let a: Vec<_> = mol_list.iter().map(|m| m.get_smiles("")).collect();
//!
//! let df = df!( "smiles" => a).unwrap();
//!
//! ```
//!
//!
//!
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use std::collections::HashMap;

use std::ffi::{CStr, CString};
use std::fs::read_to_string;
use std::mem;
use std::os::raw::c_char;
use std::os::raw::c_void;

mod bindings;

use crate::bindings::{add_hs, remove_all_hs, set_3d_coords};
use crate::bindings::{canonical_tautomer, cleanup, neutralize, normalize, reionize};
use crate::bindings::{free, free_ptr, size_t};
use crate::bindings::{
    get_descriptors, get_inchi, get_inchikey_for_inchi, get_json, get_mol, get_molblock,
    get_morgan_fp, get_smiles,
};

/// Basic class, implementing most functionality as member functions of a molecule object
pub struct Molecule {
    pkl_size: *mut size_t,
    pkl_mol: *mut i8,
}

impl Molecule {
    pub fn new(input: &str, json_info: &str) -> Molecule {
        unsafe {
            let input_cstr = CString::new(input).unwrap();
            let json_info = CString::new(json_info).unwrap();

            let pkl_size: *mut size_t = libc::malloc(mem::size_of::<u64>()) as *mut u64;
            let pkl_mol = get_mol(input_cstr.as_ptr(), pkl_size, json_info.as_ptr());
            Molecule { pkl_size, pkl_mol }
        }
    }

    pub fn get_atoms(&self) -> Vec<JsonAtom> {
        let json_mol = self.get_JsonMolecule("");
        json_mol.atoms
    }

    pub fn get_numatoms(&self) -> usize {
        let json_mol = self.get_JsonMolecule("");
        json_mol.atoms.len()
    }

    pub fn get_bonds(&self) -> Vec<JsonBond> {
        let json_mol = self.get_JsonMolecule("");
        json_mol.bonds
    }

    pub fn get_numbonds(&self) -> usize {
        let json_mol = self.get_JsonMolecule("");
        json_mol.bonds.len()
    }

    // Get a 2 dimensiona vector with atomic coordinates
    pub fn get_coords(&self) -> Vec<Vec<f32>> {
        let json_mol = self.get_JsonMolecule("");
        let conf: &JsonConformer = json_mol.conformers.get(0).unwrap().clone();
        conf.coords.to_owned()
    }

    /// Get the SMILES string from a molecule
    pub fn get_smiles(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let can_smiles_cchar: *mut c_char =
                get_smiles(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let can_smiles: &str = CStr::from_ptr(can_smiles_cchar).to_str().unwrap();
            can_smiles.to_owned()
        }
    }

    /// Normalize the topology of a molecule
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

    pub fn get_inchi(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let inchi_cchar: *mut c_char =
                get_inchi(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let inchi: &str = CStr::from_ptr(inchi_cchar).to_str().unwrap();
            inchi.to_owned()
        }
    }

    pub fn get_inchikey(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let mut inchi_cchar: *mut c_char =
                get_inchi(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            inchi_cchar = get_inchikey_for_inchi(inchi_cchar);
            let inchikey: &str = CStr::from_ptr(inchi_cchar).to_str().unwrap();
            inchikey.to_owned()
        }
    }

    pub fn add_hs(&mut self) {
        unsafe {
            add_hs(&mut self.pkl_mol as *mut _, self.pkl_size);
        }
    }

    pub fn remove_all_hs(&mut self) {
        unsafe {
            remove_all_hs(&mut self.pkl_mol as *mut _, self.pkl_size);
        }
    }

    /// Gets a representation as JSON string
    pub fn get_json(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let rdkit_json_cchar = get_json(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let mol_json_str = CStr::from_ptr(rdkit_json_cchar).to_str().unwrap();
            mol_json_str.to_owned()
        }
    }

    /// Gets a representation as a JSON Molecule object
    pub fn get_JsonMolecule(&self, json_info: &str) -> JsonMolecule {
        let json_info = self.get_json("");
        JsonMolecule::JsonMolFromJson(&json_info)
    }

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
    /// Gets a MDL molfile content as a string, see also: https://en.wikipedia.org/wiki/Chemical_table_file
    pub fn get_molblock(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let mblock_cchar: *mut c_char =
                get_molblock(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let mblock: &str = CStr::from_ptr(mblock_cchar).to_str().unwrap();
            mblock.to_owned()
        }
    }

    pub fn get_descriptors(&self) -> HashMap<String, f32> {
        let desc_string = self.get_descriptors_as_string();
        let desc_json: HashMap<String, f32> =
            serde_json::from_str(&desc_string).expect("Wrong JSON format!?");
        desc_json
    }

    pub fn get_descriptors_as_string(&self) -> String {
        unsafe {
            let desc_cchar: *mut c_char = get_descriptors(self.pkl_mol, *self.pkl_size);
            let desc_string: &str = CStr::from_ptr(desc_cchar).to_str().unwrap();
            desc_string.to_owned()
        }
    }

    pub fn get_morgan_fp_as_string(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let fp_cchar: *mut c_char =
                get_morgan_fp(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let fp_string: &str = CStr::from_ptr(fp_cchar).to_str().unwrap();
            fp_string.to_owned()
        }
    }

    /// Gets a molecule representation from a string
    pub fn PklFromString(input: &str, json_info: &str) -> (*mut i8, *mut size_t) {
        let input_cstr = CString::new(input).unwrap();
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let pkl_size: *mut size_t = libc::malloc(mem::size_of::<u64>()) as *mut u64;
            let pkl_mol = get_mol(input_cstr.as_ptr(), pkl_size, json_info.as_ptr());
            (pkl_mol, pkl_size)
        }
    }

    /// read a classical .smi file
    pub fn read_smifile(smi_file: &str) -> Vec<Molecule> {
        let smi_file = read_to_string(smi_file).expect("Could not load file.");
        let mut mol_list: Vec<Molecule> = Vec::new();
        let smiles_list: Vec<&str> = smi_file.split("\n").collect();
        for (i, s) in smiles_list.iter().enumerate() {
            if s.len() == 0 {
                continue;
            };
            let s_mod = s.trim_start_matches("\n");
            let mol: Molecule = Molecule::new(s_mod, "");
            unsafe {
                if (*mol.pkl_size == 0) {
                    eprintln!("Skipping position: {} - cannot create molecule. ", i);
                    continue;
                }
            }
            mol_list.push(mol);
        }
        mol_list
    }

    /// read a classical .sdf file
    pub fn read_sdfile(sd_file: &str) -> Vec<Molecule> {
        let sd_file = read_to_string(sd_file).expect("Could not load file.");
        let mut mol_list: Vec<Molecule> = Vec::new();
        let molblock_list: Vec<&str> = sd_file.split("$$$$").collect();
        for (i, s) in molblock_list.iter().enumerate() {
            let s_mod = s.trim_start_matches("\n");
            if s.len() <= 1 {
                continue;
            };
            let mut mol: Molecule = Molecule::new(s_mod, "");
            unsafe {
                if (*mol.pkl_size == 0) {
                    eprintln!("Skipping position: {} - cannot create molecule. ", i);
                    continue;
                }
            }
            mol_list.push(mol);
        }
        mol_list
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonBase {
    pub commonchem: HashMap<String, i32>,
    pub molecules: Vec<JsonMolecule>,
    pub defaults: RdkitDefaults,
}

/// This implements the commom chem json structure:
/// see also: [https://github.com/CommonChem/CommonChem](https://github.com/CommonChem/CommonChem)
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
pub struct RdkitDefaults {
    atom: JsonAtom,
    bond: JsonBond,
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
        JsonMolecule::JsonMolFromString(molstring, "")
    }

    pub fn JsonMolFromString(molstring: &str, json_info: &str) -> JsonMolecule {
        let json_str = JSONFromString(molstring, json_info);
        JsonMolecule::JsonMolFromJson(&json_str)
    }

    pub fn JsonMolFromSmiles(smiles: &str, json_info: &str) -> JsonMolecule {
        JsonMolecule::JsonMolFromString(smiles, "")
    }

    pub fn JsonMolFromJson(json_str: &str) -> JsonMolecule {
        let rdkit_json: JsonBase = serde_json::from_str(&json_str).expect("Wrong JSON format!?");
        let mol = serde_json::to_string(&rdkit_json.molecules[0]).unwrap();
        serde_json::from_str(&mol).expect("Wrong JSON format!?")
    }
}

pub fn JSONFromString(input: &str, json_info: &str) -> String {
    let (pkl_mol, pkl_size) = Molecule::PklFromString(input, json_info);
    let json_info = CString::new(json_info).unwrap();
    let mut mol_json_str = "";
    unsafe {
        let rdkit_json_cchar = get_json(pkl_mol, *pkl_size, json_info.as_ptr());
        mol_json_str = CStr::from_ptr(rdkit_json_cchar).to_str().unwrap();

        free(pkl_size as *mut c_void);
        free_ptr(pkl_mol);
    }
    mol_json_str.to_owned()
}

#[cfg(test)]
mod examples;
mod tests {
    use std::borrow::Borrow;

    use super::*;
    #[test]
    fn smiles2descriptors() {
        let orig_smiles = "CCCN";
        let pkl_mol = Molecule::new(orig_smiles, "");
        let desc = pkl_mol.get_descriptors();
        println!("Descriptors: {:?}", desc);
        let nheavy = desc.get("NumHeavyAtoms").unwrap().round() as i32;
        assert_eq!(nheavy, 4);
        let nrot = desc.get("NumRotatableBonds").unwrap().round() as i32;
        assert_eq!(nrot, 1);
    }

    #[test]
    fn smifile2molecules() {
        let mut mol_list: Vec<Molecule> = Molecule::read_smifile("examples/ringtest.smi");
        for (i, mol) in mol_list.iter_mut().enumerate() {
            mol.remove_all_hs();
            println!(
                "Pos:{} INCHIKEY: {} SMILES: {} ",
                i,
                mol.get_inchikey(""),
                mol.get_smiles("")
            )
        }
        assert_eq!(mol_list.len(), 11);
    }
    #[test]
    fn sdfile2molecules() {
        let mut mol_list: Vec<Molecule> = Molecule::read_sdfile("examples/test.sdf");
        for (i, mol) in mol_list.iter_mut().enumerate() {
            mol.remove_all_hs();
            println!(
                "Pos:{} INCHIKEY: {} SMILES: {} NUMATOMS: {} NUMBONDS: {}",
                i,
                mol.get_inchikey(""),
                mol.get_smiles(""),
                mol.get_numatoms(),
                mol.get_numbonds(),
            )
        }
        assert_eq!(mol_list.len(), 990);
    }
    #[test]
    fn morgan_fp() {
        let smiles = "OCC=CCO";
        let options = "{\"radius\":2,\"nBits\":32}";
        let pkl_mol = Molecule::new(smiles, "");
        let fps = pkl_mol.get_morgan_fp_as_string(options);
        println!("Fingerprints: {:?}", fps);
        assert_eq!(fps, "00000001000110001000001000010011");
    }

    #[test]
    fn generate3d() {
        let orig_smiles = "CC";
        let mut pkl_mol = Molecule::new(orig_smiles, "");
        pkl_mol.set_3d_coords("");
        let coords = pkl_mol.get_coords();
        assert_eq!(coords.len(), 2);
        assert_eq!(coords[0].len(), 3);
    }
    #[test]
    fn smiles_from_smiles_via_pkl() {
        let orig_smiles = "OCCC#CO";
        let pkl_mol = Molecule::new(orig_smiles, "");
        println!(
            "SMILES: {} Canonical SMILES: {}",
            orig_smiles,
            pkl_mol.get_smiles("")
        );
        assert_eq!(pkl_mol.get_smiles(""), "OC#CCCO");
    }
    #[test]
    fn inchi_from_smiles_via_pkl() {
        let orig_smiles = "OCCC#CO";
        let pkl_mol = Molecule::new(orig_smiles, "");
        println!("inchi:    {}", pkl_mol.get_inchi(""));
        println!("inchikey: {}", pkl_mol.get_inchikey(""));
        assert_eq!(
            pkl_mol.get_inchi(""),
            "InChI=1S/C4H6O2/c5-3-1-2-4-6/h5-6H,1,3H2"
        );
        assert_eq!(pkl_mol.get_inchikey(""), "JSPXPZKDILSYNN-UHFFFAOYSA-N");
    }
    #[test]
    fn molblock_from_smiles_via_pkl() {
        let orig_smiles = "CCO";
        let pkl_mol = Molecule::new(orig_smiles, "");
        println!("molblock:*{}*", pkl_mol.get_molblock(""));
        assert_eq!(pkl_mol.get_molblock(""),"\n     RDKit          2D\n\n  3  2  0  0  0  0  0  0  0  0999 V2000\n    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    1.2990    0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    2.5981   -0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0\n  1  2  1  0\n  2  3  1  0\nM  END\n");
    }

    #[test]
    fn get_json_via_pkl() {
        let orig_smiles = "OCO";
        let pkl_mol = Molecule::new(orig_smiles, "");
        println!("json:    {}", pkl_mol.get_json(""));
        assert_eq!(pkl_mol.get_json(""),"{\"commonchem\":{\"version\":10},\"defaults\":{\"atom\":{\"z\":6,\"impHs\":0,\"chg\":0,\"nRad\":0,\"isotope\":0,\"stereo\":\"unspecified\"},\"bond\":{\"bo\":1,\"stereo\":\"unspecified\"}},\"molecules\":[{\"atoms\":[{\"z\":8,\"impHs\":1},{\"impHs\":2},{\"z\":8,\"impHs\":1}],\"bonds\":[{\"atoms\":[0,1]},{\"atoms\":[1,2]}],\"extensions\":[{\"name\":\"rdkitRepresentation\",\"formatVersion\":2,\"toolkitVersion\":\"2021.09.11\"}]}]}");
    }
    #[test]
    fn get_json_molecule() {
        let orig_smiles = "C#C";
        let pkl_mol = Molecule::new(orig_smiles, "");
        let json_mol = pkl_mol.get_JsonMolecule("");
        println!("json molecule:    {:?}", json_mol);
        let atoms = json_mol.atoms;
        assert_eq!(atoms.len(), 2);
    }
    #[test]
    fn jsonmolecule_from_smiles() {
        let json_mol = JsonMolecule::JsonMolFromSmiles("CC(C)CCCO", "");
        println!("{:?}", json_mol);
        let bonds = json_mol.bonds;
        assert_eq!(bonds.len(), 6);
        //println!("{:?}", mol.bonds);
    }
    #[test]
    fn json_str_from_smiles() {
        let json_str = JSONFromString("CCCI", "");
        println!("JSON:{}", json_str);
        //let json_mol = json_mol_str.;
        assert_eq!(json_str,"{\"commonchem\":{\"version\":10},\"defaults\":{\"atom\":{\"z\":6,\"impHs\":0,\"chg\":0,\"nRad\":0,\"isotope\":0,\"stereo\":\"unspecified\"},\"bond\":{\"bo\":1,\"stereo\":\"unspecified\"}},\"molecules\":[{\"atoms\":[{\"impHs\":3},{\"impHs\":2},{\"impHs\":2},{\"z\":53}],\"bonds\":[{\"atoms\":[0,1]},{\"atoms\":[1,2]},{\"atoms\":[2,3]}],\"extensions\":[{\"name\":\"rdkitRepresentation\",\"formatVersion\":2,\"toolkitVersion\":\"2021.09.11\"}]}]}");
    }
    #[test]
    fn json_str_from_sdf() {
        let json_str = JSONFromString("\n     RDKit          2D\n\n  7  6  0  0  0  0  0  0  0  0999 V2000\n    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    1.2990    0.7500    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0\n    2.5981   -0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    3.8971    0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    5.1962   -0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    6.4952    0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    7.7942    1.5000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n  1  2  1  0\n  2  3  1  0\n  3  4  1  0\n  4  5  2  3\n  5  6  1  0\n  6  7  3  0\nM  END\n", "");
        println!("JSON:{}", json_str);
        assert_eq!(json_str,"{\"commonchem\":{\"version\":10},\"defaults\":{\"atom\":{\"z\":6,\"impHs\":0,\"chg\":0,\"nRad\":0,\"isotope\":0,\"stereo\":\"unspecified\"},\"bond\":{\"bo\":1,\"stereo\":\"unspecified\"}},\"molecules\":[{\"name\":\"\",\"atoms\":[{\"impHs\":3},{\"z\":8},{\"impHs\":2},{\"impHs\":1},{\"impHs\":1},{},{\"impHs\":1}],\"bonds\":[{\"atoms\":[0,1]},{\"atoms\":[1,2]},{\"atoms\":[2,3]},{\"bo\":2,\"atoms\":[3,4],\"stereo\":\"either\"},{\"atoms\":[4,5]},{\"bo\":3,\"atoms\":[5,6]}],\"conformers\":[{\"dim\":2,\"coords\":[[0.0,0.0],[1.299,0.75],[2.5981,-0.0],[3.8971,0.75],[5.1962,-0.0],[6.4952,0.75],[7.7942,1.5]]}],\"extensions\":[{\"name\":\"rdkitRepresentation\",\"formatVersion\":2,\"toolkitVersion\":\"2021.09.11\"}]}]}");
    }
    #[test]
    fn neutralize_ion() {
        let orig_smiles = "C(C(=O)[O-])[NH3+]";
        let mut pkl_mol = Molecule::new(orig_smiles, "");
        pkl_mol.neutralize("");
        let smiles = pkl_mol.get_smiles("");
        assert_eq!(smiles, "NCC(=O)O");
    }
    #[test]
    fn normalize() {
        let orig_smiles = "CN=N#N";
        let mut pkl_mol = Molecule::new(orig_smiles, "");
        pkl_mol.normalize("");
        let smiles = pkl_mol.get_smiles("");
        assert_eq!(smiles, "CN=[N+]=[N-]");
    }
}
