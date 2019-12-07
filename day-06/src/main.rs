use std::collections::BTreeMap;

fn checksum(orbits: &str) -> u32 {
    let mut graph = BTreeMap::new();

    for l in orbits.trim().lines() {
        let mut parts = l.splitn(2, ")");
        let inner = parts.next().expect("Missing inner").trim();
        let outer = parts.next().expect("Missing outer").trim();

        graph.entry(outer).or_insert(inner);
    }

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

const INPUT: &str = include_str!("input.txt");

fn main() {
    println!("{}", checksum(INPUT));
}
