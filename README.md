# rdkitcffi

This is an experimental rust wrapper for some functionaltiy of the great [RDKit](https://www.rdkit.org/) library.

It makes use of its new (and still experimental) C Foreign Function Interface (CFFI), see also this [blog post](https://greglandrum.github.io/rdkit-blog/technical/2021/05/01/rdkit-cffi-part1.html).
 
Use it at your own risk, its not recommended yet for productive use :-)
Please note, that only a limited functionality is being exposed via cffi by RDKit.
Have a look at the examples below and the test functions.
 
## Examples

Basic usage:
 
```
use rdkitrust::Molecule;
 
let orig_smiles = "OCCC#CO";
let pkl_mol = Molecule::new(orig_smiles, "");
let desc = pkl_mol.get_descriptors();
```
 
Working with SD files:
 
```
use rdkitrust::Molecule;
 
let mut mol_list : Vec<Molecule> = Molecule::read_sdfile("examples/test.sdf");
mol_list.iter_mut().for_each(|m| m.remove_all_hs());
 
```

Getting a JSON version of the molecule (via serde_json):

```
use rdkitrust::Molecule;
 
let orig_smiles = "OCCC#CO";
let pkl_mol = Molecule::new(orig_smiles, "");
println!("json molecule:    {:?}", pkl_mol.get_JsonMolecule(""));
 
```

