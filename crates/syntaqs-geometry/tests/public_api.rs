use syntaqs_geometry::prelude::*;

#[test]
fn prelude_constructs_geometry_and_maps_relation_groups() {
    let chain = Chain::new(3, Boundary::Open);
    let neighbors: Vec<_> = NearestNeighbors.groups(&chain).unwrap().collect();
    assert_eq!(neighbors, [[0, 1], [1, 2]]);
    let values = ['a', 'b', 'c'];
    let mapper = SiteMapper::new(&chain, &values).unwrap();
    let pairs: Vec<_> = mapper
        .groups(&Consecutive::<2>)
        .unwrap()
        .map(|group| group.unwrap().iter().copied().collect::<Vec<_>>())
        .collect();
    assert_eq!(pairs, [vec!['a', 'b'], vec!['b', 'c']]);
}
