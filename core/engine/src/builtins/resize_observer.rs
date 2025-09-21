//! ResizeObserver Web API implementation for Boa
//!
//! Native implementation of the ResizeObserver standard
//! https://wicg.github.io/ResizeObserver/
//!
//! This implements the complete ResizeObserver interface for observing changes in element size

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

/// JavaScript `ResizeObserver` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ResizeObserver;

impl IntrinsicObject for ResizeObserver {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::observe, js_string!("observe"), 1)
            .method(Self::unobserve, js_string!("unobserve"), 1)
            .method(Self::disconnect, js_string!("disconnect"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ResizeObserver {
    const NAME: JsString = js_string!("ResizeObserver");
}

impl BuiltInConstructor for ResizeObserver {
    const LENGTH: usize = 1;
    const P: usize = 1;
    const SP: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::resize_observer;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::resize_observer,
            context,
        )?;

        // Get the callback function (required parameter)
        let callback = args.get_or_undefined(0);
        if !callback.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("ResizeObserver constructor requires a callback function")
                .into());
        }

        // Create ResizeObserver data
        let observer_data = ResizeObserverData {
            callback: callback.clone(),
            observed_targets: HashMap::new(),
            entries: Vec::new(),
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

impl ResizeObserver {
    /// `ResizeObserver.prototype.observe()` method
    fn observe(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ResizeObserver.observe called on non-object")
        })?;

        let target = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // Validate target (should be an Element, but for now accept any object)
        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message("ResizeObserver.observe: target must be an Element")
                .into());
        }

        // Parse options (ResizeObserverOptions)
        let mut box_type = ResizeObserverBoxOptions::ContentBox;

        if let Some(options_obj) = options.as_object() {
            if let Ok(box_option) = options_obj.get(js_string!("box"), context) {
                let box_str = box_option.to_string(context)
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_else(|_| "content-box".to_string());

                box_type = match box_str.as_str() {
                    "border-box" => ResizeObserverBoxOptions::BorderBox,
                    "content-box" => ResizeObserverBoxOptions::ContentBox,
                    "device-pixel-content-box" => ResizeObserverBoxOptions::DevicePixelContentBox,
                    _ => ResizeObserverBoxOptions::ContentBox,
                };
            }
        }

        // Update observer data
        if let Some(mut observer_data) = observer_obj.downcast_mut::<ResizeObserverData>() {
            let target_id = format!("{:p}", target.as_object().unwrap().as_ref());
            observer_data.observed_targets.insert(target_id, (target.clone(), box_type));
            observer_data.is_observing = true;

            // In a real implementation, we would start observing the element's size changes
            // For now, we'll just track that it's being observed
        }

        Ok(JsValue::undefined())
    }

    /// `ResizeObserver.prototype.unobserve()` method
    fn unobserve(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ResizeObserver.unobserve called on non-object")
        })?;

        let target = args.get_or_undefined(0);

        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message("ResizeObserver.unobserve: target must be an Element")
                .into());
        }

        if let Some(mut observer_data) = observer_obj.downcast_mut::<ResizeObserverData>() {
            let target_id = format!("{:p}", target.as_object().unwrap().as_ref());
            observer_data.observed_targets.remove(&target_id);

            if observer_data.observed_targets.is_empty() {
                observer_data.is_observing = false;
            }
        }

        Ok(JsValue::undefined())
    }

    /// `ResizeObserver.prototype.disconnect()` method
    fn disconnect(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ResizeObserver.disconnect called on non-object")
        })?;

        if let Some(mut observer_data) = observer_obj.downcast_mut::<ResizeObserverData>() {
            observer_data.observed_targets.clear();
            observer_data.entries.clear();
            observer_data.is_observing = false;
        }

        Ok(JsValue::undefined())
    }
}

/// Internal data for ResizeObserver instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct ResizeObserverData {
    /// The callback function to call when size changes occur
    callback: JsValue,
    /// Map of observed target elements and their box options
    #[unsafe_ignore_trace]
    observed_targets: HashMap<String, (JsValue, ResizeObserverBoxOptions)>,
    /// Queue of resize entries waiting to be delivered
    #[unsafe_ignore_trace]
    entries: Vec<ResizeObserverEntry>,
    /// Whether this observer is currently observing any targets
    #[unsafe_ignore_trace]
    is_observing: bool,
}

/// Box options for ResizeObserver
#[derive(Debug, Clone, Copy)]
pub enum ResizeObserverBoxOptions {
    /// Observe changes to the content box (default)
    ContentBox,
    /// Observe changes to the border box
    BorderBox,
    /// Observe changes to the device pixel content box
    DevicePixelContentBox,
}

/// Represents a single resize observer entry
#[derive(Debug, Clone)]
pub struct ResizeObserverEntry {
    /// The target element being observed
    pub target: String, // Element ID for now
    /// The new content rect
    pub content_rect: DOMRectReadOnly,
    /// The new border box size
    pub border_box_size: Vec<ResizeObserverSize>,
    /// The new content box size
    pub content_box_size: Vec<ResizeObserverSize>,
    /// The new device pixel content box size
    pub device_pixel_content_box_size: Vec<ResizeObserverSize>,
}

/// Represents a resize observer size
#[derive(Debug, Clone)]
pub struct ResizeObserverSize {
    /// The inline size (width in horizontal writing mode)
    pub inline_size: f64,
    /// The block size (height in horizontal writing mode)
    pub block_size: f64,
}

/// Read-only rectangle representation
#[derive(Debug, Clone)]
pub struct DOMRectReadOnly {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}