use std::cmp::{Ordering, Reverse};
use std::fmt::Debug;

pub trait State: Sized + Clone + Debug {
    type Rt: Ord + Sized + Debug + Copy;
    type Problem;
    fn lb(&self, p: &Self::Problem) -> Self::Rt;
    fn ub(&self, p: &Self::Problem) -> Self::Rt;
    fn children(&self, p: &Self::Problem) -> Vec<Self>;
    fn root(p: &Self::Problem) -> Self;
}

#[derive(Debug, Clone)]
pub struct Maximize<S: State>(S);

impl<S: State> State for Maximize<S> {
    type Rt = Reverse<S::Rt>;
    type Problem = S::Problem;
    fn lb(&self, p: &Self::Problem) -> Self::Rt {
        Reverse(self.0.ub(p))
    }
    fn ub(&self, p: &Self::Problem) -> Self::Rt {
        Reverse(self.0.lb(p))
    }
    fn children(&self, p: &Self::Problem) -> Vec<Self> {
        self.0
            .children(p)
            .into_iter()
            .map(|x| Maximize(x))
            .collect()
    }
    fn root(p: &Self::Problem) -> Self {
        Maximize(S::root(p))
    }
}

// lb should be increasing on a path from root to leaf, if it's not, use this
#[derive(Debug, Clone)]
pub struct MonotoneLb<S: State>(S, S::Rt);
impl<S: State> State for MonotoneLb<S> {
    type Rt = S::Rt;
    type Problem = S::Problem;
    fn lb(&self, _p: &Self::Problem) -> Self::Rt {
        self.1
    }
    fn ub(&self, p: &Self::Problem) -> Self::Rt {
        self.0.ub(p)
    }
    fn children(&self, p: &Self::Problem) -> Vec<Self> {
        self.0
            .children(p)
            .into_iter()
            .map(|x| {
                let lb = x.lb(p);
                MonotoneLb(x, lb.max(self.1))
            })
            .collect()
    }
    fn root(p: &Self::Problem) -> Self {
        let root = S::root(p);
        let lb = root.lb(p);
        MonotoneLb(root, lb)
    }
}

struct HeapEl<S: State>((S::Rt, S::Rt), S);
impl<S: State> PartialEq for HeapEl<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<S: State> Eq for HeapEl<S> {}
impl<S: State> Ord for HeapEl<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        std::cmp::Reverse(self.0).cmp(&std::cmp::Reverse(other.0))
    }
}
impl<S: State> PartialOrd for HeapEl<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn find_minimum<S: State>(prob: &S::Problem) -> (S::Rt, S) {
    let mut it = 0u32;
    let mut ans = S::root(prob);
    let mut ub = ans.ub(prob);
    let mut stack = vec![((ans.lb(prob), ub), ans.clone())];
    // first dfs order to find a decent solution
    while let Some(((nlb, nub), node)) = stack.pop() {
        it += 1;
        if nub < ub {
            eprintln!("stack it:{} ub:{:?}", it, nub);
            ub = nub;
            ans = node.clone();
        }
        if nlb >= ub {
            continue;
        }
        let mut children: Vec<((S::Rt, S::Rt), S)> = node
            .children(prob)
            .into_iter()
            .map(|x| ((x.lb(prob), x.ub(prob)), x))
            .filter(|x| x.0 .0 < ub)
            .collect();
        if children.is_empty() {
            // leaf node, switch to A* order
            break;
        }
        children.sort_by_key(|x| x.0);
        stack.append(&mut children);
    }

    let mut heap = std::collections::BinaryHeap::<HeapEl<S>>::new();
    heap.extend(stack.into_iter().map(|x| HeapEl(x.0, x.1)));
    while let Some(HeapEl((nlb, nub), node)) = heap.pop() {
        it += 1;
        if nub < ub {
            eprintln!("heap it:{} ub:{:?}", it, nub);
            ub = nub;
            ans = node.clone();
        }
        if nlb >= ub {
            continue;
        }
        let mut children: Vec<HeapEl<S>> = node
            .children(prob)
            .into_iter()
            .map(|x| HeapEl((x.lb(prob), x.ub(prob)), x))
            .filter(|x| x.0 .0 < ub)
            .collect();
        children.sort_by_key(|x| x.0);
        heap.extend(children.into_iter());
    }
    eprintln!("found {:?} in {} iterations", ub, it);
    (ub, ans)
}
