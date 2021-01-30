//! Common types for GDEF, GPOS and GSUB tables.

use crate::GlyphId;
use crate::{parser::*, Tag};

#[derive(Clone, Copy)]
struct RangeRecord {
    start_glyph_id: GlyphId,
    end_glyph_id: GlyphId,
    value: u16,
}

impl RangeRecord {
    fn range(&self) -> core::ops::RangeInclusive<GlyphId> {
        self.start_glyph_id..=self.end_glyph_id
    }
}

impl FromData for RangeRecord {
    const SIZE: usize = 6;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(RangeRecord {
            start_glyph_id: s.read::<GlyphId>()?,
            end_glyph_id: s.read::<GlyphId>()?,
            value: s.read::<u16>()?,
        })
    }
}

/// A [Coverage Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-table).
#[derive(Clone, Copy, Debug)]
pub(crate) struct CoverageTable<'a> {
    data: &'a [u8],
}

impl<'a> CoverageTable<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        CoverageTable { data }
    }

    pub fn contains(&self, glyph_id: GlyphId) -> bool {
        let mut s = Stream::new(self.data);
        let format: u16 = try_opt_or!(s.read(), false);

        match format {
            1 => {
                let count = try_opt_or!(s.read::<u16>(), false);
                s.read_array16::<GlyphId>(count)
                    .unwrap()
                    .binary_search(&glyph_id)
                    .is_some()
            }
            2 => {
                let count = try_opt_or!(s.read::<u16>(), false);
                let records = try_opt_or!(s.read_array16::<RangeRecord>(count), false);
                records.into_iter().any(|r| r.range().contains(&glyph_id))
            }
            _ => false,
        }
    }
}

/// A value of [Class Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Class(pub u16);

impl FromData for Class {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Class)
    }
}

/// A [Class Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table).
#[derive(Clone, Copy)]
pub(crate) struct ClassDefinitionTable<'a> {
    data: &'a [u8],
}

impl<'a> ClassDefinitionTable<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        ClassDefinitionTable { data }
    }

    /// Any glyph not included in the range of covered glyph IDs automatically belongs to Class 0.
    pub fn get(&self, glyph_id: GlyphId) -> Class {
        self.get_impl(glyph_id).unwrap_or(Class(0))
    }

    fn get_impl(&self, glyph_id: GlyphId) -> Option<Class> {
        let mut s = Stream::new(self.data);
        let format: u16 = s.read()?;
        match format {
            1 => {
                let start_glyph_id: GlyphId = s.read()?;

                // Prevent overflow.
                if glyph_id < start_glyph_id {
                    return None;
                }

                let count: u16 = s.read()?;
                let classes = s.read_array16::<Class>(count)?;
                classes.get(glyph_id.0 - start_glyph_id.0)
            }
            2 => {
                let count: u16 = s.read()?;
                let records = s.read_array16::<RangeRecord>(count)?;
                records
                    .into_iter()
                    .find(|r| r.range().contains(&glyph_id))
                    .map(|record| Class(record.value))
            }
            _ => None,
        }
    }
}

/// A [Script List Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#script-list-table-and-script-record).
#[derive(Clone, Copy)]
pub(crate) struct ScriptListTable<'a> {
    data: &'a [u8],
    script_records: LazyArray16<'a, ScriptRecord>,
}

impl<'a> ScriptListTable<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read()?;
        Some(Self {
            data,
            script_records: s.read_array16(count)?,
        })
    }
}

/// A [Script Record](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#script-list-table-and-script-record).
#[derive(Clone, Copy)]
pub(crate) struct ScriptRecord {
    script_tag: Tag,
    script_offset: Offset16,
}

impl FromData for ScriptRecord {
    const SIZE: usize = 6;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            script_tag: s.read()?,
            script_offset: s.read()?,
        })
    }
}

/// A [Script](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#script-table-and-language-system-record).
#[derive(Clone, Copy)]
pub(crate) struct Script<'a> {
    default_lang_sys_offset: Option<Offset16>,
    lang_sys_records: LazyArray16<'a, LangSysRecord>,
}

impl<'a> Script<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let default_lang_sys_offset = s.read()?;
        let count = s.read()?;
        Some(Self {
            default_lang_sys_offset,
            lang_sys_records: s.read_array16(count)?,
        })
    }
}

/// A [Language System Record](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#script-table-and-language-system-record).
#[derive(Clone, Copy)]
pub(crate) struct LangSysRecord {
    lang_sys_tag: Tag,
    lang_sys_offset: Offset16,
}

impl FromData for LangSysRecord {
    const SIZE: usize = 6;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            lang_sys_tag: s.read()?,
            lang_sys_offset: s.read()?,
        })
    }
}

/// A [Language System Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#language-system-table).
#[derive(Clone, Copy)]
pub(crate) struct LangSysTable<'a> {
    required_feature_index: Option<u16>,
    feature_indices: LazyArray16<'a, u16>,
}

impl<'a> LangSysTable<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        // This is reserved and always NULL.
        let lookup_order_offset: Option<Offset16> = s.read()?;
        if lookup_order_offset.is_some() {
            return None;
        }
        let required_feature_index = match s.read()? {
            0xFFFF => None,
            index => Some(index),
        };
        let count = s.read()?;
        Some(Self {
            required_feature_index,
            feature_indices: s.read_array16(count)?,
        })
    }
}

/// A [Feature List Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#feature-list-table).
#[derive(Clone, Copy)]
pub(crate) struct FeatureListTable<'a> {
    feature_records: LazyArray16<'a, FeatureRecord>,
}

impl<'a> FeatureListTable<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read()?;
        Some(Self {
            feature_records: s.read_array16(count)?,
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FeatureRecord {
    feature_tag: Tag,
    feature_offset: Offset16,
}

impl FromData for FeatureRecord {
    const SIZE: usize = 6;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            feature_tag: s.read()?,
            feature_offset: s.read()?,
        })
    }
}

/// A [Feature Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#feature-table).
#[derive(Clone, Copy)]
pub(crate) struct FeatureTable<'a> {
    feature_params_offset: Option<Offset16>,
    lookup_list_indices: LazyArray16<'a, u16>,
}

impl<'a> FeatureTable<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let feature_params_offset = s.read()?;
        let count = s.read()?;
        Some(Self {
            feature_params_offset,
            lookup_list_indices: s.read_array16(count)?,
        })
    }
}

/// A [Lookup List Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#lookup-list-table).
#[derive(Clone, Copy)]
pub(crate) struct LookupListTable<'a> {
    lookup_offsets: LazyArray16<'a, Offset16>,
}

impl<'a> LookupListTable<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read()?;
        Some(Self {
            lookup_offsets: s.read_array16(count)?,
        })
    }
}

/// A [Lookup Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#lookup-table).
#[derive(Clone, Copy)]
pub(crate) struct LookupTable<'a> {
    lookup_type: u16,
    lookup_flag: u16,
    subtable_offsets: LazyArray16<'a, Offset16>,
    mark_filtering_set: u16,
}

impl<'a> LookupTable<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let lookup_type = s.read()?;
        let lookup_flag = s.read()?;
        let count = s.read()?;
        let subtable_offsets = s.read_array16(count)?;
        let mark_filtering_set = s.read()?;
        Some(Self {
            lookup_type,
            lookup_flag,
            subtable_offsets,
            mark_filtering_set,
        })
    }
}

#[cfg(feature = "variable-fonts")]
/// A [Feature Variations Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#featurevariations-table).
#[derive(Clone, Copy)]
pub(crate) struct FeatureVariations<'a> {
    feature_variation_records: LazyArray32<'a, FeatureVariationRecord>,
}

#[cfg(feature = "variable-fonts")]
impl<'a> FeatureVariations<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let version: u32 = s.read()?;
        if version != 0x00010000 {
            return None;
        }
        let count = s.read()?;
        Some(Self {
            feature_variation_records: s.read_array32(count)?,
        })
    }
}

#[cfg(feature = "variable-fonts")]
/// A [Feature Variation Record](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#featurevariations-table).
#[derive(Clone, Copy)]
pub(crate) struct FeatureVariationRecord {
    condition_set_offset: Offset32,
    feature_table_substitution_offset: Offset32,
}

#[cfg(feature = "variable-fonts")]
impl FromData for FeatureVariationRecord {
    const SIZE: usize = 6;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            condition_set_offset: s.read()?,
            feature_table_substitution_offset: s.read()?,
        })
    }
}
