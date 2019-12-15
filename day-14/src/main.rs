use std::{borrow, cmp, collections::BTreeMap, collections::HashMap, hash, str::FromStr};

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Eq)]
struct Quantity {
    name: String,
    amount: u64,
}

impl Quantity {
    fn key(&self) -> &str {
        &self.name
    }
}

impl hash::Hash for Quantity {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.key().hash(state)
    }
}

impl cmp::PartialEq for Quantity {
    fn eq(&self, other: &Self) -> bool {
        self.key().eq(other.key())
    }
}

impl borrow::Borrow<str> for Quantity {
    fn borrow(&self) -> &str {
        self.key()
    }
}

impl FromStr for Quantity {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.trim().splitn(2, " ");
        let amount = parts.next().ok_or("Missing quantity")?.parse()?;
        let name = parts.next().ok_or("Missing name")?.into();

        Ok(Quantity { amount, name })
    }
}

#[derive(Debug)]
struct Reactions(HashMap<Quantity, Vec<Quantity>>);

impl FromStr for Reactions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut map = HashMap::new();

        for l in s.trim().lines() {
            let l = l.trim();
            let mut parts = l.splitn(2, "=>");
            let dependencies = parts.next().ok_or("Missing dependencies")?;
            let dependencies = dependencies
                .split(",")
                .map(str::parse)
                .collect::<Result<_, _>>()?;
            let output = parts.next().ok_or("Missing output")?.parse()?;

            map.insert(output, dependencies);
        }

        Ok(Reactions(map))
    }
}

const ORE: &str = "ORE";
const FUEL: &str = "FUEL";

impl Reactions {
    pub fn solve(&self) -> u64 {
        let mut requirements = BTreeMap::<_, u64>::new();
        let mut next_requirements = BTreeMap::<_, u64>::new();
        let mut excess = BTreeMap::<_, u64>::new();
        let mut ore_count = 0;

        requirements.insert(FUEL, 1);

        while !requirements.is_empty() {
            // eprintln!("\n\n{:?}", requirements);
            // eprintln!("extra: {:?}", excess);

            for (requirement_name, requirement_amount) in requirements {
                // eprintln!("\nEvaluating {} ({})", requirement_name, requirement_amount);
                if requirement_name == ORE {
                    ore_count += requirement_amount;
                    continue;
                }

                let (output, dependencies) = self.producing(requirement_name);
                // eprintln!("Rule: {:?} <= {:?}", output, dependencies);

                // Use up extra from before
                let previously_produced = excess.remove(requirement_name).unwrap_or(0);

                let effective_requirement_amount =
                    requirement_amount.saturating_sub(previously_produced);

                let multiplier = rounding_up(effective_requirement_amount, output.amount);
                // eprintln!("Multiplier is {}", multiplier);

                let overproduction = output.amount * multiplier - effective_requirement_amount;
                *excess.entry(requirement_name).or_insert(0) += overproduction;
                // eprintln!("Overproducing {} by {}", requirement_name, overproduction);

                for dependency in dependencies {
                    *next_requirements.entry(&*dependency.name).or_insert(0) +=
                        dependency.amount * multiplier;
                }
            }

            requirements = next_requirements;
            next_requirements = BTreeMap::new();
        }

        ore_count
    }

    fn producing(&self, name: &str) -> (&Quantity, &[Quantity]) {
        let (k, v) = self
            .0
            .get_key_value(name)
            .unwrap_or_else(|| panic!("{} not found in reactions", name));
        (k, &**v)
    }
}

fn rounding_up(a: u64, b: u64) -> u64 {
    match (a / b, a % b) {
        (v, 0) => v,
        (v, _) => v + 1,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() -> Result<()> {
        let reactions: Reactions = r#"
            10 ORE => 10 A
            1 ORE => 1 B
            7 A, 1 B => 1 C
            7 A, 1 C => 1 D
            7 A, 1 D => 1 E
            7 A, 1 E => 1 FUEL
        "#
        .parse()?;
        assert_eq!(reactions.solve(), 31);
        Ok(())
    }

    #[test]
    fn example_2() -> Result<()> {
        let reactions: Reactions = r#"
            9 ORE => 2 A
            8 ORE => 3 B
            7 ORE => 5 C
            3 A, 4 B => 1 AB
            5 B, 7 C => 1 BC
            4 C, 1 A => 1 CA
            2 AB, 3 BC, 4 CA => 1 FUEL
        "#
        .parse()?;
        assert_eq!(reactions.solve(), 165);
        Ok(())
    }

    #[test]
    fn example_3() -> Result<()> {
        let reactions: Reactions = r#"
            157 ORE => 5 NZVS
            165 ORE => 6 DCFZ
            44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
            12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
            179 ORE => 7 PSHF
            177 ORE => 5 HKGWZ
            7 DCFZ, 7 PSHF => 2 XJWVT
            165 ORE => 2 GPVTF
            3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT
        "#
        .parse()?;
        assert_eq!(reactions.solve(), 13312);
        Ok(())
    }

    #[test]
    fn example_4() -> Result<()> {
        let reactions: Reactions = r#"
            2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
            17 NVRVD, 3 JNWZP => 8 VPVL
            53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
            22 VJHF, 37 MNCFX => 5 FWMGM
            139 ORE => 4 NVRVD
            144 ORE => 7 JNWZP
            5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
            5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
            145 ORE => 6 MNCFX
            1 NVRVD => 8 CXFTF
            1 VJHF, 6 MNCFX => 4 RFSQX
            176 ORE => 6 VJHF
        "#
        .parse()?;
        assert_eq!(reactions.solve(), 180697);
        Ok(())
    }

    #[test]
    fn example_5() -> Result<()> {
        let reactions: Reactions = r#"
            171 ORE => 8 CNZTR
            7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
            114 ORE => 4 BHXH
            14 VRPVC => 6 BMBT
            6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
            6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
            15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
            13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
            5 BMBT => 4 WPTQ
            189 ORE => 9 KTJDG
            1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
            12 VRPVC, 27 CNZTR => 2 XDBXC
            15 KTJDG, 12 BHXH => 5 XCVML
            3 BHXH, 2 VRPVC => 7 MZWV
            121 ORE => 7 VRPVC
            7 XCVML => 6 RJRHP
            5 BHXH, 4 VRPVC => 5 LTCX
        "#
        .parse()?;
        assert_eq!(reactions.solve(), 2210736);
        Ok(())
    }
}

const INPUT: &str = include_str!("input.txt");

fn main() -> Result<()> {
    let reactions: Reactions = INPUT.parse()?;
    println!("{}", reactions.solve());
    Ok(())
}
