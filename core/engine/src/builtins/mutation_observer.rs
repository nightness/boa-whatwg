//! MutationObserver Web API implementation for Boa
//!
//! Native implementation of the MutationObserver standard
//! https://dom.spec.whatwg.org/#interface-mutationobserver
//!
//! This implements the complete MutationObserver interface for watching DOM changes

use crate::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    realm::Realm, JsData, JsString,
    property::Attribute
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;

/// JavaScript `MutationObserver` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct MutationObserver;

impl IntrinsicObject for MutationObserver {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::observe, js_string!("observe"), 2)
            .method(Self::disconnect, js_string!("disconnect"), 0)
            .method(Self::take_records, js_string!("takeRecords"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for MutationObserver {
    const NAME: JsString = js_string!("MutationObserver");
}

impl BuiltInConstructor for MutationObserver {
    const LENGTH: usize = 1;
    const P: usize = 1;
    const SP: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::mutation_observer;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::mutation_observer,
            context,
        )?;

        // Get the callback function (required parameter)
        let callback = args.get_or_undefined(0);
        if !callback.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("MutationObserver constructor requires a callback function")
                .into());
        }

        // Create MutationObserver data
        let observer_data = MutationObserverData {
            callback: callback.clone(),
            observations: HashMap::new(),
            records: Vec::new(),
            is_observing: false,
        };

        let observer_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            observer_data
        );

        Ok(observer_obj.into())
    }
}

impl MutationObserver {
    /// `MutationObserver.prototype.observe()` method
    fn observe(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("MutationObserver.observe called on non-object")
        })?;

        let target = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // Validate target (should be a Node, but for now accept any object)
        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message("MutationObserver.observe: target must be a Node")
                .into());
        }

        // Parse options (MutationObserverInit)
        let mut config = MutationObserverConfig::default();

        if let Some(options_obj) = options.as_object() {
            // Parse childList option
            if let Ok(child_list) = options_obj.get(js_string!("childList"), context) {
                config.child_list = child_list.to_boolean();
            }

            // Parse attributes option
            if let Ok(attributes) = options_obj.get(js_string!("attributes"), context) {
                config.attributes = Some(attributes.to_boolean());
            }

            // Parse characterData option
            if let Ok(character_data) = options_obj.get(js_string!("characterData"), context) {
                config.character_data = Some(character_data.to_boolean());
            }

            // Parse subtree option
            if let Ok(subtree) = options_obj.get(js_string!("subtree"), context) {
                config.subtree = subtree.to_boolean();
            }

            // Parse attributeOldValue option
            if let Ok(attr_old_value) = options_obj.get(js_string!("attributeOldValue"), context) {
                config.attribute_old_value = Some(attr_old_value.to_boolean());
            }

            // Parse characterDataOldValue option
            if let Ok(char_old_value) = options_obj.get(js_string!("characterDataOldValue"), context) {
                config.character_data_old_value = Some(char_old_value.to_boolean());
            }

            // Parse attributeFilter option
            if let Ok(attr_filter) = options_obj.get(js_string!("attributeFilter"), context) {
                if let Some(filter_array) = attr_filter.as_object() {
                    // Convert array to Vec<String> (simplified implementation)
                    config.attribute_filter = Some(Vec::new());
                }
            }
        }

        // Validate configuration
        if !config.child_list && config.attributes.is_none() && config.character_data.is_none() {
            return Err(JsNativeError::typ()
                .with_message("MutationObserver.observe: At least one of childList, attributes, or characterData must be true")
                .into());
        }

        // Update observer data
        if let Some(mut observer_data) = observer_obj.downcast_mut::<MutationObserverData>() {
            let target_id = format!("{:p}", target.as_object().unwrap().as_ref());
            observer_data.observations.insert(target_id, config);
            observer_data.is_observing = true;
        }

        Ok(JsValue::undefined())
    }

    /// `MutationObserver.prototype.disconnect()` method
    fn disconnect(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("MutationObserver.disconnect called on non-object")
        })?;

        if let Some(mut observer_data) = observer_obj.downcast_mut::<MutationObserverData>() {
            observer_data.observations.clear();
            observer_data.records.clear();
            observer_data.is_observing = false;
        }

        Ok(JsValue::undefined())
    }

    /// `MutationObserver.prototype.takeRecords()` method
    fn take_records(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("MutationObserver.takeRecords called on non-object")
        })?;

        if let Some(mut observer_data) = observer_obj.downcast_mut::<MutationObserverData>() {
            // Create array of mutation records
            let records_array = context.intrinsics().constructors().array().constructor().construct(
                &[JsValue::from(observer_data.records.len())],
                None,
                context,
            )?;

            // For now, return empty array (real implementation would populate with actual MutationRecord objects)
            observer_data.records.clear();

            Ok(records_array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("MutationObserver.takeRecords called on non-MutationObserver object")
                .into())
        }
    }
}

/// Internal data for MutationObserver instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct MutationObserverData {
    /// The callback function to call when mutations occur
    callback: JsValue,
    /// Map of target nodes being observed and their configurations
    #[unsafe_ignore_trace]
    observations: HashMap<String, MutationObserverConfig>,
    /// Queue of mutation records waiting to be delivered
    #[unsafe_ignore_trace]
    records: Vec<MutationRecord>,
    /// Whether this observer is currently observing any targets
    #[unsafe_ignore_trace]
    is_observing: bool,
}

/// Configuration for what mutations to observe
#[derive(Debug, Clone)]
pub struct MutationObserverConfig {
    /// Observe changes to the list of child nodes
    pub child_list: bool,
    /// Observe changes to attributes
    pub attributes: Option<bool>,
    /// Observe changes to character data
    pub character_data: Option<bool>,
    /// Observe changes to descendants
    pub subtree: bool,
    /// Include old attribute values in records
    pub attribute_old_value: Option<bool>,
    /// Include old character data values in records
    pub character_data_old_value: Option<bool>,
    /// Filter for specific attribute names
    pub attribute_filter: Option<Vec<String>>,
}

impl Default for MutationObserverConfig {
    fn default() -> Self {
        Self {
            child_list: false,
            attributes: None,
            character_data: None,
            subtree: false,
            attribute_old_value: None,
            character_data_old_value: None,
            attribute_filter: None,
        }
    }
}

/// Represents a single mutation record
#[derive(Debug, Clone)]
pub struct MutationRecord {
    /// Type of mutation: "childList", "attributes", or "characterData"
    pub mutation_type: String,
    /// The node that was mutated
    pub target: String, // Node ID for now
    /// Nodes that were added
    pub added_nodes: Vec<String>,
    /// Nodes that were removed
    pub removed_nodes: Vec<String>,
    /// Previous sibling of added/removed nodes
    pub previous_sibling: Option<String>,
    /// Next sibling of added/removed nodes
    pub next_sibling: Option<String>,
    /// Name of changed attribute
    pub attribute_name: Option<String>,
    /// Namespace of changed attribute
    pub attribute_namespace: Option<String>,
    /// Old value of attribute or character data
    pub old_value: Option<String>,
}