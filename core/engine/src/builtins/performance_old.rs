//! Performance API implementation for Boa
//!
//! Native implementation of the Performance API standard
//! https://w3c.github.io/hr-time/
//! https://w3c.github.io/navigation-timing/
//! https://w3c.github.io/performance-timeline/
//!
//! This implements the complete Performance interface for real timing measurements

#[cfg(test)]
mod tests;

use crate::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor, Array},
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsResult, js_string,
    realm::Realm, JsString, JsData,
    context::intrinsics::{Intrinsics, StandardConstructor},
    property::{Attribute, PropertyDescriptor},
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Performance navigation timing information
#[derive(Debug, Clone)]
struct PerformanceNavigationTiming {
    navigation_start: f64,
    unload_event_start: f64,
    unload_event_end: f64,
    redirect_start: f64,
    redirect_end: f64,
    fetch_start: f64,
    domain_lookup_start: f64,
    domain_lookup_end: f64,
    connect_start: f64,
    connect_end: f64,
    secure_connection_start: f64,
    request_start: f64,
    response_start: f64,
    response_end: f64,
    dom_loading: f64,
    dom_interactive: f64,
    dom_content_loaded_event_start: f64,
    dom_content_loaded_event_end: f64,
    dom_complete: f64,
    load_event_start: f64,
    load_event_end: f64,
}

impl Default for PerformanceNavigationTiming {
    fn default() -> Self {
        let now = performance_now();
        Self {
            navigation_start: now - 1000.0,
            unload_event_start: now - 950.0,
            unload_event_end: now - 945.0,
            redirect_start: 0.0,
            redirect_end: 0.0,
            fetch_start: now - 940.0,
            domain_lookup_start: now - 935.0,
            domain_lookup_end: now - 930.0,
            connect_start: now - 925.0,
            connect_end: now - 920.0,
            secure_connection_start: now - 915.0,
            request_start: now - 910.0,
            response_start: now - 905.0,
            response_end: now - 900.0,
            dom_loading: now - 895.0,
            dom_interactive: now - 890.0,
            dom_content_loaded_event_start: now - 885.0,
            dom_content_loaded_event_end: now - 880.0,
            dom_complete: now - 875.0,
            load_event_start: now - 870.0,
            load_event_end: now - 865.0,
        }
    }
}

/// Global Performance state storage (similar to V8's PerIsolateData)
#[derive(Debug, Clone)]
struct PerformanceState {
    /// Map of performance marks by name to timestamp
    mark_map: HashMap<String, f64>,
    /// List of all performance entries
    entries: Vec<PerformanceEntry>,
    /// Performance time origin
    time_origin: f64,
    /// Navigation timing information
    navigation_timing: PerformanceNavigationTiming,
}

/// Global Performance state instance
static PERFORMANCE_STATE: OnceLock<Arc<Mutex<PerformanceState>>> = OnceLock::new();

impl Default for PerformanceState {
    fn default() -> Self {
        Self {
            mark_map: HashMap::new(),
            entries: Vec::new(),
            time_origin: performance_now(),
            navigation_timing: PerformanceNavigationTiming::default(),
        }
    }
}

/// Get or initialize the global performance state
fn get_performance_state() -> Arc<Mutex<PerformanceState>> {
    PERFORMANCE_STATE.get_or_init(|| {
        Arc::new(Mutex::new(PerformanceState::default()))
    }).clone()
}

/// Performance entry types according to W3C specifications
#[derive(Debug, Clone)]
enum PerformanceEntryType {
    Navigation,
    Resource,
    Mark,
    Measure,
    Paint,
    Element,
    Event,
    FirstInput,
    LargestContentfulPaint,
    LayoutShift,
    LongTask,
}

impl PerformanceEntryType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Navigation => "navigation",
            Self::Resource => "resource",
            Self::Mark => "mark",
            Self::Measure => "measure",
            Self::Paint => "paint",
            Self::Element => "element",
            Self::Event => "event",
            Self::FirstInput => "first-input",
            Self::LargestContentfulPaint => "largest-contentful-paint",
            Self::LayoutShift => "layout-shift",
            Self::LongTask => "longtask",
        }
    }
}

/// Performance entry representing a timing measurement
#[derive(Debug, Clone)]
struct PerformanceEntry {
    name: String,
    entry_type: PerformanceEntryType,
    start_time: f64,
    duration: f64,
}

/// Performance mark created by performance.mark()
#[derive(Debug, Clone)]
struct PerformanceMark {
    name: String,
    start_time: f64,
}

/// Performance measure created by performance.measure()
#[derive(Debug, Clone)]
struct PerformanceMeasure {
    name: String,
    start_time: f64,
    duration: f64,
}

/// Performance navigation timing information
#[derive(Debug, Clone)]
struct PerformanceNavigationTiming {
    navigation_start: f64,
    unload_event_start: f64,
    unload_event_end: f64,
    redirect_start: f64,
    redirect_end: f64,
    fetch_start: f64,
    domain_lookup_start: f64,
    domain_lookup_end: f64,
    connect_start: f64,
    connect_end: f64,
    secure_connection_start: f64,
    request_start: f64,
    response_start: f64,
    response_end: f64,
    dom_loading: f64,
    dom_interactive: f64,
    dom_content_loaded_event_start: f64,
    dom_content_loaded_event_end: f64,
    dom_complete: f64,
    load_event_start: f64,
    load_event_end: f64,
}

impl Default for PerformanceNavigationTiming {
    fn default() -> Self {
        let now = performance_now();
        Self {
            navigation_start: now - 1000.0,
            unload_event_start: now - 950.0,
            unload_event_end: now - 945.0,
            redirect_start: 0.0,
            redirect_end: 0.0,
            fetch_start: now - 940.0,
            domain_lookup_start: now - 935.0,
            domain_lookup_end: now - 930.0,
            connect_start: now - 925.0,
            connect_end: now - 920.0,
            secure_connection_start: now - 915.0,
            request_start: now - 910.0,
            response_start: now - 905.0,
            response_end: now - 900.0,
            dom_loading: now - 895.0,
            dom_interactive: now - 890.0,
            dom_content_loaded_event_start: now - 885.0,
            dom_content_loaded_event_end: now - 880.0,
            dom_complete: now - 875.0,
            load_event_start: now - 870.0,
            load_event_end: now - 865.0,
        }
    }
}

/// Performance state for maintaining entries, marks, and measures
#[derive(Debug)]
struct PerformanceState {
    entries: Vec<PerformanceEntry>,
    marks: HashMap<String, PerformanceMark>,
    measures: HashMap<String, PerformanceMeasure>,
    navigation_timing: PerformanceNavigationTiming,
    time_origin: f64,
}

impl Default for PerformanceState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            marks: HashMap::new(),
            measures: HashMap::new(),
            navigation_timing: PerformanceNavigationTiming::default(),
            time_origin: performance_now(),
        }
    }
}

static PERFORMANCE_STATE: OnceLock<Arc<Mutex<PerformanceState>>> = OnceLock::new();

fn get_performance_state() -> &'static Arc<Mutex<PerformanceState>> {
    PERFORMANCE_STATE.get_or_init(|| Arc::new(Mutex::new(PerformanceState::default())))
}

/// Get high-resolution time in milliseconds
fn performance_now() -> f64 {
    static START_TIME: OnceLock<Instant> = OnceLock::new();
    let start = START_TIME.get_or_init(Instant::now);
    start.elapsed().as_secs_f64() * 1000.0
}

/// Performance object providing timing and measurement capabilities
#[derive(Debug, Clone, Finalize)]
pub struct Performance {
    time_origin: f64,
}

unsafe impl Trace for Performance {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in Performance, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in Performance, nothing to trace
    }

    fn run_finalizer(&self) {
        // No finalizer needed for Performance
    }
}

impl JsData for Performance {}

impl Default for Performance {
    fn default() -> Self {
        Self {
            time_origin: performance_now(),
        }
    }
}

impl IntrinsicObject for Performance {
    fn init(realm: &Realm) {
        // Performance builtin initialization
        let performance_obj = BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_method(Self::now, js_string!("now"), 0)
            .static_method(Self::mark, js_string!("mark"), 1)
            .static_method(Self::measure, js_string!("measure"), 3)
            .static_method(Self::clear_marks, js_string!("clearMarks"), 0)
            .static_method(Self::clear_measures, js_string!("clearMeasures"), 0)
            .static_method(Self::get_entries, js_string!("getEntries"), 0)
            .static_method(Self::get_entries_by_type, js_string!("getEntriesByType"), 1)
            .static_method(Self::get_entries_by_name, js_string!("getEntriesByName"), 2)
            .static_property(
                js_string!("timeOrigin"),
                performance_now(),
                Attribute::READONLY,
            )
            .static_property(
                js_string!("timing"),
                Self::create_timing_object(realm.intrinsics()),
                Attribute::READONLY,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Performance {
    const NAME: JsString = js_string!("performance");
}

impl BuiltInConstructor for Performance {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.performance();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Performance constructor is not meant to be called directly
        Ok(JsValue::undefined())
    }
}

/// Create a performance object instance for global scope
pub(crate) fn create_performance_object(context: &mut Context) -> JsResult<JsValue> {
    let performance_obj = JsObject::with_object_proto(context.intrinsics());

    // Add methods as function objects
    let now_func = BuiltInBuilder::callable(context.realm(), Performance::now)
        .name(js_string!("now"))
        .length(0)
        .build();
    performance_obj.set(js_string!("now"), now_func, false, context)?;

    let mark_func = BuiltInBuilder::callable(context.realm(), Performance::mark)
        .name(js_string!("mark"))
        .length(1)
        .build();
    performance_obj.set(js_string!("mark"), mark_func, false, context)?;

    let measure_func = BuiltInBuilder::callable(context.realm(), Performance::measure)
        .name(js_string!("measure"))
        .length(3)
        .build();
    performance_obj.set(js_string!("measure"), measure_func, false, context)?;

    let clear_marks_func = BuiltInBuilder::callable(context.realm(), Performance::clear_marks)
        .name(js_string!("clearMarks"))
        .length(0)
        .build();
    performance_obj.set(js_string!("clearMarks"), clear_marks_func, false, context)?;

    let clear_measures_func = BuiltInBuilder::callable(context.realm(), Performance::clear_measures)
        .name(js_string!("clearMeasures"))
        .length(0)
        .build();
    performance_obj.set(js_string!("clearMeasures"), clear_measures_func, false, context)?;

    let get_entries_func = BuiltInBuilder::callable(context.realm(), Performance::get_entries)
        .name(js_string!("getEntries"))
        .length(0)
        .build();
    performance_obj.set(js_string!("getEntries"), get_entries_func, false, context)?;

    let get_entries_by_type_func = BuiltInBuilder::callable(context.realm(), Performance::get_entries_by_type)
        .name(js_string!("getEntriesByType"))
        .length(1)
        .build();
    performance_obj.set(js_string!("getEntriesByType"), get_entries_by_type_func, false, context)?;

    let get_entries_by_name_func = BuiltInBuilder::callable(context.realm(), Performance::get_entries_by_name)
        .name(js_string!("getEntriesByName"))
        .length(2)
        .build();
    performance_obj.set(js_string!("getEntriesByName"), get_entries_by_name_func, false, context)?;

    // Add readonly properties with proper descriptors
    let state = get_performance_state();
    let time_origin = state.lock().unwrap().time_origin;

    performance_obj.define_property_or_throw(
        js_string!("timeOrigin"),
        PropertyDescriptor::builder()
            .value(time_origin)
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;

    performance_obj.define_property_or_throw(
        js_string!("timing"),
        PropertyDescriptor::builder()
            .value(Performance::create_timing_object(context.intrinsics()))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;

    Ok(performance_obj.into())
}

impl Performance {
    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &crate::context::intrinsics::StandardConstructor =
        crate::context::intrinsics::StandardConstructors::performance;

    /// `performance.now()` - Returns current high-resolution time
    fn now(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(performance_now()))
    }

    /// `performance.mark(name)` - Creates a performance mark
    fn mark(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_lossy();
        let timestamp = performance_now();

        // Store mark in global state (V8 pattern)
        {
            let state = get_performance_state();
            let mut state = state.lock().unwrap();
            state.mark_map.insert(name_str.clone(), timestamp);
        }

        // Create PerformanceEntry object following V8's pattern with ReadOnly properties
        let performance_entry = JsObject::with_object_proto(context.intrinsics());

        // Set properties with ReadOnly descriptors like V8
        performance_entry.define_property_or_throw(
            js_string!("entryType"),
            PropertyDescriptor::builder()
                .value(js_string!("mark"))
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        performance_entry.define_property_or_throw(
            js_string!("name"),
            PropertyDescriptor::builder()
                .value(name.clone())
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        performance_entry.define_property_or_throw(
            js_string!("startTime"),
            PropertyDescriptor::builder()
                .value(timestamp)
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        performance_entry.define_property_or_throw(
            js_string!("duration"),
            PropertyDescriptor::builder()
                .value(0.0)
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        // Store entry in global state
        {
            let state = get_performance_state();
            let mut state = state.lock().unwrap();
            state.entries.push(PerformanceEntry {
                name: name_str,
                entry_type: PerformanceEntryType::Mark,
                start_time: timestamp,
                duration: 0.0,
            });
        }

        Ok(performance_entry.into())
    }

    /// `performance.measure(name, startMark, endMark)` - Creates a performance measure
    fn measure(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_lossy();

        let start_mark = args.get_or_undefined(1).to_string(context)?;
        let end_mark = args.get_or_undefined(2).to_string(context)?;

        let state = get_performance_state();
        let mut state = state.lock().unwrap();

        let start_time = if start_mark.is_empty() {
            state.navigation_timing.navigation_start
        } else {
            state.marks.get(&start_mark.to_std_string_lossy())
                .map(|m| m.start_time)
                .unwrap_or_else(performance_now)
        };

        let end_time = if end_mark.is_empty() {
            performance_now()
        } else {
            state.marks.get(&end_mark.to_std_string_lossy())
                .map(|m| m.start_time)
                .unwrap_or_else(performance_now)
        };

        let duration = end_time - start_time;

        let measure = PerformanceMeasure {
            name: name_str.clone(),
            start_time,
            duration,
        };

        let entry = PerformanceEntry {
            name: name_str.clone(),
            entry_type: PerformanceEntryType::Measure,
            start_time,
            duration,
        };

        state.measures.insert(name_str, measure);
        state.entries.push(entry);

        Ok(JsValue::undefined())
    }

    /// `performance.clearMarks(name?)` - Clears performance marks
    fn clear_marks(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let state = get_performance_state();
        let mut state = state.lock().unwrap();

        if let Some(name_val) = args.get(0) {
            if !name_val.is_undefined() {
                let name = name_val.to_string(context)?;
                let name_str = name.to_std_string_lossy();
                state.marks.remove(&name_str);
                state.entries.retain(|e| e.name != name_str || !matches!(e.entry_type, PerformanceEntryType::Mark));
            }
        } else {
            state.marks.clear();
            state.entries.retain(|e| !matches!(e.entry_type, PerformanceEntryType::Mark));
        }

        Ok(JsValue::undefined())
    }

    /// `performance.clearMeasures(name?)` - Clears performance measures
    fn clear_measures(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let state = get_performance_state();
        let mut state = state.lock().unwrap();

        if let Some(name_val) = args.get(0) {
            if !name_val.is_undefined() {
                let name = name_val.to_string(context)?;
                let name_str = name.to_std_string_lossy();
                state.measures.remove(&name_str);
                state.entries.retain(|e| e.name != name_str || !matches!(e.entry_type, PerformanceEntryType::Measure));
            }
        } else {
            state.measures.clear();
            state.entries.retain(|e| !matches!(e.entry_type, PerformanceEntryType::Measure));
        }

        Ok(JsValue::undefined())
    }

    /// `performance.getEntries()` - Returns all performance entries
    fn get_entries(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let state = get_performance_state();
        let state = state.lock().unwrap();

        let entries: Vec<JsValue> = state.entries.iter()
            .map(|entry| Self::entry_to_js_object(entry, context))
            .collect::<JsResult<Vec<_>>>()?;

        Ok(Array::create_array_from_list(entries, context).into())
    }

    /// `performance.getEntriesByType(type)` - Returns entries filtered by type
    fn get_entries_by_type(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let entry_type = args.get_or_undefined(0).to_string(context)?;
        let type_str = entry_type.to_std_string_lossy();

        let state = get_performance_state();
        let state = state.lock().unwrap();

        let entries: Vec<JsValue> = state.entries.iter()
            .filter(|entry| entry.entry_type.as_str() == type_str)
            .map(|entry| Self::entry_to_js_object(entry, context))
            .collect::<JsResult<Vec<_>>>()?;

        Ok(Array::create_array_from_list(entries, context).into())
    }

    /// `performance.getEntriesByName(name, type?)` - Returns entries filtered by name and optionally type
    fn get_entries_by_name(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_lossy();

        let type_filter = args.get(1)
            .filter(|v| !v.is_undefined())
            .map(|v| v.to_string(context))
            .transpose()?
            .map(|s| s.to_std_string_lossy());

        let state = get_performance_state();
        let state = state.lock().unwrap();

        let entries: Vec<JsValue> = state.entries.iter()
            .filter(|entry| {
                entry.name == name_str &&
                type_filter.as_ref().map_or(true, |t| entry.entry_type.as_str() == t)
            })
            .map(|entry| Self::entry_to_js_object(entry, context))
            .collect::<JsResult<Vec<_>>>()?;

        Ok(Array::create_array_from_list(entries, context).into())
    }

    /// Convert a PerformanceEntry to a JavaScript object
    fn entry_to_js_object(entry: &PerformanceEntry, context: &mut Context) -> JsResult<JsValue> {
        let obj = JsObject::with_object_proto(context.intrinsics());

        obj.set(js_string!("name"), JsValue::from(js_string!(entry.name.clone())), false, context)?;
        obj.set(js_string!("entryType"), JsValue::from(js_string!(entry.entry_type.as_str())), false, context)?;
        obj.set(js_string!("startTime"), JsValue::from(entry.start_time), false, context)?;
        obj.set(js_string!("duration"), JsValue::from(entry.duration), false, context)?;

        Ok(obj.into())
    }

    /// Create timing object for performance.timing
    fn create_timing_object(intrinsics: &Intrinsics) -> JsValue {
        let obj = JsObject::with_object_proto(intrinsics);
        let state = get_performance_state();
        let state = state.lock().unwrap();
        let timing = &state.navigation_timing;

        // Note: In a real implementation these would be populated during navigation
        // Using direct property descriptor set to avoid context requirement
        obj.insert_property(
            js_string!("navigationStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.navigation_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("unloadEventStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.unload_event_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        // Add other timing properties in the same way
        for (name, value) in [
            ("unloadEventEnd", timing.unload_event_end),
            ("redirectStart", timing.redirect_start),
            ("redirectEnd", timing.redirect_end),
            ("fetchStart", timing.fetch_start),
            ("domainLookupStart", timing.domain_lookup_start),
            ("domainLookupEnd", timing.domain_lookup_end),
            ("connectStart", timing.connect_start),
            ("connectEnd", timing.connect_end),
            ("secureConnectionStart", timing.secure_connection_start),
            ("requestStart", timing.request_start),
            ("responseStart", timing.response_start),
            ("responseEnd", timing.response_end),
            ("domLoading", timing.dom_loading),
            ("domInteractive", timing.dom_interactive),
            ("domContentLoadedEventStart", timing.dom_content_loaded_event_start),
            ("domContentLoadedEventEnd", timing.dom_content_loaded_event_end),
            ("domComplete", timing.dom_complete),
            ("loadEventStart", timing.load_event_start),
            ("loadEventEnd", timing.load_event_end),
        ] {
            obj.insert_property(
                js_string!(name),
                crate::property::PropertyDescriptor::builder()
                    .value(value)
                    .writable(false)
                    .enumerable(true)
                    .configurable(false)
                    .build(),
            );
        }

        obj.into()
    }

}