#!/bin/bash
# This contains some commands that may be necessary in order to install or update some dependencies
wget https://boostorg.jfrog.io/artifactory/main/release/1.68.0/source/boost_1_68_0.zip
cd include
unzip ../boost_1_68_0.zip

cp $RD_BASE/Code/MinimalLib/cffiwrapper.h .
cp -r $RD_BASE/Code/RDGeneral .

sudo apt-get install build-essential
sudo apt-get install libclang-dev


# select boost headers, e.g. use cc -H to identify them
cd ./include/boost
cp boost_1_67_0/boost/config.hpp .
cp boost_1_67_0/boost/config/user.hpp config
cp boost_1_67_0/boost/config/detail/select_compiler_config.hpp config/detail/
cp boost_1_67_0/boost/config/compiler/gcc.hpp config/compiler/
cp boost_1_67_0/boost/config/detail/select_platform_config.hpp config/detail/
cp boost_1_67_0/boost/config/platform/linux.hpp config/platform/
cp boost_1_67_0/boost/config/detail/posix_features.hpp config/detail/
cp boost_1_67_0/boost/config/detail/suffix.hpp config/detail/
cp boost_1_67_0/boost/config/helper_macros.hpp config
cp boost_1_67_0/boost/config/compiler/clang.hpp config/compiler/

