#!/bin/bash
bindgen include/wrapper.h -o src/bindings.rs -- -I"RDGeneral/export.h" 
