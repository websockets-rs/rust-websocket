#[derive(Clone)]
pub(crate) struct VecMap<K, V> {
    vec: Vec<(K, V)>,
}

impl<K: PartialEq, V> VecMap<K, V> {
    pub(crate) fn new() -> VecMap<K, V> {
        VecMap {
            vec: Vec::new()
        }
    }

    pub(crate) fn insert(&mut self, key: K, value: V) {
        match self.find(&key) {
            Some(pos) => self.vec[pos] = (key, value),
            None => self.vec.push((key, value))
        }
    }

    pub(crate) fn entry(&mut self, key: K) -> Entry<K, V> {
        match self.find(&key) {
            Some(pos) => Entry::Occupied(OccupiedEntry {
                vec: self,
                pos: pos,
            }),
            None => Entry::Vacant(VacantEntry {
                vec: self,
                key: key,
            })
        }
    }

    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        self.find(key).map(move |pos| &self.vec[pos].1)
    }

    pub(crate) fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.find(key).map(move |pos| &mut self.vec[pos].1)
    }

    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.find(key).is_some()
    }

    pub(crate) fn len(&self) -> usize { self.vec.len() }
    pub(crate) fn iter(&self) -> ::std::slice::Iter<(K, V)> {
        self.vec.iter()
    }
    pub(crate) fn remove(&mut self, key: &K) -> Option<V> {
        self.find(key).map(|pos| self.vec.remove(pos)).map(|(_, v)| v)
    }
    pub(crate) fn clear(&mut self) {
        self.vec.clear();
    }

    fn find(&self, key: &K) -> Option<usize> {
        self.vec.iter().position(|entry| key == &entry.0)
    }
}

pub(crate) enum Entry<'a, K: 'a, V: 'a> {
    Vacant(VacantEntry<'a, K, V>),
    Occupied(OccupiedEntry<'a, K, V>)
}

pub(crate) struct VacantEntry<'a, K: 'a, V: 'a> {
    vec: &'a mut VecMap<K, V>,
    key: K,
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    pub(crate) fn insert(self, val: V) -> &'a mut V {
        let vec = self.vec;
        vec.vec.push((self.key, val));
        let pos = vec.vec.len() - 1;
        &mut vec.vec[pos].1
    }
}

pub(crate) struct OccupiedEntry<'a, K: 'a, V: 'a> {
    vec: &'a mut VecMap<K, V>,
    pos: usize,
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    pub(crate) fn into_mut(self) -> &'a mut V {
        &mut self.vec.vec[self.pos].1
    }
}
