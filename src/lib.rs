use std::{
    any::{Any, TypeId},
    collections::BTreeMap,
    ops::{Range, RangeBounds, RangeInclusive},
    vec,
};

pub trait ValueGenerator {
    fn nth(&self, i: usize) -> Option<u64>;
    fn max(&self) -> usize;
}

pub trait Encodable {
    fn encode(self) -> u64;
    fn decode(v: u64) -> Self;
}

trait Rangeable: Copy + Ord + Encodable {
    fn add_n(self, n: usize) -> Option<Self>;
}

struct RangeGenerator<T> {
    first: T,
    total_len: usize,
}

impl<T> ValueGenerator for RangeGenerator<T>
where
    T: Rangeable,
{
    fn nth(&self, i: usize) -> Option<u64> {
        self.first.add_n(i).map(Encodable::encode)
    }

    fn max(&self) -> usize {
        self.total_len
    }
}

struct ValueGeneratorVTable {
    nth: unsafe fn(*const (), i: usize) -> Option<u64>,
    max: unsafe fn(*const ()) -> usize,
    drop: unsafe fn(*mut ()),
}

struct DynValueGenerator {
    generator: *mut (),
    vtable: ValueGeneratorVTable,
}

impl Drop for DynValueGenerator {
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.generator) }
    }
}

impl DynValueGenerator {
    fn new<V: ValueGenerator>(v: V) -> Self {
        let generator = Box::new(v);
        let generator_ptr = Box::into_raw(generator);
        unsafe fn nth<V: ValueGenerator>(generator: *const (), i: usize) -> Option<u64> {
            V::nth(unsafe { &*generator.cast() }, i)
        }
        unsafe fn max<V: ValueGenerator>(generator: *const ()) -> usize {
            V::max(unsafe { &*generator.cast() })
        }
        unsafe fn drop<V: ValueGenerator>(generator: *mut ()) {
            unsafe { Box::from_raw(generator.cast::<V>()) };
        }
        Self {
            generator: generator_ptr.cast(),
            vtable: ValueGeneratorVTable {
                nth: nth::<V>,
                max: max::<V>,
                drop: drop::<V>,
            },
        }
    }
}

impl ValueGenerator for DynValueGenerator {
    fn nth(&self, i: usize) -> Option<u64> {
        unsafe { (self.vtable.nth)(self.generator, i) }
    }

    fn max(&self) -> usize {
        unsafe { (self.vtable.max)(self.generator) }
    }
}

struct Node {
    generator: DynValueGenerator,
    type_id: TypeId,
    children: Vec<Option<Node>>,
}

pub struct AlternateRealities {
    choice_tree: Option<Node>,
    try_next: BTreeMap<i64, Vec<Vec<usize>>>,
}

impl AlternateRealities {
    pub fn new() -> Self {
        Self { choice_tree: None, try_next: BTreeMap::new() }
    }
}

impl AlternateRealities {
    pub fn get_next(&mut self) -> Option<Reality<'_>> {
        loop {
            let mut fst = self.try_next.first_entry()?;
            match fst.get_mut().pop() {
                None => {
                    fst.remove();
                }
                Some(v) => {
                    return Some(Reality {
                        current_node: Some(&mut self.choice_tree),
                        try_next: &mut self.try_next,
                        left_already_decided: v.into_iter(),
                        path_acc: Vec::new(),
                    });
                }
            }
        }
    }
}

pub struct Reality<'a> {
    // This option is only transient
    current_node: Option<&'a mut Option<Node>>,
    try_next: &'a mut BTreeMap<i64, Vec<Vec<usize>>>,
    left_already_decided: vec::IntoIter<usize>,
    path_acc: Vec<usize>,
}

impl<'a> Reality<'a> {
    pub fn get<T: Encodable + Any, V: ValueGenerator>(
        &mut self,
        new_gen: impl FnOnce() -> V,
        prio: i64,
    ) -> Option<T> {
        let raw = self.get_raw(
            || Node {
                generator: DynValueGenerator::new(new_gen()),
                type_id: TypeId::of::<T>(),
                children: vec![],
            },
            prio,
        )?;
        Some(T::decode(raw))
    }

    fn get_raw(&mut self, new_gen_node: impl FnOnce() -> Node, prio: i64) -> Option<u64> {
        let current_node = self.current_node.take().unwrap();
        let current_node = current_node.get_or_insert_with(new_gen_node);
        match self.left_already_decided.next() {
            Some(i) => {
                let v = current_node.generator.nth(i).unwrap();
                self.current_node = Some(current_node.children.get_mut(i).unwrap());
                self.path_acc.push(i);
                Some(v)
            }
            None => {
                let max = current_node.generator.max();
                let n = current_node.children.len();
                if !(n < max) {
                    return None;
                }
                let n = n + 1;
                let v = current_node.generator.nth(n)?;
                self.path_acc.push(n);
                self.try_next
                    .entry(prio)
                    .or_default()
                    .push(self.path_acc.clone());
                current_node.children.push(None);
                self.current_node = Some(current_node.children.last_mut().unwrap());
                Some(v)
            }
        }
    }
}
