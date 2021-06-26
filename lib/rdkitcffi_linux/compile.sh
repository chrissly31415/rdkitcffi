#!/bin/bash
export RDBASE="/home/loschen/programs/rdkit"

#cc -o demo.exe -I $RDBASE/Code/ -I $RDBASE/Code/MinimalLib/ demo.c $RDBASE/lib/librdkitcffi.so
cc -o demo.exe -I ~/calc/rust_cheminf/cosmicrust/include demo.c ~/calc/rust_cheminf/cosmicrust/lib/rdkitcffi_linux/linux-64/librdkitcffi.so

