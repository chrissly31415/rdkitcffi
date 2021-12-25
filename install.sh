#!/bin/bash
# This contains some commands that may be necessary in order to install some dependencies
wget https://boostorg.jfrog.io/artifactory/main/release/1.68.0/source/boost_1_68_0.zip
cd include
unzip ../boost_1_68_0.zip

cp $RD_BASE/Code/MinimalLib/cffiwrapper.h .
cp -r $RD_BASE/Code/RDGeneral .

sudo apt-get install build-essential
sudo apt-get install libclang-dev

