# rdkitcffi

This is an &#128679; experimental  &#128679; rust wrapper for some functionality of the wonderful [RDKit](https://www.rdkit.org/) library.

It makes use of its new C Foreign Function Interface (CFFI), see also this [blog post](https://greglandrum.github.io/rdkit-blog/technical/2021/05/01/rdkit-cffi-part1.html).
 
Use it at your own risk, its not recommended yet for productive use :-)  

Please note, that only a limited functionality is being exposed via cffi by RDKit and not all of this is available yet via this interface.  
Have a look at the examples below and the test functions.  

There are still some dependencies to specific version of boost and rdkit (some headers & the shared lib), that imply some manual work, see also the installation section.  
Currently those deps are added directly to the repository for the sake of simplicity, of course this should be done in a better and more dynamic way.  

Currently, only linux is supported, however support for macos should also be viable. 

 ## Examples

 Basic usage:

 ```
 use rdkitcffi::Molecule;

 let smiles = "OCCC#CO";
 let pkl_mol = Molecule::new(smiles, "").unwrap();
 
 let desc = pkl_mol.get_descriptors();
 ```

Working with SD files and filter None values:
```
use rdkitcffi::Molecule;

let mut mol_opt_list : Vec<Option<Molecule>>= rdkitcffi::read_sdfile("data/test.sdf");
let mut mol_list: Vec<Molecule> = mol_opt_list.into_iter().filter_map(|m| m).collect();
mol_list.iter_mut().for_each(|m| m.remove_all_hs());
```

 Dealing with invalid molecules / error handling
 
 ```
 use rdkitcffi::Molecule;
 
 let result = Molecule::new("OCCO", "");
 match result {
    Some(m) => println!("Result: {:?}", m),
    None => println!("Could not get molecule!"),
};
 ```
 
 Getting a JSON version of the molecule (via serde_json):

 ```
 use rdkitcffi::Molecule;

 let pkl_mol = Molecule::new("OCCO", "").unwrap();
 println!("json: {:?}", pkl_mol.get_JsonMolecule());

 ```

 Neutralizing a zwitterion

 ```
 use rdkitcffi::Molecule;

 let mut pkl_mol = Molecule::new("C(C(=O)[O-])[NH3+]", "").unwrap();
 pkl_mol.neutralize("");
 println!("{:?}", pkl_mol.get_smiles(""));

 ```

 Computing RDKit descriptors

 ```
 use rdkitcffi::Molecule;

 let pkl_mol = Molecule::new("CCCN", "").unwrap();
 let desc = pkl_mol.get_descriptors();
 let nrot = desc.get("NumRotatableBonds");
 let logp = desc.get("CrippenClogP");

 ```

Creating a polars dataframe:

```
use rdkitcffi::Molecule;
use polars::prelude::*;
use polars::df;

let mut mol_list : Vec<Molecule> = rdkitcffi::read_smifile_unwrap("data/test.smi");
let a: Vec<_> = mol_list.iter().map(|m| m.get_smiles("")).collect();
let df = df!( "smiles" => a).unwrap();

```

## Installation

Currently only linux is supported.   
In some cases you may have also to install some additional packages for installation:

```
sudo apt-get install build-essential
sudo apt-get install libclang-dev
```

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

After installation update your LD_LIBRARY_PATH in order to run binaries without cargo, e.g.:   

export LD_LIBRARY_PATH=/home/username/rdkitcffi/lib/rdkitcffi_linux/linux-64/:$LD_LIBRARY_PATH  



