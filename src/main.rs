use std::iter;

#[derive(Clone, Default)]
struct Transition(Vec<u32>, Vec<u32>);

impl Transition {
    fn consume(&self) -> &Vec<u32> {
        &self.0
    }
    fn produce(&self) -> &Vec<u32> {
        &self.1
    }
}

impl<'a> From<&'a Petrinet> for Execution<'a> {
    fn from(net: &'a Petrinet) -> Self {
        let p = net.place_count();
        let marking: Vec<bool> = [true]
            .iter()
            .cloned()
            .chain(iter::repeat(false).take((p - 1) as usize))
            .collect();
        Execution(net, marking)
    }
}

#[derive(Clone, Debug)]
struct Execution<'a>(&'a Petrinet, Vec<bool>);

impl<'a> Execution<'a> {
    fn enabled(&self, t: usize) -> bool {
        let Execution(net, marking) = self;
        match net.transition(t) {
            None => false,
            Some(tr) => tr.consume().iter().all(|&place| {
                let pp: usize = (place as usize) - 1;
                match marking.get(pp).cloned() {
                    None => false,
                    Some(marked) => marked,
                }
            }),
        }
    }
    fn fire(self, t: usize) -> Self {
        let Execution(net, marking) = self;
        match net.transition(t) {
            None => Execution(net, marking),
            Some(tr) => {
                let cs = tr.consume();
                let ps = tr.produce();

                let new_marking = marking
                    .iter()
                    .enumerate()
                    .map(|(pos, &has_token)| {
                        let place_ix = &(pos as u32 + 1);
                        let is_consumed = cs.contains(place_ix);
                        let is_produced = ps.contains(place_ix);
                        let yes_token = true; // net is 1-safe, so place either has or doesn't have a token
                        let no_token = false;
                        match (is_consumed, is_produced) {
                            (true, true) => yes_token,        // both produce and consumed, so there is a token afterwards
                            (true, false) => no_token,      // just consumed, so there is _no_ token afterwards
                            (false, true) => yes_token,       // only produced, so there is a token afterwards
                            (false, false) => has_token, // not modified by this transition
                        }
                    })
                    .collect();

                Execution(net, new_marking)
            }
        }
    }
}

#[derive(Clone, Default)]
struct Petrinet(Vec<Transition>);
impl Petrinet {
    fn new(x: Vec<Transition>) -> Self {
        Petrinet(x)
    }
    fn place_count(&self) -> u32 {
        self.0
            .iter()
            .cloned()
            .fold(0, |acc, Transition(consumed, produced)| {
                let pmax = max(&produced);
                let cmax = max(&consumed);
                let m = pmax.max(cmax);
                acc.max(m.clone()).max(1) // require at least one place (which will hold the initial token)
            })
    }
    fn transitions(&self) -> &Vec<Transition> {
        &self.0
    }
    fn transition(&self, t: usize) -> Option<&Transition> {
        self.transitions().get(t)
    }

    /// nr of transitions
    fn transition_count(&self) -> usize {
        self.0.len()
    }
}

fn max(xs: &Vec<u32>) -> &u32 {
    xs.iter().fold(&0, |acc, x| acc.max(x))
}

fn main() {
    // let net = Petrinet::new(
    //     [
    //         Transition(vec![1], vec![2, 3]),
    //         Transition(vec![2], vec![4]),
    //         Transition(vec![3], vec![5]),
    //         Transition(vec![4, 5], vec![6]),
    //     ]
    //     .to_vec(),
    // );
    // let e = Execution::from(net);
    // // println!("transition 0 is enabled = {}", e.enabled(0));
    // // println!("transition 1 is enabled = {}", e.enabled(1));
    // // println!("transition 1 is enabled = {}", e.enabled(2));
    // // println!("transition 1 is enabled = {}", e.enabled(3));

    // let e1 = e.fire(0);
    // // println!("transition 0 is enabled = {}", e1.enabled(0));
    // // println!("transition 1 is enabled = {}", e1.enabled(1));
    // // println!("transition 1 is enabled = {}", e1.enabled(2));
    // // println!("transition 1 is enabled = {}", e1.enabled(3));
    // // engine(net);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn example_usage() {
        let net = Petrinet::new(
            [
                Transition(vec![1], vec![2, 3]),
                Transition(vec![2], vec![4]),
                Transition(vec![3], vec![5]),
                Transition(vec![4, 5], vec![6]),
            ]
            .to_vec(),
        );
        let e = Execution::from(&net);
        assert_eq!(e.enabled(0), true, "transition 0 must be enabled");
        assert_eq!(e.enabled(1), false, "transition 1 must be disabled");
        assert_eq!(e.enabled(2), false, "transition 2 must be disabled");
        assert_eq!(e.enabled(3), false, "transition 3 must be disabled");
        assert_eq!(e.enabled(4), false, "transition 4 must be disabled");

        let e = e.fire(0);
        assert_eq!(e.enabled(0), false, "transition 0 must be disabled");
        assert_eq!(e.enabled(1), true, "transition 1 must be enabled");
        assert_eq!(e.enabled(2), true, "transition 2 must be enabled");
        assert_eq!(e.enabled(3), false, "transition 3 must be disabled");
        assert_eq!(e.enabled(4), false, "transition 4 must be disabled");


        let e = e.fire(2);
        assert_eq!(e.enabled(0), false, "transition 0 must be disabled");
        assert_eq!(e.enabled(1), true, "transition 1 must be enabled");
        assert_eq!(e.enabled(2), false, "transition 2 must be enabled");
        assert_eq!(e.enabled(3), false, "transition 3 must be disabled");
        assert_eq!(e.enabled(4), false, "transition 4 must be disabled");


        let e = e.fire(1);
        assert_eq!(e.enabled(0), false, "transition 0 must be disabled");
        assert_eq!(e.enabled(1), false, "transition 1 must be disabled");
        assert_eq!(e.enabled(2), false, "transition 2 must be disabled");
        assert_eq!(e.enabled(3), true, "transition 3 must be enabled");
        assert_eq!(e.enabled(4), false, "transition 4 must be disabled");


        let e = e.fire(3);
        assert_eq!(e.enabled(0), false, "transition 0 must be disabled");
        assert_eq!(e.enabled(1), false, "transition 1 must be disabled");
        assert_eq!(e.enabled(2), false, "transition 2 must be disabled");
        assert_eq!(e.enabled(3), false, "transition 3 must be disabled");
        assert_eq!(e.enabled(4), false, "transition 4 must be disabled");
    }
}
