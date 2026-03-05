use std::{cmp::Reverse, collections::BTreeMap, i64, vec};

#[derive(Debug)]
struct TimeLine {
    past: Vec<u64>,
    next: u64,
}

#[derive(Debug)]
pub struct AlternateRealities {
    try_next: BTreeMap<Reverse<i64>, Vec<TimeLine>>,
}

impl AlternateRealities {
    pub fn new() -> Self {
        Self {
            try_next: BTreeMap::from([(
                Reverse(0),
                vec![TimeLine {
                    past: vec![],
                    next: 0,
                }],
            )]),
        }
    }

    pub fn get_next(&mut self) -> Option<Reality<'_>> {
        loop {
            let mut fst = self.try_next.first_entry()?;
            match fst.get_mut().pop() {
                None => {
                    fst.remove();
                }
                Some(timeline) => {
                    return Some(Reality {
                        timeline: Some(timeline),
                        this_reality_priority: fst.key().0,
                        current_step: 0,
                        base: self,
                    });
                }
            }
        }
    }

    fn add_timeline(&mut self, prio: i64, timeline: TimeLine) {
        // dbg!(prio, &timeline);
        self.try_next.entry(Reverse(prio)).or_default().push(timeline);
    }
}

#[derive(Debug)]
pub struct Reality<'a> {
    // None to indicate the timelien should not be used anymore
    timeline: Option<TimeLine>,
    current_step: usize,
    this_reality_priority: i64,
    base: &'a mut AlternateRealities,
}

pub trait ExplorationStrategy<Value> : Sized {
    fn step(&mut self, raw: u64) -> (Option<Value>, Option<(i64, u64)>);

    fn limit(self, limit: u64) -> Limit<Self> {
        Limit { limit, inner: self }
    }
}

impl<V, S: ExplorationStrategy<V>> ExplorationStrategy<V> for &mut S {
    fn step(&mut self, raw: u64) -> (Option<V>, Option<(i64, u64)>) {
        (*self).step(raw)
    }
}

pub struct ExtremumFirstThenRandom;

impl ExplorationStrategy<i64> for ExtremumFirstThenRandom {
    fn step(&mut self, raw: u64) -> (Option<i64>, Option<(i64, u64)>) {
        match raw {
            0 => (Some(0), Some((0, 1))),
            1 => (Some(i64::MAX), Some((0, 2))),
            2 => (Some(i64::MIN), Some((-1, 3))),
            n => {
                //todo better random
                (Some((n - 2) as i64), Some((-1, n + 1)))
            }
        }
    }
}



pub struct Sequence<const N: usize, T> {
    sorted_by_prio: [(i64, T); N],
}

impl<const N: usize, T> Sequence<N, T> {
    pub fn new( sorted_by_prio: [(i64, T); N]) -> Self {
        assert!(sorted_by_prio.is_sorted_by_key(|(prio,_)| Reverse(*prio)));
        Self { sorted_by_prio }
    }
}

impl<const N: usize, T: Copy> ExplorationStrategy<T> for Sequence<N, T> {
    fn step(&mut self, raw: u64) -> (Option<T>, Option<(i64, u64)>) {
        let idx = raw as usize;
        let this = self.sorted_by_prio.get(idx).map(|(_prio, v)| *v);
        let next = self
            .sorted_by_prio
            .get(idx + 1)
            .map(|(prio, _v)| (*prio, raw + 1));
        (this, next)
    }
}

pub struct Limit<S> {
    limit: u64,
    inner: S,
}

impl<V, S: ExplorationStrategy<V>> ExplorationStrategy<V> for Limit<S> {
    fn step(&mut self, raw: u64) -> (Option<V>, Option<(i64, u64)>) {
        if self.limit == raw {
            (None, None)
        } else {
            self.inner.step(raw)
        }
    }
}

impl<'a> Reality<'a> {
    pub fn get<Value, Strat: ExplorationStrategy<Value>>(&mut self, mut strat: Strat) -> Option<Value> {
        self.get_raw(|raw| strat.step(raw))
    }

    fn get_raw<T>(&mut self, f: impl FnOnce(u64) -> (Option<T>, Option<(i64, u64)>)) -> Option<T> {
        let &TimeLine { ref past, next } = self.timeline.as_ref()?;
        match past.get(self.current_step) {
            Some(&v) => {
                self.current_step += 1;
                f(v).0
            },
            None => {
                let mut get_past = |consuming| {
                    if consuming {
                        self.timeline.take().unwrap().past
                    } else {
                        self.timeline.as_ref().unwrap().past.clone()
                    }
                };

                let (continue_here, other) = f(next);

                if let Some((prio, next_next)) = other {
                    let past = get_past(continue_here.is_none());
                    let timeline = TimeLine {
                        past,
                        next: next_next,
                    };
                    self.base.add_timeline(self.this_reality_priority + prio, timeline);
                }
                if let Some(continue_here) = continue_here {
                    let mut past = get_past(true);
                    past.push(next);
                    let new_timeline = TimeLine { past, next: 0 };
                    self.timeline = Some(new_timeline);
                    self.current_step += 1;
                    Some(continue_here)
                } else {
                    None
                }
            }
        }
    }
}
