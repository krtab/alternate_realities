use alternate_realities::{AlternateRealities, ExplorationStrategy, ExtremumFirstThenRandom, Reality, Sequence};

fn main() {
    let mut ar = AlternateRealities::new();
    while let Some(mut real) = ar.get_next() {
       process(&mut real);
    }
}

fn process(real: &mut Reality) -> Option<()> {
    let b = real.get(Sequence::new([(1,true),(0,false)]))?;
    let i = real.get(Sequence::new([(1, "ahaha"),(0,"ohoho"),(-1,"gzgzgz")]))?;
    let j = real.get(ExtremumFirstThenRandom.limit(10))?;
    dbg!(b,i,j);
    Some(())
}