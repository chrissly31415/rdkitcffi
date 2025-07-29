[![Rust](https://github.com/chrissly31415/rdkitcffi/actions/workflows/rust.yml/badge.svg)](https://github.com/chrissly31415/rdkitcffi/actions/workflows/rust.yml)
[![RDKit CFFI Build](https://github.com/chrissly31415/rdkitcffi/actions/workflows/rdkit_cffi.yml/badge.svg)](https://github.com/chrissly31415/rdkitcffi/actions/workflows/rdkit_cffi.yml)

# rdkitcffi

This is an &#128679; rust wrapper  &#128679;  for some core functionality of the great [RDKit](https://www.rdkit.org/) cheminformatics library.

It makes use of its CFFI interface, see also this [blog post](https://greglandrum.github.io/rdkit-blog/technical/2021/05/01/rdkit-cffi-part1.html).
 
Only a limited functionality is being exposed via cffi by RDKit and not all of this is available yet via this interface. Have a look at the examples below and the test functions.  
 
The rust wrapper is linked against a pre-built RDKit shared library (MinimalLib), which is stored as an artifact created via github actions and is automatically downloaded from this [location](https://github.com/chrissly31415/rdkitcffi/releases/download/rdkit-latest/rdkitcffi_linux.tar.gz) during build. 
Currently, only linux is supported.  

 ## Examples

 Basic usage:

 ```rust
 use rdkitcffi::Molecule;

 let smiles = "OCCC#CO";
 let mol = Molecule::new(smiles).unwrap();

 let natoms = mol.get_numatoms();
 ```
 
 Additional arguments can be passed via json

 ```rust
 use rdkitcffi::Molecule;

 let json_args = "{\"removeHs\":false,\"canonical\":false}";
 let mol = Molecule::new_with_args("c1cc(O[H])ccc1", json_args).unwrap();
 ```

 Working with SD files and filtering invalid molecules:

 ```rust
use rdkitcffi::{Molecule,read_sdfile};
 
 let mut mol_opt_list : Vec<Option<Molecule>>= read_sdfile("data/test.sdf");
 let mut mol_list: Vec<Molecule> = mol_opt_list.into_iter().filter_map(|m| m).collect();
 mol_list.iter_mut().for_each(|m| m.remove_all_hs());

 ```

 Dealing with invalid molecules

 ```rust
 use rdkitcffi::Molecule;

 let result = Molecule::new("OCCO");
 match result {
    Some(m) => println!("Molecule: {:?}", m),
    None => println!("Could not get molecule!"),
};
 ```


 Getting a JSON represenation (via serde_json):

 ```rust
 use rdkitcffi::Molecule;

 let mol = Molecule::new("OCCO").unwrap();
 println!("json: {:?}", mol.get_json(""));

 ```

 Neutralizing a zwitterion

 ```rust
 use rdkitcffi::Molecule;

 let mut mol = Molecule::new("C(C(=O)[O-])[NH3+]").unwrap();
 mol.neutralize("");
 println!("{:?}", mol.get_smiles(""));

 ```

 Computing RDKit descriptors

 ```rust
 use rdkitcffi::Molecule;

 let mol = Molecule::new("CCCN").unwrap();
 let desc = mol.get_descriptors_as_dict();
 let nrot = desc.get("NumRotatableBonds");
 let logp = desc.get("CrippenClogP");

 ```

 Generating 3D coordinates

 ```rust
 use rdkitcffi::Molecule;

 let mut mol = Molecule::new("CO").unwrap();
 mol.add_hs();
 mol.set_3d_coords("");
 let coords: Vec<Vec<f32>> = mol.get_coords();

 ```

 Creating a polars dataframe:

 ```rust
 use rdkitcffi::Molecule;
 use polars::prelude::*;
 use polars::df;

 let mut mol_list : Vec<Molecule> = rdkitcffi::read_smifile_unwrap("data/test.smi");
 let a: Vec<_> = mol_list.iter().map(|m| m.get_smiles("")).collect();
 let df = df!( "smiles" => a).unwrap();

 ```

## Installation
Currently only linux is supported.   

Download the repo:  

```
git clone https://github.com/chrissly31415/rdkitcffi.git  
```

If you have a rust/cargo installation, just run

```
cd rdkitcffi
cargo build  
cargo test --lib  
```

### Dynamic libraries
After installation you may want to update your LD_LIBRARY_PATH in order to run binaries without cargo, e.g.:   

```
export LD_LIBRARY_PATH=/home/username/rdkitcffi/lib/rdkitcffi_linux/linux-64/:$LD_LIBRARY_PATH
```

In case you are missing libboost-serialization libs:
```
sudo apt-get install -y libboost-all-dev
```

## Using it in your project

Modify your Cargo.toml file:   

```
[dependencies]
rdkitcffi = {path="/pathtorepo/rdkitcffi"} 
```




