use crate::petrinet::*;
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
    use std::fs::OpenOptions;
    use std::io::Write;
    let Nbpt {
        name,
        names,
        partition,
    } = nbpt;
    let mut trait_name = name;
    let valid_partition = ValidPartition::new(partition.clone()).unwrap();
    let net = Petrinet::from(valid_partition);
    let mut f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(trait_file)
        .unwrap();
    writeln!(f, "// Auto-generated file using petrinet-rs")?;
    let unique_places = partition.unique_sorted_places();
    // the trait type params comma delimited
    let trait_type_params =
        unique_places
            .iter()
            .enumerate()
            .fold("".to_owned(), |mut acc, (i, place)| {
                if i > 0 {
                    acc.push_str(",");
                }
                format!("{}T{:02}", acc, place)
            });
    trait_name.retain(|c| !c.is_whitespace());
    writeln!(f, "trait {}<{}> {{", trait_name, trait_type_params)?;
    let mut fns = "".to_owned();

    for (fn_name, trs) in names.iter().zip(net.transitions().iter()) {
        let (consume, produce) = (trs.consume(), trs.produce());
        let ins = consume
            .iter()
            .enumerate()
            .fold("".to_owned(), |mut acc, (i, place)| {
                if i > 0 {
                    acc.push_str(", ");
                }
                acc.push_str("a");
                format!("{}{:02}: T{:02}", acc, i, place)
            });

        let outs = produce
            .iter()
            .enumerate()
            .fold("".to_owned(), |mut acc, (i, place)| {
                if i > 0 {
                    acc.push_str(", ");
                }
                format!("{}T{:02}", acc, place)
            });
        let formated = format!("  fn {}({}) -> ({});", fn_name, ins, outs);
        writeln!(f, "{}", formated)?;
        fns.push_str(formated.as_str());
        fns.push_str("\n");
    }
    writeln!(f, "}}\n")?;
    let types = trait_type_params.split(",");
    let target_type = "BagOfFuns";
    let (in_type_params, fns) = types.fold(("".to_owned(), fns), |(acc_in, acc_fns), t| {
        writeln!(f, "#[derive(Default)]").unwrap();
        writeln!(f, "struct A{};", t).unwrap();
        let old = format!("{}", t);
        let new = format!("A{}", t);
        let acc_in = format!("{}A{},", acc_in, t);
        let acc_fns = acc_fns.replace(&old, &new);
        (acc_in, acc_fns)
    });
    writeln!(f, "\nstruct {};", target_type)?;
    writeln!(
        f,
        "\nimpl {}<{}> for {} {{",
        trait_name, in_type_params, target_type
    )?;
    let old = ";";
    let new = format!(" {{\n    Default::default()\n  }}");
    let fns = fns.replace(&old, &new);
    writeln!(f, "{}", fns)?;
    writeln!(f, "}}")?;

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
        let xs = partition.unique_sorted_places();
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
    fn unique_sorted_places(self) -> Places {
        let Partition(mut xs) = self;
        xs.sort_unstable();
        xs.dedup();
        xs.into_iter().skip_while(|&y| y == 0).collect()
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
                    tr.push(Transition::new(c, p));
                    c = vec![];
                    p = vec![];
                    consume = !consume;
                }
                (_, true) => c.push(i),
                (_, false) => p.push(i),
            }
        }
        Petrinet::new(tr)
    }
}

mod tests {
    use super::*;
    #[test]
    fn deserialize_convert() {
        let nbpt = Nbpt::from_file("./swap-protocol-both.nbpt.json")
            .expect("File should be converted manually to nbpt.json format using stbx");
        let valid_partition = ValidPartition::new(nbpt.clone().partition()).unwrap();
        let petrinet = Petrinet::from(valid_partition);
        let expected = Petrinet::new(vec![
            Transition::new(vec![2, 15], vec![3, 16]),
            Transition::new(vec![1, 6], vec![2, 11]),
            Transition::new(vec![2, 4], vec![19]),
            Transition::new(vec![7], vec![4, 5]),
            Transition::new(vec![5], vec![6]),
            Transition::new(vec![8, 9], vec![7]),
            Transition::new(vec![10], vec![8, 9]),
            Transition::new(vec![19], vec![3]),
            Transition::new(vec![19], vec![1, 18]),
            Transition::new(vec![13, 11], vec![12, 14]),
            Transition::new(vec![14], vec![15]),
            Transition::new(vec![12, 16], vec![17]),
            Transition::new(vec![12, 18], vec![13]),
        ]);
        assert_eq!(petrinet, expected, "The two petrinets must be identical");
        write_trait(nbpt, "./src/protocol.rs").unwrap();
    }
}
