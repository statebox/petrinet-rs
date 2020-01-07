use std::iter;

type Places = Vec<u32>;

#[derive(Debug)]
struct Transition(Places, Places);

impl Transition {
    fn consume(&self) -> &Places {
        &self.0
    }
    fn produce(&self) -> &Places {
        &self.1
    }
}

impl<'a> From<&'a Petrinet> for Execution<'a> {
    fn from(net: &'a Petrinet) -> Self {
        let p = net.place_count();
        let marking: Vec<bool> = Some(true)
            .into_iter()
            .chain(iter::repeat(false).take(p as usize - 1))
            .collect();
        Execution(net, marking)
    }
}

#[derive(Debug)]
struct Execution<'a>(&'a Petrinet, Vec<bool>);

impl<'a> Execution<'a> {
    fn enabled(&self, t: usize) -> bool {
        let Execution(net, marking) = self;
        match net.transition(t) {
            None => false,
            Some(tr) => tr.consume().iter().all(|&place| {
                let pp = place as usize - 1;
                match marking.get(pp) {
                    None => false,
                    Some(&marked) => marked,
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
    }
}

#[derive(Debug)]
struct Petrinet(Vec<Transition>);
impl Petrinet {
    fn new(x: Vec<Transition>) -> Self {
        Petrinet(x)
    }
    fn place_count(&self) -> u32 {
        self.transitions()
            .iter()
            .fold(0, |acc, Transition(consumed, produced)| {
                let pmax = max(&produced);
                let cmax = max(&consumed);
                let m = pmax.max(cmax);
                acc.max(*m).max(1) // require at least one place (which will hold the initial token)
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
        self.transitions().len()
    }
}

fn max(xs: &Places) -> &u32 {
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
        // transition i represented as (i)
        // place j represented as j
        //
        //                2 - 4
        //              /  (1)  \
        //  start -> 1 -(0)   (3)- 6
        //              \  (2)  /
        //                3 - 5
        let net = Petrinet::new(vec![
            Transition(vec![1], vec![2, 3]),
            Transition(vec![2], vec![4]),
            Transition(vec![3], vec![5]),
            Transition(vec![4, 5], vec![6]),
        ]);
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
