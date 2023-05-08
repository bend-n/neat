use core::marker::PhantomData;
use godot::obj::dom::UserDomain;
use godot::prelude::*;

pub struct Map<K, V>(Dictionary, PhantomData<(K, V)>);

impl<K, V> Map<K, V>
where
    K: ToVariant + Clone + FromVariant,
    V: GodotClass<Declarer = UserDomain> + Clone,
{
    #[inline]
    pub fn get(&self, k: K) -> Option<Gd<V>> {
        Some(Gd::<V>::from_variant(&self.0.get(k)?))
    }
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (K, Gd<V>)> + '_ {
        self.0
            .iter_shared()
            .map(|(k, v)| (K::from_variant(&k), Gd::<V>::from_variant(&v)))
    }
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = Gd<V>> + '_ {
        self.0.iter_shared().map(|(_, v)| Gd::<V>::from_variant(&v))
    }
    #[inline]
    pub fn new() -> Self {
        Self(Dictionary::new(), PhantomData)
    }
    #[inline]
    pub fn set(&mut self, k: K, v: Gd<V>) {
        self.0.set(k, v)
    }
    #[inline]
    pub fn erase(&mut self, k: K) {
        self.0.remove(k);
    }
    #[inline]
    pub fn remove(&mut self, k: K) -> Option<Gd<V>> {
        Some(self.0.remove(k)?.to())
    }
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = K> + '_ {
        self.0.keys_shared().map(|k| K::from_variant(&k))
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    #[inline]
    pub fn from_vec(vec: Vec<(K, Gd<V>)>) -> Self {
        let mut d = Self::new();
        for (k, v) in vec {
            d.set(k, v);
        }
        d
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    #[inline]
    pub fn d(&self) -> Dictionary {
        self.0.share()
    }
}

impl<K, V> std::fmt::Debug for Map<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self(Dictionary::new(), PhantomData)
    }
}
impl<K, V> Clone for Map<K, V>
where
    K: ToVariant + Clone + FromVariant,
    V: GodotClass<Declarer = UserDomain> + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let mut new = Map::new();
        for (k, v) in self.iter() {
            new.set(k.clone(), Gd::new(v.bind().clone())); // i dont trust Variant::Clone
        }
        new
    }
}
impl<K, V> From<Dictionary> for Map<K, V> {
    fn from(value: Dictionary) -> Self {
        Self(value, PhantomData)
    }
}
impl<K, V> Share for Map<K, V> {
    fn share(&self) -> Self {
        Self(self.0.share(), PhantomData)
    }
}
