#[macro_use]
extern crate serde_derive;

use std::iter;

type Places = Vec<usize>;

#[derive(Debug, PartialEq, PartialOrd)]
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
            .chain(iter::repeat(false).take(p - 1))
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
                let pp = place - 1;
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
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
struct Petrinet(Vec<Transition>);
impl Petrinet {
    fn new(x: Vec<Transition>) -> Self {
        Petrinet(x)
    }
    fn place_count(&self) -> usize {
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

fn max(xs: &Places) -> &usize {
    xs.iter().fold(&0, |acc, x| acc.max(x))
}

fn read_deserialize_file(filepath: &str) -> Result<Nbpt, std::io::Error> {
    // TODO: fix error types, not sure how it compiles as there seem to be 2 distinct errrors
    let f = std::fs::File::open(filepath)?;
    let deserialized: Nbpt = serde_json::from_reader(f)?;
    Ok(deserialized)
}
#[derive(Serialize, Deserialize, Debug)]
struct Nbpt {
    name: String,
    names: Vec<String>, // name of transitions
    partition: Partition,
}

// partitions are separated by zeros and alternate between consume and produce
// respectively. each produce-consume pair constitutes one transition
#[derive(Serialize, Deserialize, Debug)]
struct Partition(Vec<usize>);

impl From<Partition> for Petrinet {
    fn from(partition: Partition) -> Self {
        let mut consume = true;
        let mut c: Places = vec![];
        let mut p: Places = vec![];
        let mut tr: Vec<Transition> = vec![];
        for i in partition.0.into_iter() {
            match (i, consume) {
                (0, true) => {
                    consume = !consume;
                }
                (0, false) => {
                    tr.push(Transition(c, p));
                    c = vec![];
                    p = vec![];
                    consume = !consume;
                }
                (_, true) => c.push(i),
                (_, false) => p.push(i),
            }
        }
        Petrinet(tr)
    }
}

fn main() {
    // let nbpt = read_deserialize_file("./swap-protocol-both.nbpt.json").unwrap();
    // // println!("nbpt: {:?}", nbpt);
    // let petrinet = Petrinet::from(nbpt.partition);
    // println!("{:?}", petrinet);
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
    fn simple_net() {
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

    #[test]
    fn deserialize_convert() {
        let nbpt = read_deserialize_file("./swap-protocol-both.nbpt.json")
            .expect("File should be converted manually to nbpt.json format using stbx");
        let petrinet = Petrinet::from(nbpt.partition);
        let expected = Petrinet(vec![
            Transition(vec![2, 15], vec![3, 16]),
            Transition(vec![1, 6], vec![2, 11]),
            Transition(vec![2, 4], vec![19]),
            Transition(vec![7], vec![4, 5]),
            Transition(vec![5], vec![6]),
            Transition(vec![8, 9], vec![7]),
            Transition(vec![10], vec![8, 9]),
            Transition(vec![19], vec![3]),
            Transition(vec![19], vec![1, 18]),
            Transition(vec![13, 11], vec![12, 14]),
            Transition(vec![14], vec![15]),
            Transition(vec![12, 16], vec![17]),
            Transition(vec![12, 18], vec![13]),
        ]);
        assert_eq!(petrinet, expected, "the two petrinets should be identical")
    }
}
