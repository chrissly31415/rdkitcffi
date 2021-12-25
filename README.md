# rdkitcffi

This is an experimental rust wrapper for some functionaltiy of the great [RDKit](https://www.rdkit.org/) library.

It makes use of its new (and still experimental) C Foreign Function Interface (CFFI), see also this [blog post](https://greglandrum.github.io/rdkit-blog/technical/2021/05/01/rdkit-cffi-part1.html).
 
Use it at your own risk, its not recommended yet for productive use :-)
Please note, that only a limited functionality is being exposed via cffi by RDKit.
Have a look at the examples below and the test functions.

Unfortunately, there are still some dependencies that avoid making the installation straight forward, see also the installation section.
 
## Examples

Basic usage:
 
```
use rdkitcffi::Molecule;
 
let orig_smiles = "OCCC#CO";
let pkl_mol = Molecule::new(orig_smiles, "");
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


Currently you will have to download the boost header libraries before installation, e.g.:  

```
wget https://boostorg.jfrog.io/artifactory/main/release/1.68.0/source/boost_1_68_0.zip
cd include
unzip ../boost_1_68_0.zip
```

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



