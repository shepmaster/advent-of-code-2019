use std::collections::BTreeMap;

type Graph<'a> = BTreeMap<&'a str, &'a str>;

fn graph(orbits: &str) -> Graph<'_> {
    let mut graph = Graph::new();

    for l in orbits.trim().lines() {
        let mut parts = l.splitn(2, ")");
        let inner = parts.next().expect("Missing inner").trim();
        let outer = parts.next().expect("Missing outer").trim();

        graph.entry(outer).or_insert(inner);
    }

    graph
}

fn checksum(orbits: &str) -> u32 {
    let graph = graph(orbits);

    let mut cnt = 0;
    for mut orbiter in graph.keys() {
        while let Some(next) = graph.get(orbiter) {
            cnt += 1;
            orbiter = next;
        }
    }

    cnt
}

#[test]
fn specifications() {
    let orbits = r#"
        COM)B
        B)C
        C)D
        D)E
        E)F
        B)G
        G)H
        D)I
        E)J
        J)K
        K)L
    "#;

    assert_eq!(checksum(orbits), 42);
}

fn path<'a>(graph: &Graph<'a>, mut node: &'a str) -> Vec<&'a str> {
    let mut path = Vec::new();

    while let Some(&next) = graph.get(node) {
        path.push(next);
        node = next;
    }

    path
}

fn transfers(orbits: &str) -> Option<usize> {
    let graph = graph(orbits);

    let you_path = path(&graph, "YOU");
    let santa_path = path(&graph, "SAN");

    for (i1, step) in you_path.iter().enumerate() {
        if let Some((i2, _)) = santa_path.iter().enumerate().find(|&(_, v)| v == step) {
            return Some(i1 + i2);
        }
    }

    None
}

#[test]
fn specifications_2() {
    let orbits = r#"
        COM)B
        B)C
        C)D
        D)E
        E)F
        B)G
        G)H
        D)I
        E)J
        J)K
        K)L
        K)YOU
        I)SAN
    "#;

    assert_eq!(transfers(orbits), Some(4));
}

const INPUT: &str = include_str!("input.txt");

fn main() {
    println!("{}", checksum(INPUT));
    println!("{:?}", transfers(INPUT));
}
