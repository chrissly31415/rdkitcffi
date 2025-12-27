extern crate rdkitcffi;
use rdkitcffi::bindings::{disable_logging, enable_logging, free_ptr, version};
use rdkitcffi::{
    json::jsonfrom_string, read_sdfile, read_sdfile_unwrap, read_smifile_unwrap, JsonMolecule,
    Molecule,
};
use serde_json::json;
use std::ffi::CStr;

#[test]
fn basics() {
    unsafe {
        enable_logging();
        let version_str = version();
        println!(
            "Version: {:?}",
            CStr::from_ptr(version_str).to_str().unwrap()
        );
        free_ptr(version_str);
        disable_logging();
    }
}

#[test]
fn smiles2descriptors() {
    let orig_smiles = "CCCN";
    let pkl_mol = Molecule::new(orig_smiles).unwrap();
    let desc = pkl_mol.get_descriptors_as_dict();
    println!("Descriptors: {:?}", desc);
    let nheavy = desc.get("NumHeavyAtoms").unwrap().round() as i32;
    assert_eq!(nheavy, 4);
    let nrot = desc.get("NumRotatableBonds").unwrap().round() as i32;
    assert_eq!(nrot, 1);
    // Molecule will be dropped here, which should clean up resources
}

#[test]
fn smifile2molecules() {
    let mut mol_list: Vec<Molecule> = read_smifile_unwrap("data/ringtest.smi");
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

/*     #[test]
fn sditerator() {
    let sdreader = SDIterator::new("data/test.sdf");
    for mol_opt in sdreader {
        println!("natoms: {}",mol_opt.unwrap().get_numatoms());
    }

} */

#[test]
fn sdfile2molecules() {
    let mut mol_list: Vec<Option<Molecule>> = read_sdfile("data/test.sdf");
    println!("mols: {}", mol_list.len());
    let mut valid_mols: Vec<_> = mol_list.iter_mut().filter_map(Option::as_mut).collect();

    for (i, mol) in valid_mols.iter_mut().enumerate() {
        mol.remove_all_hs();
        println!(
            "Pos:{} INCHIKEY: {} SMILES: {} NUMATOMS: {} NUMBONDS: {}",
            i,
            mol.get_inchikey(""),
            mol.get_smiles(""),
            mol.get_numatoms(),
            mol.get_numbonds(),
        );
    }
    assert_eq!(mol_list.len(), 9);
}
#[test]
fn sdfile2molecules_win() {
    //tests windows newlines
    let mut mol_list: Vec<Molecule> = read_sdfile_unwrap("data/test_win.sdf");
    for (i, mol) in mol_list.iter_mut().enumerate() {
        mol.remove_all_hs();
        println!(
            "Pos:{} INCHIKEY: {} SMILES: {} ",
            i,
            mol.get_inchikey(""),
            mol.get_smiles(""),
        );
        println!(
            "Pos:{} INCHIKEY: {} SMILES: {} NUMATOMS: {} NUMBONDS: {}",
            i,
            mol.get_inchikey(""),
            mol.get_smiles(""),
            mol.get_numatoms(),
            mol.get_numbonds(),
        )
    }
    assert_eq!(mol_list.len(), 19);
}
#[test]
fn morgan_fp() {
    let smiles = "OCC=CCO";
    let options = json!({
        "radius": 2,
        "nBits": 64
    })
    .to_string();
    let mol = Molecule::new(smiles).unwrap();
    let fps = mol.get_morgan_fp(&options);
    println!("Fingerprints: {:?}", fps);
    println!("Length: {:?}", fps.len());
    assert_eq!(
        fps,
        "0000000000011000100000000000001000000001000000000000001000010001"
    );
}
#[test]
fn morgan_fp_bytes() {
    let smiles = "OCC=CCO";
    let options = r#"{
            "radius": 2,
            "nBits": 64
        }"#;
    let mol = Molecule::new(smiles).unwrap();
    let fps: Vec<i8> = mol.get_morgan_fp_as_bytes(options);
    let expected_fps: Vec<i8> = vec![0, 24, 1, 64, -128, 0, 64, -120];
    assert_eq!(fps, expected_fps);
}

#[test]
fn rdkit_fp() {
    let smiles = "OCC=CCO";
    let options = json!({
        "nBits": 64
    })
    .to_string();
    let mol = Molecule::new(smiles).unwrap();
    let fps = mol.get_rdkit_fp(&options);
    println!("RDKit Fingerprints: {:?}", fps);
    println!("Length: {:?}", fps.len());
    assert_eq!(fps.len(), 64);
    assert_eq!(
        fps,
        "1000100100000000010000000000100100000000001110000010101011011000"
    );
}

#[test]
fn rdkit_fp_bytes() {
    let smiles = "OCC=CCO";
    let options = r#"{
            "radius": 2,
            "nBits": 64
        }"#;
    let mol = Molecule::new(smiles).unwrap();
    let fps: Vec<i8> = mol.get_rdkit_fp_as_bytes(options);
    println!("RDKit fingerprint bytes: {:?}", fps);
    // Should have 8 bytes for 64 bits
    assert_eq!(fps.len(), 8);
    let expected_fps: Vec<i8> = vec![-111, 0, 2, -112, 0, 28, 84, 27];
    assert_eq!(fps, expected_fps);
}

#[test]
fn pattern_fp() {
    let smiles = "OCC=CCO";
    let options = json!({
        "nBits": 64
    })
    .to_string();
    let mol = Molecule::new(smiles).unwrap();
    let fps = mol.get_pattern_fp(&options);
    println!("Pattern Fingerprints: {:?}", fps);
    println!("Length: {:?}", fps.len());
    assert_eq!(fps.len(), 64);
    assert_eq!(fps.len(), 64);
    assert_eq!(
        fps,
        "0001001101110101010011000101001000011100011101100010001100000010"
    );
}

#[test]
fn pattern_fp_bytes() {
    let smiles = "OCC=CCO";
    let options = r#"{
            "nBits": 64
        }"#;
    let mol = Molecule::new(smiles).unwrap();
    let fps: Vec<i8> = mol.get_pattern_fp_as_bytes(options);
    println!("Pattern fingerprint bytes: {:?}", fps);
    // Should have 8 bytes for 64 bits
    assert_eq!(fps.len(), 8);
    // Should not be all zeros for this molecule
    assert!(fps.iter().any(|&x| x != 0));
    let expected_fps: Vec<i8> = vec![-56, -82, 50, 74, 56, 110, -60, 64];
    assert_eq!(fps, expected_fps);
}

#[test]
fn generate3d() {
    let orig_smiles = "CC";
    let mut pkl_mol = Molecule::new(orig_smiles).unwrap();
    pkl_mol.set_3d_coords("");
    let coords = pkl_mol.get_coords();
    assert_eq!(coords.len(), 2);
    assert_eq!(coords[0].len(), 3);
}
#[test]
fn smiles_from_smiles_via_pkl() {
    let orig_smiles = "OCCC#CO";
    let pkl_mol = Molecule::new(orig_smiles).unwrap();
    println!(
        "SMILES: {} Canonical SMILES: {}",
        orig_smiles,
        pkl_mol.get_smiles("")
    );
    assert_eq!(pkl_mol.get_smiles(""), "OC#CCCO");
}
#[test]
fn inchi_from_smiles() {
    let orig_smiles = "OCCC#CO";
    let mol = Molecule::new(orig_smiles).unwrap();
    println!("inchi:    {}", mol.get_inchi(""));
    println!("inchikey: {}", mol.get_inchikey(""));
    assert_eq!(
        mol.get_inchi(""),
        "InChI=1S/C4H6O2/c5-3-1-2-4-6/h5-6H,1,3H2"
    );
    assert_eq!(mol.get_inchikey(""), "JSPXPZKDILSYNN-UHFFFAOYSA-N");
}
#[test]
fn molblock_from_smiles_via_pkl() {
    let orig_smiles = "CCO";
    let mol = Molecule::new(orig_smiles).unwrap();
    println!("molblock:*{}*", mol.get_molblock(""));
    assert_eq!(mol.get_molblock(""),"\n     RDKit          2D\n\n  3  2  0  0  0  0  0  0  0  0999 V2000\n    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    1.2990    0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    2.5981   -0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0\n  1  2  1  0\n  2  3  1  0\nM  END\n");
}

#[test]
fn get_json_via_pkl() {
    let expected = r#"{"rdkitjson":{"version":11},"defaults":{"atom":{"z":6,"impHs":0,"chg":0,"nRad":0,"isotope":0,"stereo":"unspecified"},"bond":{"bo":1,"stereo":"unspecified"}},"molecules":[{"atoms":[{"z":8,"impHs":1},{"impHs":2},{"z":8,"impHs":1}],"bonds":[{"atoms":[0,1]},{"atoms":[1,2]}],"extensions":[{"name":"rdkitRepresentation","formatVersion":2,"toolkitVersion":"2024.09.6"}]}]}"#;
    let orig_smiles = "OCO";
    let mol = Molecule::new(orig_smiles).unwrap();
    println!("json:    {}", mol.get_json(""));
    assert_eq!(mol.get_json(""), expected);
}
#[test]
fn get_json_molecule() {
    let orig_smiles = "C#C";
    let pkl_mol = Molecule::new(orig_smiles).unwrap();
    let json_mol = pkl_mol.get_json_molecule();
    println!("json molecule:    {:?}", json_mol);
    let atoms = json_mol.atoms;
    assert_eq!(atoms.len(), 2);
}
#[test]
fn jsonmolecule_from_smiles() {
    let json_mol = JsonMolecule::json_mol_from_smiles("CC(C)CCCO");
    println!("{:?}", json_mol);
    let bonds = json_mol.bonds;
    assert_eq!(bonds.len(), 6);
}
#[test]
fn json_str_from_smiles() {
    let expected = r#"{"rdkitjson":{"version":11},"defaults":{"atom":{"z":6,"impHs":0,"chg":0,"nRad":0,"isotope":0,"stereo":"unspecified"},"bond":{"bo":1,"stereo":"unspecified"}},"molecules":[{"atoms":[{"impHs":3},{"impHs":2},{"impHs":2},{"z":53}],"bonds":[{"atoms":[0,1]},{"atoms":[1,2]},{"atoms":[2,3]}],"extensions":[{"name":"rdkitRepresentation","formatVersion":2,"toolkitVersion":"2024.09.6"}]}]}"#;
    let json_str = jsonfrom_string("CCCI");
    assert_eq!(json_str, expected);
}
#[test]
fn json_str_from_sdf() {
    let expected = r#"{"rdkitjson":{"version":11},"defaults":{"atom":{"z":6,"impHs":0,"chg":0,"nRad":0,"isotope":0,"stereo":"unspecified"},"bond":{"bo":1,"stereo":"unspecified"}},"molecules":[{"name":"","atoms":[{"impHs":3},{"z":8},{"impHs":2},{"impHs":1},{"impHs":1},{},{"impHs":1}],"bonds":[{"atoms":[0,1]},{"atoms":[1,2]},{"atoms":[2,3]},{"bo":2,"atoms":[3,4],"stereo":"either"},{"atoms":[4,5]},{"bo":3,"atoms":[5,6]}],"conformers":[{"dim":2,"coords":[[0.0,0.0],[1.299,0.75],[2.5981,-0.0],[3.8971,0.75],[5.1962,-0.0],[6.4952,0.75],[7.7942,1.5]]}],"extensions":[{"name":"rdkitRepresentation","formatVersion":2,"toolkitVersion":"2024.09.6"}]}]}"#;
    let json_str = jsonfrom_string("\n     RDKit          2D\n\n  7  6  0  0  0  0  0  0  0  0999 V2000\n    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    1.2990    0.7500    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0\n    2.5981   -0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    3.8971    0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    5.1962   -0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    6.4952    0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    7.7942    1.5000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n  1  2  1  0\n  2  3  1  0\n  3  4  1  0\n  4  5  2  3\n  5  6  1  0\n  6  7  3  0\nM  END\n");
    assert_eq!(json_str, expected);
}
#[test]
fn neutralize_ion() {
    let orig_smiles = "C(C(=O)[O-])[NH3+]";
    let mut mol = Molecule::new(orig_smiles).unwrap();
    mol.neutralize("");
    let smiles = mol.get_smiles("");
    assert_eq!(smiles, "NCC(=O)O");
}
#[test]
fn normalize() {
    let orig_smiles = "CN=N#N";
    let mut pkl_mol = Molecule::new(orig_smiles).unwrap();
    pkl_mol.normalize("");
    let smiles = pkl_mol.get_smiles("");
    assert_eq!(smiles, "CN=[N+]=[N-]");
}
#[test]
fn smarts_cxssmiles() {
    let orig_smiles = "CN=N#N";
    let pkl_mol = Molecule::new(orig_smiles).unwrap();
    let smarts = pkl_mol.get_smarts("");
    assert_eq!(smarts, "[#6]-[#7]=[#7+]=[#7-]");

    let cx_input = "CO |$C2;O1$| carbon monoxide'";
    let pkl_mol2 = Molecule::new(cx_input).unwrap();
    let cxsmiles = pkl_mol2.get_cxsmiles("");
    println!("cxsmiles: {:?}", cxsmiles);
    assert_eq!(cxsmiles, "CO |$C2;O1$|");
}
#[test]
fn find_substructure() {
    let mol = Molecule::new("Cl[C@H](F)C[C@H](F)Cl").unwrap();
    let query_mol = Molecule::get_qmol("Cl[C@@H](F)C", "").unwrap();
    let res = mol.get_substruct_match(&query_mol, "");
    assert_eq!(res, "{\"atoms\":[0,1,2,3],\"bonds\":[0,1,2]}");
    let res = mol.get_substruct_matches(&query_mol, "");
    assert_eq!(
        res,
        "[{\"atoms\":[0,1,2,3],\"bonds\":[0,1,2]},{\"atoms\":[6,4,5,3],\"bonds\":[5,4,3]}]"
    );
}
#[test]
fn create_image() {
    let orig_smiles = "CN=N#N";
    let pkl_mol = Molecule::new(orig_smiles).unwrap();
    let svg = pkl_mol.get_svg("{\"width\":350,\"height\":300}");
    assert!(svg.contains("width='350px'"));
    assert!(svg.contains("height='300px'"));
    assert!(svg.contains("</svg>"));
}
#[test]
fn test_molblock() {
    let pkl_mol = Molecule::new("CN=N#N").unwrap();
    let v3k_molblock = pkl_mol.get_v3kmolblock("");
    let res = Molecule::new(&v3k_molblock);
    assert!(res.is_some());
}
#[test]
fn bad_mol() {
    let molblock = "THIOL_12\n     RDKit          3D\n\n 25 25  0  0  0  0  0  0  0  0999 V2000\n   -2.2510   -2.6650   -2.0550 S   0  0  0  0  0  0  0  0  0  0  0  0\n   -3.3040   -2.7120   -2.1100 H   0  0  0  0  0  0  0  0  0  0  0  0\n   -1.7910   -1.5140   -0.7240 C   0  0  0  0  0  0  0  0  0  0  0  0\n   -2.1270   -2.0270    0.1920 H   0  0  0  0  0  0  0  0  0  0  0  0\n   -2.4730   -0.6640   -0.8710 H   0  0  0  0  0  0  0  0  0  0  0  0\n   -0.2780   -0.7500   -0.3280 C   0  0  0  0  0  0  0  0  0  0  0  0\n    0.2420   -1.8480   -0.5140 O   0  0  0  0  0  0  0  0  0  0  0  0\n    0.4860    0.3560   -0.2740 N   0  0  0  0  0  0  0  0  0  0  0  0\n    0.0540    1.2670   -0.1190 H   0  0  0  0  0  0  0  0  0  0  0  0\n    1.9050    0.2450   -0.7390 C   0  0  1  0  0  0  0  0  0  0  0  0\n    1.9190   -0.1360   -1.7740 H   0  0  0  0  0  0  0  0  0  0  0  0\n    2.4830    1.6820   -0.6980 C   0  0  0  0  0  0  0  0  0  0  0  0\n    2.4150    2.0880    0.3240 H   0  0  0  0  0  0  0  0  0  0  0  0\n    3.5420    1.6740   -0.9990 H   0  0  0  0  0  0  0  0  0  0  0  0\n    1.7270    2.5420   -1.6810 C   0  0  0  0  0  0  0  0  0  0  0  0\n    1.7070    2.3180   -2.9770 N   0  0  0  0  0  0  0  0  0  0  0  0\n    2.2060    1.5550   -3.4590 H   0  0  0  0  0  0  0  0  0  0  0  0\n    0.9600    3.1990   -3.5600 C   0  0  0  0  0  0  0  0  0  0  0  0\n    0.7540    3.2500   -4.6280 H   0  0  0  0  0  0  0  0  0  0  0  0\n    0.5040    3.9880   -2.6970 N   0  0  0  0  0  0  0  0  0  0  0  0\n    0.9760    3.6220   -1.3870 C   0  0  0  0  0  0  0  0  0  0  0  0\n    0.7740    4.0890   -0.4240 H   0  0  0  0  0  0  0  0  0  0  0  0\n    2.7730   -0.7080    0.0810 C   0  0  0  0  0  0  0  0  0  0  0  0\n    3.9440   -0.8660   -0.2210 O   0  0  0  0  0  0  0  0  0  0  0  0\n    2.2450   -1.3190    1.1390 O   0  0  0  0  0  0  0  0  0  0  0  0\n  1  2  1  0\n  1  3  1  0\n  3  4  1  0\n  3  5  1  0\n  3  6  1  0\n  6  7  2  0\n  6  8  1  0\n  8  9  1  0\n  8 10  1  0\n 10 11  1  6\n 10 12  1  0\n 10 23  1  0\n 12 13  1  0\n 12 14  1  0\n 12 15  1  0\n 15 16  1  0\n 15 21  2  0\n 16 17  1  0\n 16 18  1  0\n 18 19  1  0\n 18 20  2  0\n 20 21  1  0\n 21 22  1  0\n 23 24  2  0\n 23 25  1  0\nM  CHG  1  25  -1\nM  END\n";
    let mut pkl_mol = Molecule::new(molblock).unwrap();
    println!("1: {:?}", pkl_mol);
    pkl_mol.cleanup(""); // this is needed to avoid exception...
                         //pkl_mol2.remove_all_hs();
    println!("2: {:?}", pkl_mol);
    pkl_mol.canonical_tautomer("");
    assert_eq!(pkl_mol.get_smiles(""), "O=C(CS)NC(Cc1c[nH]cn1)C(=O)[O-]");
}
#[test]
fn json_details() {
    let json_args = "{\"removeHs\":false,\"canonical\":false}";
    let pkl_mol = Molecule::new_with_args("c1cc(O[H])ccc1", json_args).unwrap();
    println!("{:?}", pkl_mol);
    assert_eq!(pkl_mol.get_smiles(""), "[H]Oc1ccccc1");
}
#[test]
fn fragments() {
    let json_args = "{\"sanitize\":false}";
    let mut pkl_mol = Molecule::new_with_args("[Pt]CCN(=O)=O", json_args).unwrap();
    let smiles = pkl_mol.get_smiles(json_args);
    assert_eq!(smiles, "O=N(=O)CC[Pt]");
    pkl_mol.cleanup("");
    assert_eq!(pkl_mol.get_smiles(""), "[CH2-]C[N+](=O)[O-].[Pt+]");
    pkl_mol.fragment_parent("");
}

// Property function tests - commented out until available in RDKit CFFI library
// TODO: Uncomment when property functions are added to the library
//
// #[test]
// fn test_has_prop() {
//     let mol = Molecule::new("CCO").unwrap();
//     // Test with non-existent property
//     assert_eq!(mol.has_prop("nonexistent"), false);
//     assert!(!mol.has_prop("_nonexistent_property_12345"));
//     assert!(!mol.has_prop("test_key"));
// }

// Property function tests - commented out until available in RDKit CFFI library
// TODO: Uncomment when property functions are added to the library
//
// #[test]
// fn test_set_and_get_prop() {
//     let mut mol = Molecule::new("CCO").unwrap();
//
//     // Set a property
//     mol.set_prop("test_key", "test_value", false);
//
//     // Check it exists
//     assert!(mol.has_prop("test_key"));
//
//     // Get the property
//     let value = mol.get_prop("test_key");
//     assert_eq!(value, Some("test_value".to_string()));
//
//     // Test with computed property
//     mol.set_prop("computed_key", "computed_value", true);
//     assert!(mol.has_prop("computed_key"));
//     assert_eq!(
//         mol.get_prop("computed_key"),
//         Some("computed_value".to_string())
//     );
// }
//
// #[test]
// fn test_get_prop_nonexistent() {
//     let mol = Molecule::new("CCO").unwrap();
//     // Get non-existent property
//     assert_eq!(mol.get_prop("nonexistent"), None);
//     assert!(!mol.has_prop("nonexistent"));
// }
//
// #[test]
// fn test_get_prop_list() {
//     let mut mol = Molecule::new("CCO").unwrap();
//
//     // Set some properties
//     mol.set_prop("prop1", "value1", false);
//     mol.set_prop("prop2", "value2", false);
//     mol.set_prop("computed_prop", "computed_value", true);
//
//     // Get all properties
//     let all_props = mol.get_prop_list(true, true);
//     println!("All properties: {:?}", all_props);
//     assert!(all_props.len() >= 3); // At least our 3 properties
//
//     // Get only non-computed properties
//     let non_computed = mol.get_prop_list(true, false);
//     println!("Non-computed properties: {:?}", non_computed);
//     assert!(non_computed.iter().any(|p| p == "prop1"));
//     assert!(non_computed.iter().any(|p| p == "prop2"));
// }
//
// #[test]
// fn test_clear_prop() {
//     let mut mol = Molecule::new("CCO").unwrap();
//
//     // Set a property
//     mol.set_prop("to_clear", "value", false);
//     assert!(mol.has_prop("to_clear"));
//
//     // Clear it
//     let cleared = mol.clear_prop("to_clear");
//     assert!(cleared);
//     assert!(!mol.has_prop("to_clear"));
//     assert_eq!(mol.get_prop("to_clear"), None);
//
//     // Try to clear non-existent property
//     let not_cleared = mol.clear_prop("nonexistent");
//     assert!(!not_cleared);
// }
//
// #[test]
// fn test_keep_props() {
//     let mut mol = Molecule::new("CCO").unwrap();
//
//     // Set multiple properties
//     mol.set_prop("keep1", "value1", false);
//     mol.set_prop("keep2", "value2", false);
//     mol.set_prop("remove1", "value3", false);
//     mol.set_prop("remove2", "value4", false);
//
//     // Verify all exist
//     assert!(mol.has_prop("keep1"));
//     assert!(mol.has_prop("keep2"));
//     assert!(mol.has_prop("remove1"));
//     assert!(mol.has_prop("remove2"));
//
//     // Keep only keep1 and keep2
//     // Note: The JSON format for keep_props depends on RDKit's implementation
//     // This is a basic test - the actual JSON format may need adjustment
//     let keep_json = r#"{"props":["keep1","keep2"]}"#;
//     mol.keep_props(keep_json);
//
//     // After keep_props, the properties should be filtered
//     // The exact behavior depends on RDKit's implementation
//     println!(
//         "Properties after keep_props: {:?}",
//         mol.get_prop_list(true, true)
//     );
// }
//
// #[test]
// fn test_property_operations_chain() {
//     let mut mol = Molecule::new("c1ccccc1O").unwrap(); // phenol
//
//     // Chain of property operations
//     mol.set_prop("name", "phenol", false);
//     mol.set_prop("mw", "94.11", false);
//     mol.set_prop("formula", "C6H6O", false);
//
//     assert_eq!(mol.get_prop("name"), Some("phenol".to_string()));
//     assert_eq!(mol.get_prop("mw"), Some("94.11".to_string()));
//     assert_eq!(mol.get_prop("formula"), Some("C6H6O".to_string()));
//
//     // Update a property
//     mol.set_prop("mw", "94.11 g/mol", false);
//     assert_eq!(mol.get_prop("mw"), Some("94.11 g/mol".to_string()));
//
//     // Clear one property
//     mol.clear_prop("formula");
//     assert!(!mol.has_prop("formula"));
//     assert!(mol.has_prop("name"));
//     assert!(mol.has_prop("mw"));
// }
