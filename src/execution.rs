use crate::petrinet::Petrinet;
use std::iter;
impl<'a> From<&'a Petrinet> for Execution<'a> {
    fn from(net: &'a Petrinet) -> Self {
        let p = net.place_count();
        let marking: Vec<bool> = Some(true)
            .into_iter()
            .chain(iter::repeat(false).take(p - 1))
            .collect();
        Execution(net, marking)
    }
}

#[derive(Debug)]
pub struct Execution<'a>(&'a Petrinet, Vec<bool>);

impl<'a> Execution<'a> {
    pub fn enabled(&self, t: usize) -> bool {
        let Execution(net, marking) = self;
        match net.transition(t) {
            None => false,
            Some(tr) => tr.consume().iter().all(|&place| {
                let pp = place - 1;
                match marking.get(pp) {
                    None => false,
                    Some(&marked) => marked,
                }
            }),
        }
    }
    pub fn fire(self, t: usize) -> Self {
        if !self.enabled(t) {return self;}
        let Execution(net, marking) = self;
        let tr = net.transition(t).expect("already know transition is enabled and therefore exists");
        let cs = tr.consume();
        let ps = tr.produce();

        let new_marking = marking
            .iter()
            .enumerate()
            .map(|(pos, &has_token)| {
                let place_ix = &(pos + 1);
                let is_consumed = cs.contains(place_ix);
                let is_produced = ps.contains(place_ix);
                let yes_token = true; // net is 1-safe, so place either has or doesn't have a token
                let no_token = false;
                match (is_consumed, is_produced) {
                    (true, true) => yes_token, // both produce and consumed, so there is a token afterwards
                    (true, false) => no_token, // just consumed, so there is _no_ token afterwards
                    (false, true) => yes_token, // only produced, so there is a token afterwards
                    (false, false) => has_token, // not modified by this transition
                }
            })
            .collect();

        // println!("execution: {:?}", new_marking);
        Execution(net, new_marking)
    }
}
