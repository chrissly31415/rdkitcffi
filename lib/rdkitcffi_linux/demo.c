#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "cffiwrapper.h"

void canon_smiles(){
  char *pkl;
  size_t pkl_size;
  
  pkl = get_mol("c1cc(O)ccc1",&pkl_size,"");
  
  //get smiles
  char *smiles=get_smiles(pkl,pkl_size,NULL);
  printf("Canonical SMILES: %s\n",smiles);

  //get 3D structure
  add_hs(&pkl,&pkl_size);

  set_3d_coords(&pkl,&pkl_size,"{\"randomSeed\":42}");

  char *molb = get_molblock(pkl,pkl_size,NULL);
  printf("%s\n",molb);
  free(molb);
  free(pkl);
}

int main(){
  enable_logging();
  printf("RDKIT CFFI %s\n",version()); 
  canon_smiles();
  return 0;
}
