#!/bin/bash
# Make all necessary methods public for external browser-apis crate

# jsobject.rs
sed -i '186s/pub(crate) fn from_proto_and_data_with_shared_shape/pub fn from_proto_and_data_with_shared_shape/' core/engine/src/object/jsobject.rs

# builder.rs
sed -i '232s/pub(crate) fn static_property/pub fn static_property/' core/engine/src/builtins/builder.rs
sed -i '282s/pub(crate) fn method/pub fn method/' core/engine/src/builtins/builder.rs
sed -i '327s/pub(crate) fn accessor/pub fn accessor/' core/engine/src/builtins/builder.rs
sed -i '505s/pub(crate) fn callable_with_intrinsic/pub fn callable_with_intrinsic/' core/engine/src/builtins/builder.rs
sed -i '545s/pub(crate) fn from_standard_constructor/pub fn from_standard_constructor/' core/engine/src/builtins/builder.rs
sed -i '596s/pub(crate) fn static_property/pub fn static_property/' core/engine/src/builtins/builder.rs

# array/mod.rs
sed -i '393s/pub(crate) fn create_array_from_list/pub fn create_array_from_list/' core/engine/src/builtins/array/mod.rs

# jspromise.rs
sed -i 's/pub struct JsPromise {/pub struct JsPromise {/' core/engine/src/object/builtins/jspromise.rs

echo "Made all methods public"
