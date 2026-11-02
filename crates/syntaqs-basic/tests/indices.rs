use core::num::NonZeroUsize;
use syntaqs_basic::Id128;
use syntaqs_core::HilbertSpace;

#[test]
fn uuid_generation_and_core_connection_preserve_existing_identity() {
    let a = Id128::new();
    let b = Id128::new();
    assert_ne!(a, b);
    assert_ne!(a, Id128::default());
    assert_ne!(Id128::default(), Id128::default());
    assert_eq!(core::mem::size_of::<Id128>(), 16);
    fn cloned<T: Clone>(value: &T) -> T {
        value.clone()
    }
    assert_eq!(cloned(&a), a);
    let copied = a;
    assert_eq!(copied, a);
    let mut set = std::collections::HashSet::new();
    set.insert(a);
    assert!(set.contains(&copied));
    let display = a.to_string();
    assert_eq!(display.len(), 36);
    assert_eq!(&display[14..15], "4");
    assert!(matches!(&display[19..20], "8" | "9" | "a" | "b"));
    let factor = HilbertSpace::factor(a);
    let copy = factor.clone();
    assert_eq!(copy.factor_index(), Some(&a));
    assert!(copy.is_compatible_with(&factor));
    let space = HilbertSpace::tensor_all([factor, HilbertSpace::factor(b)]).unwrap();
    assert_eq!(space.indices().copied().collect::<Vec<_>>(), vec![a, b]);
    assert_eq!(space.dim_with(NonZeroUsize::new(1).unwrap()), Ok(1));
}

#[test]
fn tag_names_define_identity_and_are_only_shortened_in_expressions() {
    use syntaqs_basic::Tag;
    use syntaqs_core::{LocalOperator, LocalState, SpaceError};
    let tag = Tag::new("spin-left-long");
    let same = Tag::from(String::from("spin-left-long"));
    assert_eq!(tag, same);
    assert_eq!(tag.clone(), same);
    assert_eq!(tag.as_str(), "spin-left-long");
    assert_eq!(tag.to_string(), "spin-left-long");
    assert_eq!(format!("{tag:?}"), "Tag { raw: \"spin-left-long\" }");
    let mut set = std::collections::HashSet::new();
    set.insert(tag.clone());
    assert!(set.contains(&same));
    assert_eq!(set.len(), 1);
    assert!(Tag::from("a") < Tag::from("b"));
    assert_ne!(Tag::from("é"), Tag::from("e\u{301}"));
    for raw in ["", "日本語の名前", "[,]\n"] {
        assert_eq!(Tag::from(raw).to_string(), raw);
    }
    let factor = syntaqs_core::HilbertSpace::factor(tag);
    let other = syntaqs_core::HilbertSpace::factor(same);
    assert!(matches!(
        factor.tensor(&other),
        Err(SpaceError::DuplicateIndex)
    ));
    let state = LocalState::<_, &str, i32>::primitive("up")
        .at(&factor)
        .unwrap();
    let op = LocalOperator::<_, i32>::primitive("Sx")
        .at(&factor)
        .unwrap();
    assert_eq!(state.to_string(), "|up[spin-lef]>");
    assert_eq!(op.to_string(), "Sx[spin-lef]");
    assert!(format!("{state:?}").contains("spin-left-long"));
    // Distinct full identities may intentionally share a Display prefix.
    let distinct = syntaqs_core::HilbertSpace::factor(Tag::from("spin-left-other"));
    assert!(factor.tensor(&distinct).is_ok());
    assert_eq!(
        LocalOperator::<_, i32>::primitive("Sx")
            .at(&distinct)
            .unwrap()
            .to_string(),
        op.to_string()
    );
}

#[test]
fn uuid_standalone_and_debug_remain_full_while_expression_display_is_short() {
    let id = Id128::new();
    let full = id.to_string();
    assert!(format!("{id:?}").contains(&full));
    let factor = HilbertSpace::factor(id);
    let state = syntaqs_core::LocalState::<_, &str, i32>::primitive("up")
        .at(&factor)
        .unwrap();
    assert_eq!(state.to_string(), format!("|up[{}]>", &full[..8]));
    assert!(format!("{state:#?}").contains(&full));
}
