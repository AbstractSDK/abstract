use crate::AbstractInterfaceError;
use std::collections::HashMap;
use std::{collections::HashSet, hash::Hash};

pub fn diff<K, V>(
    scraped_entries: HashMap<K, V>,
    on_chain_entries: HashMap<K, V>,
) -> Result<(HashSet<K>, HashMap<K, V>), AbstractInterfaceError>
where
    K: Eq + Hash + Clone,
    V: Clone + PartialEq,
{
    let union_keys = get_union_keys(&scraped_entries, &on_chain_entries);
    Ok(get_changes(
        &union_keys,
        &scraped_entries,
        &on_chain_entries,
    ))
}

fn get_union_keys<'a, K, V>(
    scraped_entries: &'a HashMap<K, V>,
    on_chain_entries: &'a HashMap<K, V>,
) -> Vec<&'a K>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    let on_chain_binding = on_chain_entries.keys().collect::<HashSet<_>>();
    let scraped_binding = scraped_entries.keys().collect::<HashSet<_>>();

    on_chain_binding.union(&scraped_binding).cloned().collect()
}

fn get_changes<K, V>(
    union_keys: &Vec<&K>,
    scraped_entries: &HashMap<K, V>,
    on_chain_entries: &HashMap<K, V>,
) -> (HashSet<K>, HashMap<K, V>)
where
    K: Eq + Hash + Clone,
    V: Clone + PartialEq,
{
    let mut to_remove: HashSet<K> = HashSet::new();
    let mut to_add: HashMap<K, V> = HashMap::new();

    for entry in union_keys {
        if !scraped_entries.contains_key(entry) {
            to_remove.insert((*entry).clone());
        } else if !on_chain_entries.contains_key(*entry) {
            let val = scraped_entries.get(*entry).unwrap();
            to_add.insert((*entry).to_owned(), val.clone());
        } else {
            // If the values are not the same we still update
            let val_scraped = scraped_entries.get(*entry).unwrap();
            let val_on_chain = on_chain_entries.get(*entry).unwrap();
            if val_scraped != val_on_chain {
                to_add.insert((*entry).to_owned(), val_scraped.clone());
            }
        }
    }
    (to_remove, to_add)
}
