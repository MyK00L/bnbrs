use bitset_fixed::BitSet;
use std::io::stdin;

#[derive(Clone, Debug)]
struct MaxCliqueState {
    k: BitSet,
    siz: usize,
    deg: Vec<u32>,
    took: BitSet,
}
impl MaxCliqueState {
    fn fork_on(&self) -> Option<usize> {
        let max = self.deg.iter().enumerate().max_by_key(|x| x.1);
        max.and_then(|x| if *x.1 == 0 { None } else { Some(x.0) })
    }
    fn fork0(&mut self, _g: &[BitSet], i: usize) {
        self.deg[i] = 0;
        self.k.set(i, false);
    }
    fn fork1(&mut self, g: &[BitSet], i: usize) {
        self.siz += 1;
        self.took.set(i, true);
        self.k &= &g[i];
        for (i, gi) in g.iter().enumerate() {
            if self.deg[i] != 0 {
                self.deg[i] = if self.k[i] {
                    (self.k.clone() & gi).count_ones()
                } else {
                    0
                }
            }
        }
        self.fork0(g, i);
    }
    fn gcp_with_order(&self, g: &[BitSet], v: &[(u32, usize)]) -> u32 {
        let mut isets = vec![BitSet::new(g.len())];
        for (i, gi) in g.iter().enumerate() {
            if self.took[i] {
                isets[0] |= gi;
            }
        }
        for o in v.iter() {
            let i = o.1;
            let mut inserted = false;
            for iset in isets.iter_mut() {
                if !iset[i] {
                    *iset |= &g[i];
                    inserted = true;
                    break;
                }
            }
            if !inserted {
                isets.push(g[i].clone());
            }
        }
        isets.len() as u32
    }
    fn gcp(&self, g: &[BitSet]) -> u32 {
        let mut v = Vec::<(u32, usize)>::new();
        for i in 0..g.len() {
            if self.deg[i] != 0 {
                v.push((self.deg[i], i));
            }
        }
        v.sort_by_key(|x| g.len() as u32 - x.0);
        let mut res = self.gcp_with_order(g, &v);
        v.sort_by_key(|x| x.0);
        res = res.min(self.gcp_with_order(g, &v));
        res + self.siz as u32
    }
}
impl bnbrs::State for MaxCliqueState {
    type Rt = u32;
    type Problem = Vec<BitSet>;
    fn lb(&self, _g: &Self::Problem) -> u32 {
        self.siz as u32
    }
    fn ub(&self, g: &Self::Problem) -> u32 {
        self.gcp(g)
    }
    fn children(&self, g: &Self::Problem) -> Vec<Self> {
        if let Some(i) = self.fork_on() {
            let mut c0 = self.clone();
            let mut c1 = self.clone();
            c0.fork0(g, i);
            c1.fork1(g, i);
            vec![c1, c0]
        } else {
            vec![]
        }
    }
    fn root(g: &Self::Problem) -> Self {
        MaxCliqueState {
            k: !BitSet::new(g.len()),
            siz: 0,
            deg: g.iter().map(|x| x.count_ones()).collect(),
            took: BitSet::new(g.len()),
        }
    }
}

#[derive(Default)]
struct Scanner {
    buffer: Vec<String>,
}
impl Scanner {
    fn next<T: std::str::FromStr>(&mut self) -> T {
        loop {
            if let Some(token) = self.buffer.pop() {
                return token.parse().ok().expect("Failed parse");
            }
            let mut input = String::new();
            stdin().read_line(&mut input).expect("Failed read");
            self.buffer = input.split_whitespace().rev().map(String::from).collect();
        }
    }
}

fn main() {
    let mut scan = Scanner::default();
    let n = scan.next::<usize>();
    let m = scan.next::<usize>();
    let mut g = vec![BitSet::new(n); n];
    for (i, gi) in g.iter_mut().enumerate().take(n) {
        gi.set(i, true);
    }
    for _ in 0..m {
        let u = scan.next::<usize>() - 1;
        let v = scan.next::<usize>() - 1;
        g[u].set(v, true);
        g[v].set(u, true);
    }
    let res = bnbrs::find_minimum::<bnbrs::MonotoneLb<bnbrs::Maximize<MaxCliqueState>>>(&g);
    println!("{}", res.0 .0);
}
