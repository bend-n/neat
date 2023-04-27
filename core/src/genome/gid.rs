use godot::prelude::utilities::randi;
use godot::prelude::*;
use std::fmt::Debug;

#[derive(GodotClass, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[class(base=RefCounted)]
pub struct GenomeId {
    uuid: [u8; 8],
}

#[godot_api]
impl RefCountedVirtual for GenomeId {
    fn init(_base: Base<Self::Base>) -> Self {
        Self::default()
    }

    fn to_string(&self) -> GodotString {
        format!("{self:?}").into()
    }
}

impl Debug for GenomeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GID<{}>", self.uuid_str())
    }
}

macro_rules! rand_u8 {
    () => {
        (randi() & 255) as u8
    };
}

/// dont look at it too hard
fn uuid_bin() -> [u8; 8] {
    [
        rand_u8!(),
        rand_u8!(),
        rand_u8!(),
        rand_u8!(),
        rand_u8!(),
        rand_u8!(),
        rand_u8!(),
        rand_u8!(),
    ]
}

impl Default for GenomeId {
    fn default() -> Self {
        Self { uuid: uuid_bin() }
    }
}

impl GenomeId {
    pub fn uuid_str(&self) -> String {
        format!("{:x}", ((self.uuid[0] as i32) << 8) + self.uuid[1] as i32)
    }

    pub fn full_str(&self) -> String {
        format!("{:x}", i64::from(*self))
    }
}

impl From<GenomeId> for i64 {
    fn from(value: GenomeId) -> Self {
        Self::from_be_bytes(value.uuid)
    }
}
impl From<i64> for GenomeId {
    fn from(value: i64) -> Self {
        Self {
            uuid: value.to_be_bytes(),
        }
    }
}

impl ToVariant for GenomeId {
    fn to_variant(&self) -> Variant {
        i64::from(*self).to_variant()
    }
}

impl FromVariant for GenomeId {
    fn try_from_variant(variant: &Variant) -> Result<Self, VariantConversionError> {
        Ok(i64::from_variant(variant).into())
    }
}
