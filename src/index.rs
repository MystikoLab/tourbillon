use std::collections::HashMap;

//TODO: Add aliases and ranking support
#[derive(Debug)]
pub struct RawIndex<'a> {
    pub name: &'a str,
    pub alternate_names: &'a [&'a str],
}

#[derive(Debug, Clone)]
pub struct WeightedIndex<'a> {
    pub ranking: i32,
    pub name: &'a str,
    pub aliases: &'a [String],
    pub default_aliases: &'a [&'a str],
}

impl<'a> RawIndex<'a> {
    pub fn weigh_index(
        &self,
        alias_config: &'a HashMap<String, Vec<String>>,
        rank_index: &HashMap<String, i32>,
    ) -> WeightedIndex<'a> {
        let name = self.name.to_lowercase();

        let aliases = alias_config.get(&name).map_or_default(Vec::as_slice);

        let ranking = rank_index.get(&name).copied().unwrap_or(0);

        WeightedIndex {
            ranking,
            name: self.name,
            aliases,
            default_aliases: self.alternate_names,
        }
    }
}

impl<'a> WeightedIndex<'a> {
    pub fn rank_against_query(&self, query: &str) -> i32 {
        let mut comparison = compare(query, self.name);

        for alias in self.default_aliases {
            let new_comparison = compare(query, alias);
            if new_comparison > comparison {
                comparison = new_comparison
            }
        }

        for alias in self.aliases {
            let new_comparison = compare(query, alias);
            if new_comparison > comparison {
                comparison = new_comparison
            }
        }

        if comparison < query.len() as i32 {
            0
        } else {
            ((comparison + self.ranking) / 2) + 1
        }
    }
}

fn compare(query: &str, content: &str) -> i32 {
    let query_chars = query
        .chars()
        .flat_map(char::to_lowercase)
        .collect::<Vec<char>>();
    let mut chars_in_query_found = 0;
    let mut current_index = 0;

    for current_letter in content.to_lowercase().chars() {
        if Some(&current_letter) == query_chars.get(current_index) {
            current_index += 1;
            chars_in_query_found += 1;
        }

        if current_index == query.len() {
            break;
        }
    }

    chars_in_query_found
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_comparison() {
        assert_eq!(compare("aari", "safari"), 4);
        assert_eq!(compare("aari", "mail"), 1);

        assert_eq!(compare("sys", "sytem"), 2);
        assert_eq!(compare("sac", "s"), 1);
    }

    #[test]
    fn test_rank_against_query() {
        let mut index = WeightedIndex {
            aliases: &[],
            default_aliases: &[],
            ranking: 4,
            name: "Safari",
        };

        assert_eq!(index.rank_against_query("aari"), 5);

        index.ranking = 2;

        assert_eq!(index.rank_against_query("aari"), 4);

        let aliases = [String::from("browser")];
        index.aliases = &aliases;
        assert_eq!(index.rank_against_query("bower"), 4);

        index.ranking = 0;
        assert_eq!(index.rank_against_query("bower"), 3);
    }
}
