#!/bin/bash

# Make browser API exports public for external browser-apis crate

echo "Making browser API exports public in Boa..."

cd core/engine/src

# Make Array public
sed -i 's/pub(crate) struct Array;/pub struct Array;/g' builtins/array/mod.rs

# Make Promise and JsPromise public
sed -i 's/pub(crate) struct JsPromise/pub struct JsPromise/g' object/builtins/jspromise.rs
sed -i 's/pub(crate) struct PromiseCapability/pub struct PromiseCapability/g' builtins/promise/mod.rs

# Make get_prototype_from_constructor public
sed -i 's/pub(crate) fn get_prototype_from_constructor/pub fn get_prototype_from_constructor/g' object/internal_methods/mod.rs

# Make Json public
sed -i 's/pub(crate) struct Json;/pub struct Json;/g' builtins/json/mod.rs

# Make ReadableStreamData and StreamState public
sed -i 's/pub(crate) struct ReadableStreamData/pub struct ReadableStreamData/g' builtins/readable_stream/mod.rs
sed -i 's/pub(crate) enum StreamState/pub enum StreamState/g' builtins/readable_stream/mod.rs

# Make Selection public
sed -i 's/pub(crate) struct Selection/pub struct Selection/g' builtins/selection.rs

# Make ServiceWorkerContainer public
sed -i 's/pub(crate) struct ServiceWorkerContainer/pub struct ServiceWorkerContainer/g' builtins/service_worker_container.rs

# Make file system functions public
sed -i 's/pub(crate) fn show_open_file_picker/pub fn show_open_file_picker/g' builtins/file_system.rs
sed -i 's/pub(crate) fn show_save_file_picker/pub fn show_save_file_picker/g' builtins/file_system.rs
sed -i 's/pub(crate) fn show_directory_picker/pub fn show_directory_picker/g' builtins/file_system.rs

echo "Browser API exports made public!"
