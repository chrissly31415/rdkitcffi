This directory contains just the necessary headers for bindgen/compilation.    

Use e.g. cc -H to identify those header files. Currently the following headers are included:


```
├── boost
│   ├── config
│   │   ├── compiler
│   │   │   ├── clang.hpp
│   │   │   └── gcc.hpp
│   │   ├── detail
│   │   │   ├── posix_features.hpp
│   │   │   ├── select_compiler_config.hpp
│   │   │   ├── select_platform_config.hpp
│   │   │   └── suffix.hpp
│   │   ├── helper_macros.hpp
│   │   ├── platform
│   │   │   └── linux.hpp
│   │   └── user.hpp
│   └── config.hpp
├── cffiwrapper.h
├── RDGeneral
│   ├── export.h
│   └── RDExportMacros.h
```






