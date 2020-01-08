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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Nbpt {
    name: String,
    names: Vec<String>, // name of transitions
    partition: Partition,
}

impl Nbpt {
    fn from_file(filepath: &str) -> Result<Nbpt, std::io::Error> {
        // TODO: fix error types, not sure how it compiles as there seem to be 2 distinct errrors
        let f = std::fs::File::open(filepath)?;
        let deserialized: Nbpt = serde_json::from_reader(f)?;
        Ok(deserialized)
    }
    fn partition(self) -> Partition {
        self.partition
    }
}

fn write_trait(nbpt: Nbpt, trait_file: &str) -> Result<(), std::io::Error> {
    let valid_partition = ValidPartition::new(nbpt.clone().partition()).unwrap();
    let net = Petrinet::from(valid_partition);
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(trait_file)
        .unwrap();
    {
        writeln!(f, "// Auto-generated file using petrinet-rs").unwrap();
        let Partition(mut xs) = nbpt.partition;
        xs.sort_unstable();
        xs.dedup();
        let type_params = xs.into_iter().enumerate().fold("".to_owned(), |mut acc, (i, x)| {
            if i > 0 { acc.push_str(","); }
            acc.push_str("T");
            acc.push_str(&(x.to_string()));
            acc
        });
        let mut name = nbpt.name;
        name.retain(|c| !c.is_whitespace());
        writeln!(f, "trait {}<{}> {{ ", name, type_params).unwrap();
    }
    {
        for (name, trs) in nbpt.names.iter().zip(net.transitions().iter()) {
            let Transition(consume, produce) = trs;
            let c = consume.iter().enumerate().fold("".to_owned(), |mut acc, (i, x)| {
                if i > 0 { acc.push_str(", "); }
                acc.push_str("p");
                acc.push_str(&(i.to_string()));
                acc.push_str(": T");
                acc.push_str(&(x.to_string()));
                acc
            });

            let t = produce.iter().enumerate().fold("".to_owned(), |mut acc, (i, x)| {
                if i > 0 { acc.push_str(", "); }
                acc.push_str("T");
                acc.push_str(&(x.to_string()));
                acc
            });
            writeln!(f, "  fn {}({}) -> ({})", name, c, t).unwrap();
        }
    }
    writeln!(f, "}}", ).unwrap();
    Ok(())
}
// Partitions are separated by zeros and alternate between consume and produce
// respectively. Each produce-consume pair constitutes one transition
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Partition(Vec<usize>);
impl Partition {
    // Partition values should start at 1 and increament in +1 steps; zeros are
    // the separators
    fn is_valid(partition: Partition) -> bool {
        let Partition(mut xs) = partition;
        xs.sort_unstable();
        xs.dedup();
        let xs: Vec<_> = xs.into_iter().skip_while(|&y| y == 0).collect();
        for (ix, x) in xs.iter().enumerate() {
            if ix < 1 {
                // must start at 1
                if *x != 1 as usize {
                    return false;
                }
            } else {
                let diff = x - xs[ix - 1];
                // difference between sorted subsequent items must be 1
                // println!("{:?}, {:?}", x, xs[ix - 1]);
                if diff > 1 {
                    return false;
                }
            }
        }
        true
    }
}
struct ValidPartition(Vec<usize>);
impl ValidPartition {
    fn new(partition: Partition) -> Result<Self, &'static str> {
        // TODO: create Error type
        match Partition::is_valid(partition.clone()) {
            true => Ok(ValidPartition(partition.0)),
            false => Err("Partition values should start at 1 and increament in +1 steps; zeros are the separators")
        }
    }
}

impl From<ValidPartition> for Petrinet {
    fn from(partition: ValidPartition) -> Self {
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
        let nbpt = Nbpt::from_file("./swap-protocol-both.nbpt.json")
            .expect("File should be converted manually to nbpt.json format using stbx");
        let valid_partition = ValidPartition::new(nbpt.clone().partition()).unwrap();
        let petrinet = Petrinet::from(valid_partition);
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
        assert_eq!(petrinet, expected, "the two petrinets should be identical");
        write_trait(nbpt, "protocol.rs").unwrap();



    }
}
