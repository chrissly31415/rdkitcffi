# rdkitcffi

This is an &#128679; experimental  &#128679; rust wrapper for some functionality of the wonderful [RDKit](https://www.rdkit.org/) library.

It makes use of its new (and still experimental) C Foreign Function Interface (CFFI), see also this [blog post](https://greglandrum.github.io/rdkit-blog/technical/2021/05/01/rdkit-cffi-part1.html).
 
Use it at your own risk, its not recommended yet for productive use :-)
Please note, that only a limited functionality is being exposed via cffi by RDKit and not all of this is available yet via this interface.  
Have a look at the examples below and the test functions.

Note: there are still some dependencies to specific version of boost and rdkit (some headers & the shared lib), that imply some manual work, see also the installation section.  
Currently those deps are added directly to the repository for the sake of simplicity, of course this should be done in a better and more dynamic way.  

Currently, only linux is supported, however support for macos should also be viable. 
 
## Examples

Basic usage:
 
```
use rdkitcffi::Molecule;
 
let pkl_mol = Molecule::new("OCCC#CO", "");
let desc = pkl_mol.get_descriptors();
```
 
Working with SD files:
 
```
use rdkitcffi::Molecule;
 
let mut mol_list : Vec<Molecule> = Molecule::read_sdfile("examples/test.sdf");
mol_list.iter_mut().for_each(|m| m.remove_all_hs());
 
```

Getting a JSON version of the molecule (via serde_json):

```
use rdkitcffi::Molecule;
 
let orig_smiles = "OCCC#CO";
let pkl_mol = Molecule::new(orig_smiles, "");
println!("json molecule:    {:?}", pkl_mol.get_JsonMolecule(""));
```

## Installation

In some cases you will have also to install some additional packages for installation:  

```
sudo apt-get install build-essential
sudo apt-get install libclang-dev
```

If you have a rust/cargo installation, just run

```
cargo build  
cargo test  
```

After installation update your LD_LIBRARY_PATH in order to run binaries without cargo, e.g.:   

export LD_LIBRARY_PATH=/home/username/rdkitcffi/lib/rdkitcffi_linux/linux-64/:$LD_LIBRARY_PATH  



