This directory contains the RDKit MinimalLib shared library, see also:   
http://rdkit.blogspot.com/2019/11/introducing-new-rdkit-javascript.html  
https://greglandrum.github.io/rdkit-blog/technical/2021/05/01/rdkit-cffi-part1.html  


Useful commands for test compiling some C code   

```
#!/bin/bash
export RDBASE="/home/user/programs/rdkit"

cc -o demo.exe -I $RDBASE/Code/ -I $RDBASE/Code/MinimalLib/ demo.c $RDBASE/lib/librdkitcffi.so
cc -H -v -o demo.exe -I ../..//include -I/home/loschen/programs/boost_1_67_0 demo.c linux-64/librdkitcffi.so
```