//! ShadowRoot interface implementation for Shadow DOM
//!
//! The ShadowRoot interface represents the root node of a DOM subtree that is rendered separately from a document's main DOM tree.
//! https://dom.spec.whatwg.org/#interface-shadowroot

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    builtins::document_fragment::DocumentFragmentData,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::{Attribute, PropertyDescriptorBuilder},
    realm::Realm,
    string::{StaticJsStrings, JsString},
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult,
};
use boa_gc::{Finalize, Trace, GcRefCell};
use std::collections::HashMap;

/// Shadow DOM modes
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub enum ShadowRootMode {
    Open,
    Closed,
}

impl ShadowRootMode {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "open" => Some(ShadowRootMode::Open),
            "closed" => Some(ShadowRootMode::Closed),
            _ => None,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            ShadowRootMode::Open => "open",
            ShadowRootMode::Closed => "closed",
        }
    }
}

/// ShadowRoot data structure implementing the Shadow DOM specification
#[derive(Debug, Trace, Finalize, JsData)]
pub struct ShadowRootData {
    // Inherit from DocumentFragment
    fragment_data: DocumentFragmentData,
    // ShadowRoot-specific properties
    mode: ShadowRootMode,
    clonable: bool,
    serializable: bool,
    delegates_focus: bool,
    slottables: GcRefCell<Vec<JsObject>>,
    assigned_slot: GcRefCell<Option<JsObject>>,
    // Reference to host element
    host: GcRefCell<Option<JsObject>>,
    // Custom properties for event retargeting
    event_path_cache: GcRefCell<HashMap<String, Vec<JsObject>>>,
}

impl ShadowRootData {
    /// Create a new ShadowRoot with specified options
    pub fn new(mode: ShadowRootMode, clonable: bool, serializable: bool, delegates_focus: bool) -> Self {
        Self {
            fragment_data: DocumentFragmentData::new(),
            mode,
            clonable,
            serializable,
            delegates_focus,
            slottables: GcRefCell::new(Vec::new()),
            assigned_slot: GcRefCell::new(None),
            host: GcRefCell::new(None),
            event_path_cache: GcRefCell::new(HashMap::new()),
        }
    }

    /// Get the mode of the shadow root
    pub fn mode(&self) -> &ShadowRootMode {
        &self.mode
    }

    /// Check if shadow root is clonable
    pub fn is_clonable(&self) -> bool {
        self.clonable
    }

    /// Check if shadow root is serializable
    pub fn is_serializable(&self) -> bool {
        self.serializable
    }

    /// Check if shadow root delegates focus
    pub fn delegates_focus(&self) -> bool {
        self.delegates_focus
    }

    /// Set the host element for this shadow root
    pub fn set_host(&self, host: JsObject) {
        *self.host.borrow_mut() = Some(host);
    }

    /// Get the host element
    pub fn get_host(&self) -> Option<JsObject> {
        self.host.borrow().clone()
    }

    /// Get the fragment data (for delegation to DocumentFragment methods)
    pub fn fragment_data(&self) -> &DocumentFragmentData {
        &self.fragment_data
    }

    /// Add slottable element
    pub fn add_slottable(&self, slottable: JsObject) {
        self.slottables.borrow_mut().push(slottable);
    }

    /// Remove slottable element
    pub fn remove_slottable(&self, slottable: &JsObject) {
        self.slottables.borrow_mut().retain(|s| !JsObject::equals(s, slottable));
    }

    /// Get all slottables
    pub fn get_slottables(&self) -> Vec<JsObject> {
        self.slottables.borrow().clone()
    }

    /// Set assigned slot
    pub fn set_assigned_slot(&self, slot: Option<JsObject>) {
        *self.assigned_slot.borrow_mut() = slot;
    }

    /// Get assigned slot
    pub fn get_assigned_slot(&self) -> Option<JsObject> {
        self.assigned_slot.borrow().clone()
    }

    /// Find slots in this shadow root (simplified implementation)
    pub fn find_slots(&self) -> Vec<JsObject> {
        // In a full implementation, this would traverse the shadow DOM tree
        // looking for <slot> elements
        Vec::new()
    }

    /// Assign slottables to slots (simplified implementation)
    pub fn assign_slottables(&self) {
        // In a full implementation, this would implement the slotting algorithm
        // as specified in the Shadow DOM spec
    }

    /// Get innerHTML for serialization
    pub fn get_inner_html(&self) -> String {
        if !self.is_serializable() {
            return String::new();
        }
        // Delegate to fragment implementation
        // In a full implementation, this would serialize the shadow tree
        String::new()
    }

    /// Set innerHTML (if allowed)
    pub fn set_inner_html(&self, _html: &str) -> Result<(), String> {
        // In a full implementation, this would parse and set the shadow tree content
        Ok(())
    }

    /// Compute composed path for event retargeting
    pub fn compute_composed_path(&self, target: JsObject, composed: bool) -> Vec<JsObject> {
        // Simplified implementation of event path computation for Shadow DOM
        let mut path = Vec::new();

        // In a full implementation, this would:
        // 1. Start from the event target
        // 2. Walk up the DOM tree, including shadow roots
        // 3. Handle composed flag (whether event crosses shadow boundaries)
        // 4. Handle closed shadow roots (events don't cross their boundaries)

        if let Some(host) = self.get_host() {
            if composed || self.mode() == &ShadowRootMode::Open {
                path.push(target);
                path.push(host);
            } else {
                // For closed shadow roots with non-composed events,
                // the path stops at the shadow root
                path.push(target);
            }
        } else {
            path.push(target);
        }

        path
    }

    /// Retarget an event for Shadow DOM encapsulation
    pub fn retarget_event(&self, original_target: JsObject) -> JsObject {
        // According to Shadow DOM spec, events crossing shadow boundaries
        // are retargeted to the shadow host
        match self.mode() {
            ShadowRootMode::Open => {
                // For open shadow roots, retarget to host if we have one
                self.get_host().unwrap_or(original_target)
            }
            ShadowRootMode::Closed => {
                // For closed shadow roots, always retarget to host to maintain encapsulation
                self.get_host().unwrap_or(original_target)
            }
        }
    }

    /// Check if an event should cross this shadow boundary
    pub fn should_event_cross_boundary(&self, composed: bool) -> bool {
        match self.mode() {
            ShadowRootMode::Open => composed, // Open roots allow composed events to cross
            ShadowRootMode::Closed => false, // Closed roots block all events at boundary
        }
    }

    /// Get related target for focus events (simplified)
    pub fn retarget_related_target(&self, related_target: Option<JsObject>) -> Option<JsObject> {
        // In focus/blur events, related_target also needs retargeting
        if let Some(rt) = related_target {
            Some(self.retarget_event(rt))
        } else {
            None
        }
    }
}

/// JavaScript accessor implementations
impl ShadowRootData {
    /// `ShadowRoot.prototype.mode` getter
    fn get_mode_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ShadowRoot.mode called on non-object")
        })?;

        if let Some(shadow_data) = this_obj.downcast_ref::<ShadowRootData>() {
            Ok(JsValue::from(js_string!(shadow_data.mode().to_string())))
        } else {
            Err(JsNativeError::typ()
                .with_message("ShadowRoot.mode called on non-ShadowRoot object")
                .into())
        }
    }

    /// `ShadowRoot.prototype.clonable` getter
    fn get_clonable_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ShadowRoot.clonable called on non-object")
        })?;

        if let Some(shadow_data) = this_obj.downcast_ref::<ShadowRootData>() {
            Ok(JsValue::from(shadow_data.is_clonable()))
        } else {
            Err(JsNativeError::typ()
                .with_message("ShadowRoot.clonable called on non-ShadowRoot object")
                .into())
        }
    }

    /// `ShadowRoot.prototype.serializable` getter
    fn get_serializable_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ShadowRoot.serializable called on non-object")
        })?;

        if let Some(shadow_data) = this_obj.downcast_ref::<ShadowRootData>() {
            Ok(JsValue::from(shadow_data.is_serializable()))
        } else {
            Err(JsNativeError::typ()
                .with_message("ShadowRoot.serializable called on non-ShadowRoot object")
                .into())
        }
    }

    /// `ShadowRoot.prototype.delegatesFocus` getter
    fn get_delegates_focus_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ShadowRoot.delegatesFocus called on non-object")
        })?;

        if let Some(shadow_data) = this_obj.downcast_ref::<ShadowRootData>() {
            Ok(JsValue::from(shadow_data.delegates_focus()))
        } else {
            Err(JsNativeError::typ()
                .with_message("ShadowRoot.delegatesFocus called on non-ShadowRoot object")
                .into())
        }
    }

    /// `ShadowRoot.prototype.host` getter
    fn get_host_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ShadowRoot.host called on non-object")
        })?;

        if let Some(shadow_data) = this_obj.downcast_ref::<ShadowRootData>() {
            match shadow_data.get_host() {
                Some(host) => Ok(host.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("ShadowRoot.host called on non-ShadowRoot object")
                .into())
        }
    }

    /// `ShadowRoot.prototype.innerHTML` getter
    fn get_inner_html_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ShadowRoot.innerHTML called on non-object")
        })?;

        if let Some(shadow_data) = this_obj.downcast_ref::<ShadowRootData>() {
            let html = shadow_data.get_inner_html();
            Ok(JsValue::from(js_string!(html)))
        } else {
            Err(JsNativeError::typ()
                .with_message("ShadowRoot.innerHTML called on non-ShadowRoot object")
                .into())
        }
    }

    /// `ShadowRoot.prototype.innerHTML` setter
    fn set_inner_html_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ShadowRoot.innerHTML setter called on non-object")
        })?;

        let html_value = args.get_or_undefined(0);
        let html_string = html_value.to_string(context)?;

        if let Some(shadow_data) = this_obj.downcast_ref::<ShadowRootData>() {
            match shadow_data.set_inner_html(&html_string.to_std_string_escaped()) {
                Ok(()) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::error().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("ShadowRoot.innerHTML setter called on non-ShadowRoot object")
                .into())
        }
    }

    /// `ShadowRoot.prototype.getHTML()` method (2025 spec)
    fn get_html(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ShadowRoot.getHTML called on non-object")
        })?;

        if let Some(shadow_data) = this_obj.downcast_ref::<ShadowRootData>() {
            let html = shadow_data.get_inner_html();
            Ok(JsValue::from(js_string!(html)))
        } else {
            Err(JsNativeError::typ()
                .with_message("ShadowRoot.getHTML called on non-ShadowRoot object")
                .into())
        }
    }
}

/// The `ShadowRoot` object
#[derive(Debug, Trace, Finalize)]
pub struct ShadowRoot;

impl ShadowRoot {
    // Static method implementations for BuiltInBuilder
    fn get_mode_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        ShadowRootData::get_mode_accessor(this, args, context)
    }

    fn get_clonable_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        ShadowRootData::get_clonable_accessor(this, args, context)
    }

    fn get_serializable_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        ShadowRootData::get_serializable_accessor(this, args, context)
    }

    fn get_delegates_focus_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        ShadowRootData::get_delegates_focus_accessor(this, args, context)
    }

    fn get_host_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        ShadowRootData::get_host_accessor(this, args, context)
    }

    fn get_inner_html_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        ShadowRootData::get_inner_html_accessor(this, args, context)
    }

    fn set_inner_html_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        ShadowRootData::set_inner_html_accessor(this, args, context)
    }

    fn get_html(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        ShadowRootData::get_html(this, args, context)
    }

    /// Create a new ShadowRoot instance
    pub fn create_shadow_root(
        mode: ShadowRootMode,
        options: &ShadowRootInit,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let shadow_data = ShadowRootData::new(
            mode,
            options.clonable,
            options.serializable,
            options.delegates_focus,
        );

        let shadow_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().shadow_root().prototype(),
            shadow_data,
        );

        Ok(shadow_obj)
    }
}

/// Options for creating a ShadowRoot
#[derive(Debug, Clone)]
pub struct ShadowRootInit {
    pub mode: ShadowRootMode,
    pub clonable: bool,
    pub serializable: bool,
    pub delegates_focus: bool,
}

impl Default for ShadowRootInit {
    fn default() -> Self {
        Self {
            mode: ShadowRootMode::Open,
            clonable: false,
            serializable: false,
            delegates_focus: false,
        }
    }
}

impl IntrinsicObject for ShadowRoot {
    fn init(realm: &Realm) {
        let mode_get_func = BuiltInBuilder::callable(realm, Self::get_mode_accessor)
            .name(js_string!("get mode"))
            .build();
        let clonable_get_func = BuiltInBuilder::callable(realm, Self::get_clonable_accessor)
            .name(js_string!("get clonable"))
            .build();
        let serializable_get_func = BuiltInBuilder::callable(realm, Self::get_serializable_accessor)
            .name(js_string!("get serializable"))
            .build();
        let delegates_focus_get_func = BuiltInBuilder::callable(realm, Self::get_delegates_focus_accessor)
            .name(js_string!("get delegatesFocus"))
            .build();
        let host_get_func = BuiltInBuilder::callable(realm, Self::get_host_accessor)
            .name(js_string!("get host"))
            .build();
        let inner_html_get_func = BuiltInBuilder::callable(realm, Self::get_inner_html_accessor)
            .name(js_string!("get innerHTML"))
            .build();
        let inner_html_set_func = BuiltInBuilder::callable(realm, Self::set_inner_html_accessor)
            .name(js_string!("set innerHTML"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Properties
            .accessor(
                js_string!("mode"),
                Some(mode_get_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("clonable"),
                Some(clonable_get_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("serializable"),
                Some(serializable_get_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("delegatesFocus"),
                Some(delegates_focus_get_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("host"),
                Some(host_get_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("innerHTML"),
                Some(inner_html_get_func),
                Some(inner_html_set_func),
                Attribute::CONFIGURABLE,
            )
            // Methods
            .method(Self::get_html, js_string!("getHTML"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ShadowRoot {
    const NAME: JsString = StaticJsStrings::SHADOW_ROOT;
}

impl BuiltInConstructor for ShadowRoot {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::shadow_root;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // ShadowRoot constructor should not be called directly
        // ShadowRoot instances are created through Element.attachShadow()
        return Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into());
    }
}

#[cfg(test)]
mod tests;