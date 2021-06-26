//! This is an experimental rust wrapper for some functionaltiy of the great [RDKit](https://www.rdkit.org/) library.
//! 
//! It makes use of its new (and still experimental) C Foreign Function Interface (CFFI), see also this [blog post](https://greglandrum.github.io/rdkit-blog/technical/2021/05/01/rdkit-cffi-part1.html).
//! 
//! Use it at your own risk, its not recommended yet for productive use :-)
//! Please note, that only a limited functionality is being exposed via cffi by RDKit.
//! Have a look at the examples below and the test functions.
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
use crate::bindings::{free, free_ptr, size_t};
use crate::bindings::{
    get_inchi, get_inchikey_for_inchi, get_json, get_mol, get_molblock, get_smiles,canonical_tautomer, get_descriptors
};


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

    pub fn get_coords(&self) -> Vec<Vec<f32>> {
        let json_mol = self.get_JsonMolecule("");
        let conf: &JsonConformer = json_mol.conformers.get(0).unwrap().clone();
        conf.coords.to_owned()
    }

    pub fn get_smiles(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let can_smiles_cchar: *mut c_char =
                get_smiles(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let can_smiles: &str = CStr::from_ptr(can_smiles_cchar).to_str().unwrap();
            can_smiles.to_owned()
        }
    }

    pub fn canonical_tautomer(&mut self, json_info: &str) {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            canonical_tautomer(&mut self.pkl_mol as *mut _, self.pkl_size, json_info.as_ptr());
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

    pub fn get_json(&self, json_info: &str) -> String {
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let rdkit_json_cchar = get_json(self.pkl_mol, *self.pkl_size, json_info.as_ptr());
            let mol_json_str = CStr::from_ptr(rdkit_json_cchar).to_str().unwrap();
            mol_json_str.to_owned()
        }
    }

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
            let desc_json: HashMap<String, f32> = serde_json::from_str(&desc_string).expect("Wrong JSON format!?");
            desc_json
    }

    pub fn get_descriptors_as_string(&self) -> String{
        unsafe {
            let desc_cchar: *mut c_char =
            get_descriptors(self.pkl_mol, *self.pkl_size);
            let desc_string: &str = CStr::from_ptr(desc_cchar).to_str().unwrap();
            desc_string.to_owned()
        }
    }


    pub fn PklFromString(input: &str, json_info: &str) -> (*mut i8, *mut size_t) {
        let input_cstr = CString::new(input).unwrap();
        let json_info = CString::new(json_info).unwrap();
        unsafe {
            let pkl_size: *mut size_t = libc::malloc(mem::size_of::<u64>()) as *mut u64;
            let pkl_mol = get_mol(input_cstr.as_ptr(), pkl_size, json_info.as_ptr());
            (pkl_mol, pkl_size)
        }
    }

    pub fn read_smifile(smi_file: &str) -> Vec<Molecule> {
        let smi_file = read_to_string(smi_file).expect("Could not load file.");
        let mut mol_list: Vec<Molecule> = Vec::new();
        let smiles_list: Vec<&str> = smi_file.split("\n").collect();
        for (i, s) in smiles_list.iter().enumerate() {
            if s.len() == 0 {
                continue;
            };
            let s_mod = s.trim_start_matches("\n");
            let mut mol: Molecule = Molecule::new(s_mod, "");
            unsafe {
                if (*mol.pkl_size == 0) {
                    eprintln!("Skipping position: {} - cannot create molecule. ",i);
                    continue;
                }             
            }
            mol_list.push(mol);
        }
        mol_list
    }

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
                    eprintln!("Skipping position: {} - cannot create molecule. ",i);
                    continue;
                }             
            }
            mol_list.push(mol);
        }
        mol_list
    }
}

//this implements the commom chem json structure:
//https://github.com/CommonChem/CommonChem
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonBase {
    pub commonchem: HashMap<String, i32>,
    pub molecules: Vec<JsonMolecule>,
    pub defaults: RdkitDefaults,
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
    use super::*;
    #[test]
    fn smiles2descriptors() {
        let orig_smiles = "OCCC#CO";
        let pkl_mol = Molecule::new(orig_smiles, "");
        let desc = pkl_mol.get_descriptors();
        println!("Descriptors: {:?}",desc)
    }

    #[test]
    fn smifile2molecules() {
        //PklMolecule::read_smifile("examples/ringtest.smi");
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
    }
    #[test]
    fn sdfile2molecules() {
        let mut mol_list: Vec<Molecule> = Molecule::read_sdfile("examples/test.sdf");
        for (i, mol) in mol_list.iter_mut().enumerate() {
            mol.remove_all_hs();
            println!(
                "Pos:{} INCHIKEY: {} SMILES: {} ",
                i,
                mol.get_inchikey(""),
                mol.get_smiles("")
            )
        }
    }
    #[test]
    fn generate3d() {
        let orig_smiles = "OCCC#CO";
        let mut pkl_mol = Molecule::new(orig_smiles, "");
        pkl_mol.set_3d_coords("");
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
    }
    #[test]
    fn inchi_from_smiles_via_pkl() {
        let orig_smiles = "OCCC#CO";
        let pkl_mol = Molecule::new(orig_smiles, "");

        println!("inchi:    {}", pkl_mol.get_inchi(""));
        println!("inchikey: {}", pkl_mol.get_inchikey(""));
    }
    #[test]
    fn molblock_from_smiles_via_pkl() {
        let orig_smiles = "OCCC#CO";
        let pkl_mol = Molecule::new(orig_smiles, "");
        println!("molblock:    {}", pkl_mol.get_molblock(""));
    }
    #[test]
    fn get_json_via_pkl() {
        let orig_smiles = "OCCC#CO";
        let pkl_mol = Molecule::new(orig_smiles, "");
        println!("json:    {}", pkl_mol.get_json(""));
    }
    #[test]
    fn get_json_molecule() {
        let orig_smiles = "OCCC#CO";
        let pkl_mol = Molecule::new(orig_smiles, "");
        println!("json molecule:    {:?}", pkl_mol.get_JsonMolecule(""));
    }
    #[test]
    fn base_functionality() {
        let mut mol_list: Vec<Molecule> = Molecule::read_sdfile("examples/test.sdf");
        for (i, mol) in mol_list.iter_mut().enumerate() {
            mol.remove_all_hs();
            println!(
                "Pos:{:3} natoms: {:3} nbonds: {:3} ",
                i,
                mol.get_numatoms(),
                mol.get_numbonds()
            )
        }
    }
    #[test]
    fn jsonmolecule_from_smiles() {
        let mol = JsonMolecule::JsonMolFromSmiles("CC(C)CCCO", "");
        println!("{:?}", mol);
        //println!("{:?}", mol.bonds);
    }
    #[test]
    fn json_object_from_smiles() {
        let json_str = JSONFromString("CCCI", "");
        println!("JSON:{}", json_str);
    }
    #[test]
    fn json_object_from_sdf() {
        let json_str = JSONFromString("\n     RDKit          2D\n\n  7  6  0  0  0  0  0  0  0  0999 V2000\n    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    1.2990    0.7500    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0\n    2.5981   -0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    3.8971    0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    5.1962   -0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    6.4952    0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    7.7942    1.5000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n  1  2  1  0\n  2  3  1  0\n  3  4  1  0\n  4  5  2  3\n  5  6  1  0\n  6  7  3  0\nM  END\n", "");
        println!("JSON:{}", json_str);
    }
}
