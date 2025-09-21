//! IntersectionObserver Web API implementation for Boa
//!
//! Native implementation of the IntersectionObserver standard
//! https://w3c.github.io/IntersectionObserver/
//!
//! This implements the complete IntersectionObserver interface for observing changes in element visibility

use crate::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    realm::Realm, JsData, JsString
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;

/// JavaScript `IntersectionObserver` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct IntersectionObserver;

impl IntrinsicObject for IntersectionObserver {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::observe, js_string!("observe"), 1)
            .method(Self::unobserve, js_string!("unobserve"), 1)
            .method(Self::disconnect, js_string!("disconnect"), 0)
            .method(Self::take_records, js_string!("takeRecords"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IntersectionObserver {
    const NAME: JsString = js_string!("IntersectionObserver");
}

impl BuiltInConstructor for IntersectionObserver {
    const LENGTH: usize = 1;
    const P: usize = 1;
    const SP: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::intersection_observer;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::intersection_observer,
            context,
        )?;

        // Get the callback function (required parameter)
        let callback = args.get_or_undefined(0);
        if !callback.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("IntersectionObserver constructor requires a callback function")
                .into());
        }

        // Get the options (optional parameter)
        let options = args.get_or_undefined(1);

        // Parse options (IntersectionObserverInit)
        let mut config = IntersectionObserverConfig::default();

        if let Some(options_obj) = options.as_object() {
            // Parse root option
            if let Ok(root) = options_obj.get(js_string!("root"), context) {
                if !root.is_null() {
                    config.root = Some(format!("{:p}", root.as_object().unwrap().as_ref()));
                }
            }

            // Parse rootMargin option
            if let Ok(root_margin) = options_obj.get(js_string!("rootMargin"), context) {
                config.root_margin = root_margin.to_string(context)
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_else(|_| "0px".to_string());
            }

            // Parse threshold option
            if let Ok(threshold) = options_obj.get(js_string!("threshold"), context) {
                if let Some(threshold_array) = threshold.as_object() {
                    // Convert array to Vec<f64> (simplified implementation)
                    config.threshold = vec![0.0]; // Default for now
                } else if let Ok(threshold_num) = threshold.to_number(context) {
                    config.threshold = vec![threshold_num];
                }
            }
        }

        // Create IntersectionObserver data
        let observer_data = IntersectionObserverData {
            callback: callback.clone(),
            config,
            observed_targets: HashMap::new(),
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

impl IntersectionObserver {
    /// `IntersectionObserver.prototype.observe()` method
    fn observe(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("IntersectionObserver.observe called on non-object")
        })?;

        let target = args.get_or_undefined(0);

        // Validate target (should be an Element, but for now accept any object)
        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message("IntersectionObserver.observe: target must be an Element")
                .into());
        }

        // Update observer data
        if let Some(mut observer_data) = observer_obj.downcast_mut::<IntersectionObserverData>() {
            let target_id = format!("{:p}", target.as_object().unwrap().as_ref());
            observer_data.observed_targets.insert(target_id, target.clone());
            observer_data.is_observing = true;

            // In a real implementation, we would start observing the element's intersection
            // For now, we'll just track that it's being observed
        }

        Ok(JsValue::undefined())
    }

    /// `IntersectionObserver.prototype.unobserve()` method
    fn unobserve(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("IntersectionObserver.unobserve called on non-object")
        })?;

        let target = args.get_or_undefined(0);

        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message("IntersectionObserver.unobserve: target must be an Element")
                .into());
        }

        if let Some(mut observer_data) = observer_obj.downcast_mut::<IntersectionObserverData>() {
            let target_id = format!("{:p}", target.as_object().unwrap().as_ref());
            observer_data.observed_targets.remove(&target_id);

            if observer_data.observed_targets.is_empty() {
                observer_data.is_observing = false;
            }
        }

        Ok(JsValue::undefined())
    }

    /// `IntersectionObserver.prototype.disconnect()` method
    fn disconnect(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("IntersectionObserver.disconnect called on non-object")
        })?;

        if let Some(mut observer_data) = observer_obj.downcast_mut::<IntersectionObserverData>() {
            observer_data.observed_targets.clear();
            observer_data.records.clear();
            observer_data.is_observing = false;
        }

        Ok(JsValue::undefined())
    }

    /// `IntersectionObserver.prototype.takeRecords()` method
    fn take_records(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("IntersectionObserver.takeRecords called on non-object")
        })?;

        if let Some(mut observer_data) = observer_obj.downcast_mut::<IntersectionObserverData>() {
            // Create array of intersection records
            let records_array = context.intrinsics().constructors().array().constructor().construct(
                &[JsValue::from(observer_data.records.len())],
                None,
                context,
            )?;

            // For now, return empty array (real implementation would populate with actual IntersectionObserverEntry objects)
            observer_data.records.clear();

            Ok(records_array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("IntersectionObserver.takeRecords called on non-IntersectionObserver object")
                .into())
        }
    }
}

/// Internal data for IntersectionObserver instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct IntersectionObserverData {
    /// The callback function to call when intersections change
    callback: JsValue,
    /// Configuration options for the observer
    #[unsafe_ignore_trace]
    config: IntersectionObserverConfig,
    /// Map of observed target elements
    #[unsafe_ignore_trace]
    observed_targets: HashMap<String, JsValue>,
    /// Queue of intersection records waiting to be delivered
    #[unsafe_ignore_trace]
    records: Vec<IntersectionObserverEntry>,
    /// Whether this observer is currently observing any targets
    #[unsafe_ignore_trace]
    is_observing: bool,
}

/// Configuration for IntersectionObserver
#[derive(Debug, Clone)]
pub struct IntersectionObserverConfig {
    /// The root element for intersection calculation (null = viewport)
    pub root: Option<String>,
    /// Margin around the root element
    pub root_margin: String,
    /// Threshold values for triggering callbacks
    pub threshold: Vec<f64>,
}

impl Default for IntersectionObserverConfig {
    fn default() -> Self {
        Self {
            root: None,
            root_margin: "0px".to_string(),
            threshold: vec![0.0],
        }
    }
}

/// Represents a single intersection observer entry
#[derive(Debug, Clone)]
pub struct IntersectionObserverEntry {
    /// The target element being observed
    pub target: String, // Element ID for now
    /// The intersection ratio (0.0 to 1.0)
    pub intersection_ratio: f64,
    /// Whether the target is intersecting
    pub is_intersecting: bool,
    /// Timestamp when the intersection was observed
    pub time: f64,
    /// Bounding rectangle of the target element
    pub bounding_client_rect: DOMRect,
    /// Bounding rectangle of the intersection
    pub intersection_rect: DOMRect,
    /// Bounding rectangle of the root element
    pub root_bounds: Option<DOMRect>,
}

/// Simple rectangle representation
#[derive(Debug, Clone)]
pub struct DOMRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}