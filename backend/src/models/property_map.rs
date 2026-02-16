use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct InsertEntry<'a> {
    map: &'a mut PropertyMap,
    type_id: TypeId,
    index: usize,
}

impl<'a> InsertEntry<'a> {
    pub fn alias(self, alias: impl Into<String>) -> &'a mut PropertyMap {
        self.map
            .aliases
            .insert(alias.into(), (self.type_id, self.index));
        self.map
    }
}

#[derive(Debug)]
pub struct PropertyMap {
    properties: HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>,
    aliases: HashMap<String, (TypeId, usize)>,
}

impl PropertyMap {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) -> InsertEntry<'_> {
        let type_id = TypeId::of::<T>();
        let values = self.properties.entry(type_id).or_default();
        values.push(Box::new(value));
        let index = values.len() - 1;

        InsertEntry {
            map: self,
            type_id,
            index,
        }
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.properties
            .get(&TypeId::of::<T>())
            .and_then(|values| values.last())
            .and_then(|v| v.downcast_ref::<T>())
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.properties
            .get_mut(&TypeId::of::<T>())
            .and_then(|values| values.last_mut())
            .and_then(|v| v.downcast_mut::<T>())
    }

    pub fn get_by_alias<T: Any + Send + Sync>(&self, alias: &str) -> Option<&T> {
        let (type_id, index) = self.aliases.get(alias)?;
        self.properties
            .get(type_id)
            .and_then(|values| values.get(*index))
            .and_then(|v| v.downcast_ref::<T>())
    }
}
