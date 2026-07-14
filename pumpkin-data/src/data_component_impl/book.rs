use crate::data_component_impl::{DataComponentImpl, default_impl};
use pumpkin_nbt::tag::NbtTag;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WritableBookContentImpl {
    pub pages: Vec<String>,
}
impl WritableBookContentImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let compound = tag.extract_compound()?;
        let mut pages = Vec::new();
        if let Some(NbtTag::List(l)) = compound.get("pages") {
            for _ in l {
                pages.push(String::new());
            }
        }
        Some(Self { pages })
    }
}
impl DataComponentImpl for WritableBookContentImpl {
    default_impl!(WritableBookContent);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WrittenBookContentImpl {
    pub pages: Vec<String>,
    pub title: String,
    pub author: String,
    pub generation: i32,
    pub resolved: bool,
}
impl WrittenBookContentImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let compound = tag.extract_compound()?;
        let mut pages = Vec::new();
        if let Some(NbtTag::List(l)) = compound.get("pages") {
            for _ in l {
                pages.push(String::new());
            }
        }
        let title = compound.get_string("title")?;
        let author = compound.get_string("author")?;
        let generation = compound.get_int("generation")?;
        let resolved = compound.get_bool("resolved")?;
        Some(Self {
            pages,
            title: title.to_owned(),
            author: author.to_owned(),
            generation,
            resolved,
        })
    }
}
impl DataComponentImpl for WrittenBookContentImpl {
    default_impl!(WrittenBookContent);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DebugStickStateImpl;
impl DataComponentImpl for DebugStickStateImpl {
    default_impl!(DebugStickState);
}
