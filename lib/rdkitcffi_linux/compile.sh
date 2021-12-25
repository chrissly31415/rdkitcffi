#!/bin/bash
#export RDBASE="/home/loschen/programs/rdkit"

#cc -o demo.exe -I $RDBASE/Code/ -I $RDBASE/Code/MinimalLib/ demo.c $RDBASE/lib/librdkitcffi.so
cc -v -o demo.exe -I ../..//include -I/home/loschen/programs/boost_1_67_0 demo.c linux-64/librdkitcffi.so

