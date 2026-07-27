//! Value-class, temperature, and lifetime selection (Chimera write-time compile).
//!
//! Matches INDEXING_STRATEGY_PROPOSAL FINAL DESIGN §§1 and 3:
//! tiny → inline, medium → micro-pages, large → value log; plus temperature
//! and lifetime zones that feed the background compiler.

use super::ValueLocator;

/// Default max payload size stored **inline** with the locator entry.
pub const DEFAULT_TINY_MAX: usize = 64;

/// Default max payload size packed into a **point micro-page** container.
/// Bodies larger than this go to the large-value log (or chunk path).
pub const DEFAULT_MEDIUM_MAX: usize = 16 * 1024;

/// Physical value class chosen at write / recompile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ValueClass {
    /// Tiny values co-located with the Hydra / locator entry.
    Tiny = 1,
    /// Medium values packed into immutable micro-page containers.
    Medium = 2,
    /// Large values in an append-only extent / value log.
    Large = 3,
}

impl ValueClass {
    /// Wire discriminant.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Parse wire discriminant.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Tiny),
            2 => Some(Self::Medium),
            3 => Some(Self::Large),
            _ => None,
        }
    }

    /// Human-readable label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tiny => "tiny",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

/// Access-pattern / temperature placement (FINAL DESIGN §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TemperatureClass {
    /// Hot random point reads → cache-aligned / replicated extents.
    HotRandom = 1,
    /// Warm mixed → packed micro-pages.
    WarmMixed = 2,
    /// Cold range-heavy scans → key-ordered compressed runs.
    ColdRangeHeavy = 3,
}

impl TemperatureClass {
    /// Wire discriminant.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Parse wire discriminant.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::HotRandom),
            2 => Some(Self::WarmMixed),
            3 => Some(Self::ColdRangeHeavy),
            _ => None,
        }
    }

    /// Human-readable label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HotRandom => "hot_random",
            Self::WarmMixed => "warm_mixed",
            Self::ColdRangeHeavy => "cold_range_heavy",
        }
    }
}

/// Lifetime class for zone placement and GC grouping (FINAL DESIGN §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LifetimeClass {
    /// Short-lived → dedicated append zones (less GC copy of long-lived data).
    ShortLived = 1,
    /// Long-lived → separate stable zones / extents.
    LongLived = 2,
}

impl LifetimeClass {
    /// Wire discriminant.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Parse wire discriminant.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::ShortLived),
            2 => Some(Self::LongLived),
            _ => None,
        }
    }

    /// Human-readable label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ShortLived => "short_lived",
            Self::LongLived => "long_lived",
        }
    }
}

/// Thresholds and overrides for value-class selection.
#[derive(Debug, Clone)]
pub struct ClassifyOptions {
    /// Bodies with `len <= tiny_max` → [`ValueClass::Tiny`].
    pub tiny_max: usize,
    /// Bodies with `tiny_max < len <= medium_max` → [`ValueClass::Medium`].
    pub medium_max: usize,
    /// Force a class (tests / operator override). `None` = size-based.
    pub force_class: Option<ValueClass>,
}

impl Default for ClassifyOptions {
    fn default() -> Self {
        Self {
            tiny_max: DEFAULT_TINY_MAX,
            medium_max: DEFAULT_MEDIUM_MAX,
            force_class: None,
        }
    }
}

/// Classify a payload by size into a physical value class.
pub fn classify_value(len: usize, opts: &ClassifyOptions) -> ValueClass {
    if let Some(c) = opts.force_class {
        return c;
    }
    if len <= opts.tiny_max {
        ValueClass::Tiny
    } else if len <= opts.medium_max {
        ValueClass::Medium
    } else {
        ValueClass::Large
    }
}

/// Observed access + lifetime hints used for placement compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementHints {
    /// Temperature / access pattern.
    pub temperature: TemperatureClass,
    /// Expected lifetime.
    pub lifetime: LifetimeClass,
    /// Whether ordered range scans dominate over point gets.
    pub range_scan_heavy: bool,
}

impl Default for PlacementHints {
    fn default() -> Self {
        Self {
            temperature: TemperatureClass::WarmMixed,
            lifetime: LifetimeClass::LongLived,
            range_scan_heavy: false,
        }
    }
}

/// Choose the initial locator *shape* for a new write (no concrete ids yet).
///
/// Concrete ids/slots are filled by the write path or background compiler.
pub fn initial_locator_kind(
    value_len: usize,
    opts: &ClassifyOptions,
    hints: &PlacementHints,
) -> LocatorKind {
    match classify_value(value_len, opts) {
        ValueClass::Tiny => LocatorKind::Inline,
        ValueClass::Large => LocatorKind::LargeValueLog,
        ValueClass::Medium => {
            if hints.range_scan_heavy
                || matches!(hints.temperature, TemperatureClass::ColdRangeHeavy)
            {
                LocatorKind::ScanExtent
            } else {
                LocatorKind::PointContainer
            }
        }
    }
}

/// Discriminant of a [`super::ValueLocator`] without payload bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LocatorKind {
    /// Memory-resident / hot cache.
    Resident = 1,
    /// Bytes inline with the index entry.
    Inline = 2,
    /// Slot in a point-optimized micro-page container.
    PointContainer = 3,
    /// Range in a scan-optimized key-ordered extent.
    ScanExtent = 4,
    /// Record in the large-value log.
    LargeValueLog = 5,
}

impl LocatorKind {
    /// Wire discriminant.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Parse wire discriminant.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Resident),
            2 => Some(Self::Inline),
            3 => Some(Self::PointContainer),
            4 => Some(Self::ScanExtent),
            5 => Some(Self::LargeValueLog),
            _ => None,
        }
    }

    /// Human-readable label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Resident => "resident",
            Self::Inline => "inline",
            Self::PointContainer => "point_container",
            Self::ScanExtent => "scan_extent",
            Self::LargeValueLog => "large_value_log",
        }
    }

    /// Kind of an existing locator.
    pub fn of(loc: &ValueLocator) -> Self {
        match loc {
            ValueLocator::Resident { .. } => Self::Resident,
            ValueLocator::Inline { .. } => Self::Inline,
            ValueLocator::PointContainer { .. } => Self::PointContainer,
            ValueLocator::ScanExtent { .. } => Self::ScanExtent,
            ValueLocator::LargeValueLog { .. } => Self::LargeValueLog,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_by_size_defaults() {
        let opts = ClassifyOptions::default();
        assert_eq!(classify_value(0, &opts), ValueClass::Tiny);
        assert_eq!(classify_value(64, &opts), ValueClass::Tiny);
        assert_eq!(classify_value(65, &opts), ValueClass::Medium);
        assert_eq!(classify_value(16 * 1024, &opts), ValueClass::Medium);
        assert_eq!(classify_value(16 * 1024 + 1, &opts), ValueClass::Large);
    }

    #[test]
    fn force_class_override() {
        let opts = ClassifyOptions {
            force_class: Some(ValueClass::Large),
            ..Default::default()
        };
        assert_eq!(classify_value(1, &opts), ValueClass::Large);
    }

    #[test]
    fn initial_kind_respects_scan_hint() {
        let opts = ClassifyOptions::default();
        let mut hints = PlacementHints::default();
        assert_eq!(
            initial_locator_kind(200, &opts, &hints),
            LocatorKind::PointContainer
        );
        hints.range_scan_heavy = true;
        assert_eq!(
            initial_locator_kind(200, &opts, &hints),
            LocatorKind::ScanExtent
        );
        assert_eq!(initial_locator_kind(8, &opts, &hints), LocatorKind::Inline);
        assert_eq!(
            initial_locator_kind(32 * 1024, &opts, &hints),
            LocatorKind::LargeValueLog
        );
    }
}
