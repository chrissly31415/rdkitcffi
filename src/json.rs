use crate::Molecule;
use serde::{Deserialize, Serialize};
use serde_json::value::Value;

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
    pub fn new(molstring: &str) -> JsonMolecule {
        JsonMolecule::json_mol_from_string(molstring, "")
    }

    pub fn json_mol_from_string(molstring: &str, json_info: &str) -> JsonMolecule {
        let json_str = jsonfrom_string(molstring);
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

pub fn jsonfrom_string(input: &str) -> String {
    let pkl_mol = Molecule::new(input).unwrap();
    pkl_mol.get_json("")
}
