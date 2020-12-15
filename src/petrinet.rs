#[derive(Debug, PartialEq, PartialOrd)]
pub struct Petrinet(Vec<Transition>);
impl Petrinet {
    pub fn new(x: Vec<Transition>) -> Self {
        Petrinet(x)
    }
    pub fn place_count(&self) -> usize {
        self.transitions()
            .iter()
            .fold(0, |acc, Transition(consumed, produced)| {
                let pmax = max(&produced);
                let cmax = max(&consumed);
                let m = pmax.max(cmax);
                acc.max(*m).max(1) // require at least one place (which will hold the initial token)
            })
    }
    pub fn transitions(&self) -> &Vec<Transition> {
        &self.0
    }
    pub fn transition(&self, t: usize) -> Option<&Transition> {
        self.transitions().get(t)
    }

    /// nr of transitions
    fn transition_count(&self) -> usize {
        self.transitions().len()
    }
}

pub type Places = Vec<usize>;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Transition(Places, Places);

impl Transition {
    pub fn new(consume: Places, produce: Places) -> Self {
        Transition(consume, produce)
    }
    pub fn consume(&self) -> &Places {
        &self.0
    }
    pub fn produce(&self) -> &Places {
        &self.1
    }
}
fn max(xs: &Places) -> &usize {
    xs.iter().fold(&0, |acc, x| acc.max(x))
}

mod create_and_execute_net {
    use super::*;
    use crate::execution::Execution;
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
        assert_eq!(e.enabled(2), false, "transition 2 must be disabled");
        assert_eq!(e.enabled(3), false, "transition 3 must be disabled");
        assert_eq!(e.enabled(4), false, "transition 4 must be disabled");

        // can't fire transition without all its inputs activated
        let e = e.fire(3);
        assert_eq!(e.enabled(0), false, "transition 0 must be disabled");
        assert_eq!(e.enabled(1), true, "transition 1 must be enabled");
        assert_eq!(e.enabled(2), false, "transition 2 must be disabled");
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
