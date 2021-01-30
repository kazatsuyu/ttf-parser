use crate::{
    parser::{Offset, Offset16, Stream},
    FeatureListTable, FeatureVariations, LookupListTable, ScriptListTable,
};

#[derive(Clone, Copy)]
pub struct Table<'a> {
    script_list_table: ScriptListTable<'a>,
    feature_list_table: FeatureListTable<'a>,
    lookup_list_table: LookupListTable<'a>,
    #[cfg(feature = "variable-fonts")]
    feature_variations: Option<FeatureVariations<'a>>,
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let version: u32 = s.read()?;
        if !(version == 0x00010000 || version == 0x00010001) {
            return None;
        }
        let script_list_offset: Offset16 = s.read()?;
        let feature_list_offset: Offset16 = s.read()?;
        let lookup_list_offset: Offset16 = s.read()?;
        #[cfg(feature = "variable-fonts")]
        let feature_variations_offset: Option<Offset16> = if version > 0x00010000 {
            s.read()?
        } else {
            None
        };
        let script_list_table = ScriptListTable::parse(data.get(script_list_offset.to_usize()..)?)?;
        let feature_list_table =
            FeatureListTable::parse(data.get(feature_list_offset.to_usize()..)?)?;
        let lookup_list_table = LookupListTable::parse(data.get(lookup_list_offset.to_usize()..)?)?;
        #[cfg(feature = "variable-fonts")]
        let feature_variations = if let Some(offset) = feature_variations_offset {
            Some(FeatureVariations::parse(data.get(offset.to_usize()..)?)?)
        } else {
            None
        };
        Some(Self {
            script_list_table,
            feature_list_table,
            lookup_list_table,
            #[cfg(feature = "variable-fonts")]
            feature_variations,
        })
    }
}
