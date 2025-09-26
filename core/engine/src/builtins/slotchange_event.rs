//! SlotChange event system for Shadow DOM
//!
//! Implementation of slotchange events according to WHATWG DOM specification
//! https://dom.spec.whatwg.org/#mutation-algorithms

use crate::{
    builtins::{
        element::ElementData,
        html_slot_element::HTMLSlotElementData,
        shadow_root::ShadowRootData,
        event::EventData,
        custom_event::CustomEventData,
    },
    object::JsObject,
    value::JsValue,
    Context, JsResult, JsNativeError,
    js_string,
};
use boa_gc::{Finalize, Trace, GcRefCell};
use std::collections::{HashMap, HashSet, VecDeque};

/// SlotChange event system for managing slot assignment changes
pub struct SlotChangeEventSystem;

impl SlotChangeEventSystem {
    /// Fire slotchange event on affected slots after slot assignment changes
    /// This implements the WHATWG algorithm for signaling slot changes
    pub fn signal_slot_change(slot: &JsObject, context: &mut Context) -> JsResult<()> {
        if let Some(slot_data) = slot.downcast_ref::<HTMLSlotElementData>() {
            // Check if the slot is connected (has a root)
            if !Self::is_slot_connected(slot) {
                return Ok(());
            }

            // Queue slotchange event for this slot
            Self::queue_slotchange_event(slot.clone(), context)?;
        }

        Ok(())
    }

    /// Queue slotchange event to be dispatched later
    /// Events are queued and dispatched in microtasks to batch changes
    fn queue_slotchange_event(slot: JsObject, context: &mut Context) -> JsResult<()> {
        // Get or create the slotchange event queue
        let mut queue = Self::get_slotchange_event_queue(context);

        // Add slot to queue if not already present
        if !queue.contains(&slot) {
            queue.push_back(slot);
        }

        // Store updated queue
        Self::set_slotchange_event_queue(context, queue);

        // Schedule microtask to process queue
        Self::schedule_slotchange_processing(context);

        Ok(())
    }

    /// Process queued slotchange events
    pub fn process_slotchange_event_queue(context: &mut Context) -> JsResult<()> {
        let mut queue = Self::get_slotchange_event_queue(context);

        while let Some(slot) = queue.pop_front() {
            Self::dispatch_slotchange_event(&slot, context)?;
        }

        // Clear the queue
        Self::set_slotchange_event_queue(context, VecDeque::new());

        Ok(())
    }

    /// Dispatch slotchange event on a slot element
    fn dispatch_slotchange_event(slot: &JsObject, context: &mut Context) -> JsResult<()> {
        // Create slotchange event
        let event = Self::create_slotchange_event(context)?;

        // Event target and current target will be set during dispatch

        // Dispatch event on slot element
        if let Some(element_data) = slot.downcast_ref::<ElementData>() {
            let event_value = JsValue::from(event);
            element_data.dispatch_event("slotchange", &event_value, context)?;
        }

        Ok(())
    }

    /// Create slotchange event object
    fn create_slotchange_event(context: &mut Context) -> JsResult<JsObject> {
        // Create custom event for slotchange
        let event_data = CustomEventData::new(
            "slotchange".to_string(),
            false, // bubbles
            false, // cancelable
            JsValue::undefined(), // detail
        );

        let realm = context.realm().clone();
        let prototype = realm.intrinsics().constructors().object().prototype();

        let event = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            event_data,
        );

        Ok(event)
    }

    /// Signal slot change for multiple slots (batch processing)
    pub fn signal_slot_list_change(slots: &[JsObject], context: &mut Context) -> JsResult<()> {
        for slot in slots {
            Self::signal_slot_change(slot, context)?;
        }
        Ok(())
    }

    /// Check if a slot is connected to a document
    fn is_slot_connected(slot: &JsObject) -> bool {
        // In a full implementation, this would check if the slot has a root
        // For now, assume slots in shadow roots are connected
        if let Some(element_data) = slot.downcast_ref::<ElementData>() {
            // Check if slot has a parent (simplified check)
            element_data.get_parent_node().is_some()
        } else {
            false
        }
    }

    /// Get slotchange event queue from context
    fn get_slotchange_event_queue(context: &mut Context) -> VecDeque<JsObject> {
        // In a real implementation, this would be stored in the context
        // For now, return empty queue
        VecDeque::new()
    }

    /// Set slotchange event queue in context
    fn set_slotchange_event_queue(context: &mut Context, queue: VecDeque<JsObject>) {
        // In a real implementation, this would store the queue in the context
        // For now, this is a no-op
    }

    /// Schedule slotchange event processing
    fn schedule_slotchange_processing(context: &mut Context) {
        // In a real implementation, this would schedule a microtask
        // For now, process immediately (simplified)
        let _ = Self::process_slotchange_event_queue(context);
    }

    /// Compute new slot assignments and fire slotchange events for changes
    pub fn assign_slottables_and_fire_events(
        shadow_root: &JsObject,
        context: &mut Context,
    ) -> JsResult<()> {
        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            // Get all slots in shadow tree
            let slots = shadow_data.find_slots();

            // Store previous assignments for comparison
            let previous_assignments = Self::capture_current_slot_assignments(&slots);

            // Perform slot assignment algorithm
            Self::perform_slot_assignment(shadow_root, &slots, context)?;

            // Compare with previous assignments and fire events for changes
            Self::fire_slotchange_events_for_assignment_changes(
                &slots,
                &previous_assignments,
                context,
            )?;
        }

        Ok(())
    }

    /// Capture current slot assignments for comparison
    fn capture_current_slot_assignments(slots: &[JsObject]) -> HashMap<JsObject, Vec<JsObject>> {
        let mut assignments = HashMap::new();

        for slot in slots {
            if let Some(slot_data) = slot.downcast_ref::<HTMLSlotElementData>() {
                let assigned = slot_data.get_assigned_nodes();
                assignments.insert(slot.clone(), assigned);
            }
        }

        assignments
    }

    /// Perform slot assignment algorithm
    fn perform_slot_assignment(
        shadow_root: &JsObject,
        slots: &[JsObject],
        context: &mut Context,
    ) -> JsResult<()> {
        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            let slottables = shadow_data.get_slottables();

            // Clear all current assignments
            for slot in slots {
                if let Some(slot_data) = slot.downcast_ref::<HTMLSlotElementData>() {
                    slot_data.clear_assigned_nodes();
                }
            }

            // Assign slottables to appropriate slots
            for slottable in &slottables {
                if let Some(slot) = Self::find_slot_for_slottable(slottable, slots) {
                    if let Some(slot_data) = slot.downcast_ref::<HTMLSlotElementData>() {
                        slot_data.assign_node(slottable.clone());
                    }
                }
            }
        }

        Ok(())
    }

    /// Find appropriate slot for a slottable element
    fn find_slot_for_slottable(slottable: &JsObject, slots: &[JsObject]) -> Option<JsObject> {
        if let Some(element_data) = slottable.downcast_ref::<ElementData>() {
            let slot_attribute = element_data.get_attribute("slot").unwrap_or_default();

            // Look for slot with matching name
            for slot in slots {
                if let Some(slot_data) = slot.downcast_ref::<HTMLSlotElementData>() {
                    let slot_name = slot_data.get_name();

                    if slot_name == slot_attribute {
                        return Some(slot.clone());
                    }
                }
            }

            // If no named slot found and slottable has no slot attribute,
            // assign to default slot (empty name)
            if slot_attribute.is_empty() {
                for slot in slots {
                    if let Some(slot_data) = slot.downcast_ref::<HTMLSlotElementData>() {
                        if slot_data.get_name().is_empty() {
                            return Some(slot.clone());
                        }
                    }
                }
            }
        }

        None
    }

    /// Fire slotchange events for assignment changes
    fn fire_slotchange_events_for_assignment_changes(
        slots: &[JsObject],
        previous_assignments: &HashMap<JsObject, Vec<JsObject>>,
        context: &mut Context,
    ) -> JsResult<()> {
        for slot in slots {
            if let Some(slot_data) = slot.downcast_ref::<HTMLSlotElementData>() {
                let current_assigned = slot_data.get_assigned_nodes();
                let previous_assigned = previous_assignments
                    .get(slot)
                    .cloned()
                    .unwrap_or_default();

                // Check if assignment changed
                if !Self::assignments_are_equal(&current_assigned, &previous_assigned) {
                    Self::signal_slot_change(slot, context)?;
                }
            }
        }

        Ok(())
    }

    /// Check if two slot assignments are equal
    fn assignments_are_equal(a: &[JsObject], b: &[JsObject]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        for (obj_a, obj_b) in a.iter().zip(b.iter()) {
            if !JsObject::equals(obj_a, obj_b) {
                return false;
            }
        }

        true
    }

    /// Handle insertion of slottable elements
    pub fn handle_slottable_insertion(
        slottable: &JsObject,
        shadow_root: &JsObject,
        context: &mut Context,
    ) -> JsResult<()> {
        // Add slottable to shadow root's slottables list
        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            shadow_data.add_slottable(slottable.clone());
        }

        // Recompute slot assignments
        Self::assign_slottables_and_fire_events(shadow_root, context)?;

        Ok(())
    }

    /// Handle removal of slottable elements
    pub fn handle_slottable_removal(
        slottable: &JsObject,
        shadow_root: &JsObject,
        context: &mut Context,
    ) -> JsResult<()> {
        // Remove slottable from shadow root's slottables list
        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            shadow_data.remove_slottable(slottable);
        }

        // Recompute slot assignments
        Self::assign_slottables_and_fire_events(shadow_root, context)?;

        Ok(())
    }

    /// Handle slot attribute changes
    pub fn handle_slot_attribute_change(
        slottable: &JsObject,
        old_value: Option<String>,
        new_value: Option<String>,
        shadow_root: &JsObject,
        context: &mut Context,
    ) -> JsResult<()> {
        // Only process if the attribute actually changed
        if old_value != new_value {
            // Recompute slot assignments
            Self::assign_slottables_and_fire_events(shadow_root, context)?;
        }

        Ok(())
    }

    /// Handle slot name attribute changes
    pub fn handle_slot_name_change(
        slot: &JsObject,
        old_name: Option<String>,
        new_name: Option<String>,
        shadow_root: &JsObject,
        context: &mut Context,
    ) -> JsResult<()> {
        // Update slot's name
        if let Some(slot_data) = slot.downcast_ref::<HTMLSlotElementData>() {
            slot_data.set_name(new_name.clone().unwrap_or_default());
        }

        // Only process if the name actually changed
        if old_name != new_name {
            // Recompute slot assignments
            Self::assign_slottables_and_fire_events(shadow_root, context)?;
        }

        Ok(())
    }

    /// Get flattened assigned nodes for a slot (for rendering)
    pub fn get_flattened_assigned_nodes(slot: &JsObject) -> Vec<JsObject> {
        if let Some(slot_data) = slot.downcast_ref::<HTMLSlotElementData>() {
            let assigned = slot_data.get_assigned_nodes();

            if assigned.is_empty() {
                // Return fallback content (children of slot)
                if let Some(element_data) = slot.downcast_ref::<ElementData>() {
                    element_data.get_children()
                } else {
                    Vec::new()
                }
            } else {
                // Return assigned nodes, recursively flattening nested slots
                let mut flattened = Vec::new();
                for node in assigned {
                    if let Some(_nested_slot_data) = node.downcast_ref::<HTMLSlotElementData>() {
                        // Recursively flatten nested slot
                        let nested_nodes = Self::get_flattened_assigned_nodes(&node);
                        flattened.extend(nested_nodes);
                    } else {
                        flattened.push(node);
                    }
                }
                flattened
            }
        } else {
            Vec::new()
        }
    }
}

/// Slot assignment tracking for event management
pub struct SlotAssignmentTracker {
    /// Map of slots to their previous assigned nodes
    previous_assignments: GcRefCell<HashMap<JsObject, Vec<JsObject>>>,
    /// Set of slots that need slotchange events fired
    pending_events: GcRefCell<HashSet<JsObject>>,
}

impl SlotAssignmentTracker {
    /// Create new slot assignment tracker
    pub fn new() -> Self {
        Self {
            previous_assignments: GcRefCell::new(HashMap::new()),
            pending_events: GcRefCell::new(HashSet::new()),
        }
    }

    /// Update assignments and track changes
    pub fn update_assignment(
        &self,
        slot: &JsObject,
        new_assignment: Vec<JsObject>,
    ) -> bool {
        let mut assignments = self.previous_assignments.borrow_mut();
        let old_assignment = assignments.get(slot).cloned().unwrap_or_default();

        // Check if assignment changed
        let changed = !SlotChangeEventSystem::assignments_are_equal(&new_assignment, &old_assignment);

        if changed {
            assignments.insert(slot.clone(), new_assignment);
            self.pending_events.borrow_mut().insert(slot.clone());
        }

        changed
    }

    /// Get slots with pending slotchange events
    pub fn get_pending_slots(&self) -> Vec<JsObject> {
        self.pending_events.borrow().iter().cloned().collect()
    }

    /// Clear pending events for processed slots
    pub fn clear_pending(&self, slots: &[JsObject]) {
        let mut pending = self.pending_events.borrow_mut();
        for slot in slots {
            pending.remove(slot);
        }
    }

    /// Clear all tracking data
    pub fn clear_all(&self) {
        self.previous_assignments.borrow_mut().clear();
        self.pending_events.borrow_mut().clear();
    }
}

unsafe impl Trace for SlotAssignmentTracker {
    boa_gc::empty_trace!();
}

impl Finalize for SlotAssignmentTracker {}

/// Utilities for slot mutation observation
pub struct SlotMutationObserver;

impl SlotMutationObserver {
    /// Observe slot-related mutations and fire appropriate events
    pub fn observe_slot_mutations(
        shadow_root: &JsObject,
        mutations: &[SlotMutation],
        context: &mut Context,
    ) -> JsResult<()> {
        let mut affected_slots = HashSet::new();

        for mutation in mutations {
            match mutation {
                SlotMutation::SlottableInserted { slottable, .. } => {
                    // Find which slot this affects
                    if let Some(slot) = Self::find_affected_slot(slottable, shadow_root) {
                        affected_slots.insert(slot);
                    }
                }
                SlotMutation::SlottableRemoved { slottable, .. } => {
                    // Find which slot this affects
                    if let Some(slot) = Self::find_affected_slot(slottable, shadow_root) {
                        affected_slots.insert(slot);
                    }
                }
                SlotMutation::SlotAttributeChanged { element, .. } => {
                    // Find slots that might be affected
                    let slots = Self::find_slots_affected_by_attribute_change(element, shadow_root);
                    affected_slots.extend(slots);
                }
                SlotMutation::SlotNameChanged { slot, .. } => {
                    affected_slots.insert(slot.clone());
                }
            }
        }

        // Fire slotchange events for all affected slots
        let slots: Vec<_> = affected_slots.into_iter().collect();
        SlotChangeEventSystem::signal_slot_list_change(&slots, context)?;

        Ok(())
    }

    /// Find slot affected by slottable changes
    fn find_affected_slot(slottable: &JsObject, shadow_root: &JsObject) -> Option<JsObject> {
        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            let slots = shadow_data.find_slots();
            SlotChangeEventSystem::find_slot_for_slottable(slottable, &slots)
        } else {
            None
        }
    }

    /// Find slots affected by attribute changes
    fn find_slots_affected_by_attribute_change(
        element: &JsObject,
        shadow_root: &JsObject,
    ) -> Vec<JsObject> {
        // This would need to check both old and new slot assignments
        // For now, return all slots (conservative approach)
        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            shadow_data.find_slots()
        } else {
            Vec::new()
        }
    }
}

/// Types of slot-related mutations
#[derive(Debug, Clone)]
pub enum SlotMutation {
    /// A slottable element was inserted
    SlottableInserted {
        slottable: JsObject,
        parent: JsObject,
    },
    /// A slottable element was removed
    SlottableRemoved {
        slottable: JsObject,
        previous_parent: JsObject,
    },
    /// A slot attribute was changed on an element
    SlotAttributeChanged {
        element: JsObject,
        old_value: Option<String>,
        new_value: Option<String>,
    },
    /// A slot element's name attribute changed
    SlotNameChanged {
        slot: JsObject,
        old_name: Option<String>,
        new_name: Option<String>,
    },
}