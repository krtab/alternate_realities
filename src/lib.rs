use std::{collections::BTreeMap, vec};

struct TimeLine {
    past: Vec<u64>,
    next: u64,
}

pub struct AlternateRealities {
    try_next: BTreeMap<i64, Vec<TimeLine>>,
}

impl AlternateRealities {
    pub fn new() -> Self {
        Self {
            try_next: BTreeMap::from([(
                0,
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
                        current_step: 0,
                        base: self,
                    });
                }
            }
        }
    }

    fn add_timeline(&mut self, prio: i64, timeline: TimeLine) {
        self.try_next.entry(prio).or_default().push(timeline);
    }
}

pub struct Reality<'a> {
    // None to indicate the timelien should not be used anymore
    timeline: Option<TimeLine>,
    current_step: usize,
    base: &'a mut AlternateRealities,
}

pub trait ExplorationStrategy<Value> {
    fn prio(&self, raw: u64) -> (Option<i64>, Option<i64>);
    fn decode(&self, raw: u64) -> Value;
}

impl<V, S: ExplorationStrategy<V>> ExplorationStrategy<V> for &S {
    fn prio(&self, raw: u64) -> (Option<i64>, Option<i64>) {
        (*self).prio(raw)
    }

    fn decode(&self, raw: u64) -> V {
        (*self).decode(raw)
    }
}

struct ExtremumFirstThenRandom;

impl ExplorationStrategy<i64> for ExtremumFirstThenRandom {
    fn prio(&self, raw: u64) -> (Option<i64>, Option<i64>) {
        match raw {
            0 => (Some())
        }
    }

    fn decode(&self, raw: u64) -> i64 {
        todo!()
    }
}

impl<'a> Reality<'a> {
    pub fn get<Value, Strat: ExplorationStrategy<Value>>(&mut self, strat: Strat) -> Option<Value> {
        let raw = self.get_raw(|raw| strat.prio(raw))?;
        Some(strat.decode(raw))
    }

    pub fn get_bool(&mut self, prio_true: i64, prio_false: i64) -> Option<bool> {
        let &TimeLine { ref past, next } = self.timeline.as_ref()?;
        match past.get(self.current_step) {
            Some(0) => Some(false),
            Some(1) => Some(true),
            Some(_) => panic!("Wrong boolean encoding"),
            None => {
                debug_assert_eq!(next, 0);
                let past_true = {
                    let mut v = Vec::with_capacity(past.len() + 1);
                    v.extend_from_slice(past);
                    v.push(1);
                    v
                };
                let past_false = {
                    let mut v = self.timeline.take().unwrap().past;
                    v.push(0);
                    v
                };
                self.base.add_timeline(
                    prio_true,
                    TimeLine {
                        past: past_true,
                        next: 0,
                    },
                );
                self.base.add_timeline(
                    prio_false,
                    TimeLine {
                        past: past_false,
                        next: 0,
                    },
                );
                None
            }
        }
    }

    fn get_raw(&mut self, prio: impl FnOnce(u64) -> (Option<i64>, Option<i64>)) -> Option<u64> {
        let &TimeLine { ref past, next } = self.timeline.as_ref()?;
        match past.get(self.current_step) {
            Some(&v) => Some(v),
            None => {
                let mut already_last = false;

                let mut get_past = |last_call| {
                    if last_call {
                        debug_assert!(!already_last);
                        already_last = true;
                        self.timeline.take().unwrap().past
                    } else {
                        self.timeline.as_ref().unwrap().past.clone()
                    }
                };

                let (prio_this, prio_other) = prio(next);

                let next_next = next.checked_add(1);

                if let Some(prio_this) = prio_this {
                    let mut past = get_past(prio_other.is_none() || next_next.is_none());
                    past.push(next);
                    let timeline = TimeLine { past, next: 0 };
                    self.base.add_timeline(prio_this, timeline);
                }
                if let Some(prio_other) = prio_other {
                    if let Some(next) = next_next {
                        let past = get_past(true);
                        let timeline = TimeLine { past, next };
                        self.base.add_timeline(prio_other, timeline);
                    }
                }
                None
            }
        }
    }
}
