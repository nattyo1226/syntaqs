use std::{cell::Cell, fmt, fmt::Write, rc::Rc};

use qmb_core::{HilbertSpace, LocalOperator, LocalState, Operator, State, TensorKind};

type LO = LocalOperator<&'static str, &'static str>;
type LS = LocalState<&'static str, &'static str, &'static str>;
type O = Operator<&'static str, &'static str, &'static str>;
type S = State<&'static str, &'static str, &'static str, &'static str>;

#[test]
fn local_operator_syntax_preserves_grouping_coefficients_and_order() {
    let a = LO::primitive("A");
    let b = LO::primitive("B");
    let c = LO::primitive("C");
    let sum = LO::sum([a.clone(), b.clone()]);
    let cases = [
        (LO::zero(), "0"),
        (LO::sum([]), "0"),
        (LO::sum([a.clone()]), "A"),
        (LO::sum([a.clone(), a.clone()]), "A + A"),
        (a.scaled("0"), "(0) * A"),
        (a.scaled("1"), "(1) * A"),
        (a.scaled("2+3i"), "(2+3i) * A"),
        (LO::sum([a.clone(), b.scaled("-1+0i")]), "A + (-1+0i) * B"),
        (&sum * &c, "(A + B) * C"),
        (&a * &sum, "A * (A + B)"),
        ((&a * &b) * &c, "A * B * C"),
        (&a * (&b * &c), "A * (B * C)"),
        (LO::sum([a.clone(), sum.clone()]), "A + (A + B)"),
        (LO::sum([sum.clone(), c.clone()]), "(A + B) + C"),
        (sum.scaled("2+3i"), "(2+3i) * (A + B)"),
        (a.scaled("3").scaled("2"), "(2) * ((3) * A)"),
        ((&a * &b).scaled("2"), "(2) * (A * B)"),
        (&a * b.scaled("2"), "A * ((2) * B)"),
        (a.scaled("2") * &b, "((2) * A) * B"),
        (a.adjoint(), "(A)†"),
        ((&a * &b).adjoint(), "(A * B)†"),
        (a.adjoint().adjoint(), "((A)†)†"),
    ];
    for (expr, expected) in cases {
        assert_eq!(expr.to_string(), expected);
    }
    assert_eq!(a.to_string(), "A");
    assert_eq!(sum.to_string(), "A + B");
}

#[test]
fn local_state_placement_propagates_to_each_primitive_and_action() {
    let h = HilbertSpace::factor("a");
    let up = LS::primitive("up");
    let down = LS::primitive("down");
    let sx = LO::primitive("Sx");
    let sy = LO::primitive("Sy");
    let sum = LS::sum([up.clone(), down]);
    let cases = [
        (LS::zero(), "0", "0"),
        (up.clone(), "|up>", "|up[a]>"),
        (sum.clone(), "|up> + |down>", "|up[a]> + |down[a]>"),
        (
            sum.scaled("2+3i"),
            "(2+3i) * (|up> + |down>)",
            "(2+3i) * (|up[a]> + |down[a]>)",
        ),
        (up.act(&sx), "Sx |up>", "Sx[a] |up[a]>"),
        (
            up.act(&(&sx + &sy)),
            "(Sx + Sy) |up>",
            "(Sx + Sy)[a] |up[a]>",
        ),
        (
            up.act(&(&sx * &sy)),
            "(Sx * Sy) |up>",
            "(Sx * Sy)[a] |up[a]>",
        ),
        (
            up.act(&sx.scaled("2")),
            "((2) * Sx) |up>",
            "((2) * Sx)[a] |up[a]>",
        ),
        (
            up.act(&sy).act(&sx),
            "Sx (Sy |up>)",
            "Sx[a] (Sy[a] |up[a]>)",
        ),
        (
            sum.act(&sx),
            "Sx (|up> + |down>)",
            "Sx[a] (|up[a]> + |down[a]>)",
        ),
        (
            up.scaled("2").act(&sx),
            "Sx ((2) * |up>)",
            "Sx[a] ((2) * |up[a]>)",
        ),
        (
            up.act(&sx).scaled("2"),
            "(2) * (Sx |up>)",
            "(2) * (Sx[a] |up[a]>)",
        ),
        (
            LS::sum([sum.clone(), up.clone()]),
            "(|up> + |down>) + |up>",
            "(|up[a]> + |down[a]>) + |up[a]>",
        ),
    ];
    for (local, raw, placed) in cases {
        assert_eq!(local.to_string(), raw);
        assert_eq!(local.at(&h).unwrap().to_string(), placed);
    }
    assert_eq!(LO::zero().at(&h).unwrap().to_string(), "0[a]");
    assert_eq!(sx.adjoint().at(&h).unwrap().to_string(), "((Sx)†)[a]");
}

#[test]
fn composite_operator_and_state_variants_use_the_same_syntax() {
    let h = HilbertSpace::factor("a");
    let a = LO::primitive("A").at(&h).unwrap();
    let b = LO::primitive("B").at(&h).unwrap();
    let sum = O::sum([a.clone(), b.clone()]).unwrap();
    let cases = [
        (O::zero(), "0"),
        (O::sum([]).unwrap(), "0"),
        (O::identity(), "I"),
        (O::product([]).unwrap(), "I"),
        (O::scalar("2"), "(2) * I"),
        (a.scaled("0"), "(0) * A[a]"),
        (sum.clone(), "A[a] + B[a]"),
        ((&a * &b).unwrap(), "A[a] * B[a]"),
        (
            O::product([a.clone(), b.clone(), a.clone()]).unwrap(),
            "A[a] * B[a] * A[a]",
        ),
        ((&a * (&b * &a).unwrap()).unwrap(), "A[a] * (B[a] * A[a])"),
        ((&sum * &a).unwrap(), "(A[a] + B[a]) * A[a]"),
        ((&a * &sum).unwrap(), "A[a] * (A[a] + B[a])"),
        (sum.scaled("2"), "(2) * (A[a] + B[a])"),
        (
            O::sum([a.clone(), sum.clone()]).unwrap(),
            "A[a] + (A[a] + B[a])",
        ),
        (a.scaled("3").scaled("2"), "(2) * ((3) * A[a])"),
        ((&a * b.scaled("2")).unwrap(), "A[a] * ((2) * B[a])"),
        (a.adjoint().adjoint(), "((A[a])†)†"),
    ];
    for (expr, expected) in cases {
        assert_eq!(expr.to_string(), expected);
    }
    let up = LS::primitive("up").at(&h).unwrap();
    let down = LS::primitive("down").at(&h).unwrap();
    let states = S::sum(&h, [up.clone(), down]).unwrap();
    let cases = [
        (S::zero(&h), "0"),
        (S::sum(&h, []).unwrap(), "0"),
        (S::unit(), "unit"),
        (S::tensor_in(&HilbertSpace::unit(), []).unwrap(), "unit"),
        (up.scaled("1"), "(1) * |up[a]>"),
        (states.clone(), "|up[a]> + |down[a]>"),
        (states.scaled("2"), "(2) * (|up[a]> + |down[a]>)"),
        (states.act(&a).unwrap(), "A[a] (|up[a]> + |down[a]>)"),
        (up.act(&sum).unwrap(), "(A[a] + B[a]) |up[a]>"),
        (
            up.act(&(&a * &b).unwrap()).unwrap(),
            "(A[a] * B[a]) |up[a]>",
        ),
        (up.act(&a.scaled("2")).unwrap(), "((2) * A[a]) |up[a]>"),
        (up.act(&b).unwrap().act(&a).unwrap(), "A[a] (B[a] |up[a]>)"),
        (up.scaled("2").act(&a).unwrap(), "A[a] ((2) * |up[a]>)"),
        (up.act(&a).unwrap().scaled("2"), "(2) * (A[a] |up[a]>)"),
    ];
    for (expr, expected) in cases {
        assert_eq!(expr.to_string(), expected);
    }
}

#[test]
fn tensors_compress_only_primitive_factors_and_keep_source_grouping() {
    for kind in [TensorKind::Ordinary, TensorKind::Graded] {
        let factors = ["a", "b", "c"].map(HilbertSpace::factor);
        let h = HilbertSpace::tensor_all_with(kind, factors.clone()).unwrap();
        let [a, b, c] = factors;
        let ab = HilbertSpace::tensor_all_with(kind, [a.clone(), b.clone()]).unwrap();
        let up = LS::primitive("up");
        let down = LS::primitive("down");
        let sa = up.at(&a).unwrap();
        let sb = down.at(&b).unwrap();
        let sc = up.at(&c).unwrap();
        let flat = S::tensor_in(&h, [sa.clone(), sb.clone(), sc.clone()]).unwrap();
        let pair = S::tensor_in(&ab, [sa.clone(), sb.clone()]).unwrap();
        let nested = S::tensor_in(&h, [pair, sc.clone()]).unwrap();
        assert_eq!(flat.to_string(), "|up[a],down[b],up[c]>");
        assert_eq!(nested.to_string(), flat.to_string());
        assert_ne!(format!("{flat:?}"), format!("{nested:?}"));
        assert!(nested.space().is_compatible_with(&h));
        let sum = LS::sum([up.clone(), down.clone()]).at(&a).unwrap();
        let mixed = S::tensor_in(
            &h,
            [
                sum,
                S::tensor_in(
                    &HilbertSpace::tensor_all_with(kind, [b.clone(), c.clone()]).unwrap(),
                    [sb.clone(), sc.clone()],
                )
                .unwrap(),
            ],
        )
        .unwrap();
        assert_eq!(mixed.to_string(), "(|up[a]> + |down[a]>) ⊗ |down[b],up[c]>");
        let with_unit = S::tensor_in(&h, [S::unit(), flat.clone()]).unwrap();
        assert_eq!(with_unit.to_string(), "unit ⊗ |up[a],down[b],up[c]>");
        assert_eq!(
            with_unit.scaled("2").to_string(),
            "(2) * (unit ⊗ |up[a],down[b],up[c]>)"
        );
        let grouped = S::tensor_in(&h, [S::unit(), with_unit]).unwrap();
        assert_eq!(grouped.to_string(), "unit ⊗ (unit ⊗ |up[a],down[b],up[c]>)");
        let scaled = S::tensor_in(&ab, [sa.scaled("2"), sb.clone()]).unwrap();
        assert_eq!(scaled.to_string(), "((2) * |up[a]>) ⊗ |down[b]>");
        let op = LO::primitive("a†").at(&b).unwrap();
        let acting = S::tensor_in(&ab, [sa.clone(), sb.act(&op).unwrap()]).unwrap();
        assert_eq!(acting.to_string(), "|up[a]> ⊗ (a†[b] |down[b]>)");
        assert_eq!(
            flat.act(&op).unwrap().to_string(),
            "a†[b] |up[a],down[b],up[c]>"
        );
        assert_eq!(
            flat.act(&op).unwrap().to_string(),
            flat.act(&op.bind(&h).unwrap()).unwrap().to_string()
        );
        assert_eq!(op.to_string(), op.bind(&h).unwrap().to_string());
        assert!(op.space().is_none());
        assert_eq!(
            (&op * LO::primitive("a").at(&a).unwrap())
                .unwrap()
                .to_string(),
            "a†[b] * a[a]"
        );
    }
}

#[test]
fn pretty_layout_and_outer_flags_do_not_change_leaf_content() {
    let [a, b, c] = ["a", "b", "c"].map(HilbertSpace::factor);
    let h = HilbertSpace::tensor_all([a.clone(), b.clone(), c.clone()]).unwrap();
    let state = S::tensor_in(
        &h,
        [
            LS::primitive("up").at(&a).unwrap(),
            LS::primitive("down").at(&b).unwrap(),
            LS::primitive("up").at(&c).unwrap(),
        ],
    )
    .unwrap();
    let op = LO::sum([LO::primitive("A"), LO::primitive("B"), LO::primitive("C")]);
    assert_eq!(format!("{op:#}"), "A\n+ B\n+ C");
    let placed = O::sum([
        LO::primitive("A").at(&a).unwrap(),
        LO::primitive("B").at(&b).unwrap(),
        LO::primitive("C").at(&c).unwrap(),
    ])
    .unwrap();
    assert_eq!(
        format!("{:#}", state.act(&placed).unwrap()),
        "(\n    A[a]\n    + B[b]\n    + C[c]\n) |up[a],\n    down[b],\n    up[c]>"
    );
    assert_eq!(
        format!("{:#}", op.at(&a).unwrap()),
        "(\n    A\n    + B\n    + C\n)[a]"
    );
    assert_eq!(
        format!("{:#}", LS::primitive("up").at(&a).unwrap()),
        "|up[a]>"
    );
    assert_eq!(format!("{:#}", S::zero(&h)), "0");
    assert_eq!(format!("{:#}", S::unit()), "unit");
    let sum = LS::sum([LS::primitive("up"), LS::primitive("down")]);
    let mixed = S::tensor_in(&a, [S::unit(), sum.at(&a).unwrap()]).unwrap();
    assert_eq!(
        format!("{:#}", mixed.scaled("2")),
        "(2) * (\n    unit\n    ⊗ (\n        |up[a]>\n        + |down[a]>\n    )\n)"
    );
}

// Deliberately lacks Clone, Eq, Hash, Ord, Debug and all algebra/numeric traits.
struct Label {
    text: &'static str,
    calls: Rc<Cell<usize>>,
}
impl Label {
    fn new(text: &'static str, calls: &Rc<Cell<usize>>) -> Self {
        Self {
            text,
            calls: calls.clone(),
        }
    }
}
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        assert!(!f.alternate());
        assert_eq!(f.width(), None);
        assert_eq!(f.precision(), None);
        self.calls.set(self.calls.get() + 1);
        // Multiple writes, deliberately ignoring precision.
        for ch in self.text.chars() {
            f.write_char(ch)?;
        }
        Ok(())
    }
}

#[test]
fn display_only_nonclone_leaves_and_index_support_streaming_unicode_prefixes() {
    for (raw, prefix) in [
        ("", ""),
        ("1234567", "1234567"),
        ("12345678", "12345678"),
        ("123456789", "12345678"),
        ("あいうえおかきくけ", "あいうえおかきく"),
        ("ab😀e\u{301}xyz末尾", "ab😀e\u{301}xyz"),
    ] {
        let calls = Rc::new(Cell::new(0));
        let factor = HilbertSpace::factor(Label::new(raw, &calls));
        let local = LocalState::<_, Label, Label>::primitive(Label::new("up", &calls));
        let op = LocalOperator::<_, Label>::primitive(Label::new("Sx", &calls));
        let expr = local
            .act(&op)
            .scaled(Label::new("c", &calls))
            .at(&factor)
            .unwrap();
        assert_eq!(
            format!("{expr:#30.2}"),
            format!("(c) * (Sx[{prefix}] |up[{prefix}]>)")
        );
        assert_eq!(calls.get(), 5); // coefficient, operator, state, and two index appearances
        assert_eq!(local.to_string(), "|up>");
        assert_eq!(op.to_string(), "Sx");
        assert_eq!(factor.factor_index().unwrap().to_string(), raw);
    }
    // None of these types has formatting capabilities; construction still works.
    struct Opaque;
    let space = HilbertSpace::factor(Opaque);
    let state = LocalState::<_, Opaque, Opaque>::primitive(Opaque)
        .at(&space)
        .unwrap();
    let op = LocalOperator::<_, Opaque>::primitive(Opaque)
        .at(&space)
        .unwrap();
    drop((state, op));
}

struct FailsAfter {
    bytes: usize,
}
impl fmt::Write for FailsAfter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > self.bytes {
            return Err(fmt::Error);
        }
        self.bytes -= s.len();
        Ok(())
    }
}

#[test]
fn sink_errors_propagate_without_formatting_later_leaves() {
    let calls = Rc::new(Cell::new(0));
    let op = LocalOperator::<_, i32>::primitive(Label::new("ABC", &calls));
    let sum = LocalOperator::sum([op.clone(), op]);
    assert!(write!(FailsAfter { bytes: 2 }, "{sum}").is_err());
    assert_eq!(calls.get(), 1);
    let factor = HilbertSpace::factor(Label::new("123456789", &calls));
    let placed = LS::primitive("up").at(&factor).unwrap();
    calls.set(0);
    assert!(write!(FailsAfter { bytes: 5 }, "{placed}").is_err());
    assert_eq!(calls.get(), 1);
    let mut sink = String::new();
    write!(sink, "{placed}").unwrap();
    assert_eq!(sink, "|up[12345678]>");
}

#[test]
fn debug_keeps_management_fields_and_full_index() {
    #[derive(Debug)]
    struct DebugOnly;
    let opaque = LocalState::<_, DebugOnly, DebugOnly>::primitive(DebugOnly);
    assert!(format!("{opaque:?}").contains("DebugOnly"));
    let h = HilbertSpace::factor("full-index-longer-than-eight");
    let local = LS::primitive("up").act(&LO::primitive("Sx"));
    let state = local.at(&h).unwrap();
    let op = LO::primitive("Sy").at(&h).unwrap().bind(&h).unwrap();
    assert_eq!(state.to_string(), "Sx[full-ind] |up[full-ind]>");
    for debug in [
        format!("{state:?}"),
        format!("{state:#?}"),
        format!("{op:?}"),
        format!("{h:?}"),
    ] {
        assert!(debug.contains("full-index-longer-than-eight"));
        assert!(debug.contains("Factor"));
    }
    assert!(format!("{local:?}").contains("Act"));
    assert!(format!("{local:#?}").contains("has_action"));
    assert!(format!("{state:?}").contains("space"));
    assert!(format!("{op:#?}").contains("ambient"));
}

#[derive(Default)]
struct Length(usize);
impl fmt::Write for Length {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0 += s.len();
        Ok(())
    }
}
fn length(expr: &impl fmt::Display) -> usize {
    let mut sink = Length::default();
    write!(sink, "{expr}").unwrap();
    sink.0
}

#[test]
fn deep_local_and_composite_syntax_formats_and_drops_on_a_small_stack() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            const DEPTH: usize = 20_000;
            let h = HilbertSpace::factor("a");
            let up = LS::primitive("up");
            let a = LO::primitive("A");
            let mut local_action = up.clone();
            let mut local_scale = up.clone();
            let mut local_sum = up.clone();
            let mut local_product = a.clone();
            let mut local_adjoint = a.clone();
            let mut local_op_scale = a.clone();
            let mut local_op_sum = a.clone();
            let leaf = up.at(&h).unwrap();
            let op = a.at(&h).unwrap();
            let mut action = leaf.clone();
            let mut scale = leaf.clone();
            let mut sum = leaf.clone();
            let mut product = op.clone();
            let mut adjoint = op.clone();
            let mut op_scale = op.clone();
            let mut op_sum = op.clone();
            let mut tensor = leaf.clone();
            for _ in 0..DEPTH {
                local_action = local_action.act(&a);
                local_scale = local_scale.scaled("1");
                local_sum = LS::sum([local_sum, LS::zero()]);
                local_product = local_product * &a;
                local_adjoint = local_adjoint.adjoint();
                local_op_scale = local_op_scale.scaled("1");
                local_op_sum = LO::sum([local_op_sum, LO::zero()]);
                action = action.act(&op).unwrap();
                scale = scale.scaled("1");
                sum = S::sum(&h, [sum, S::zero(&h)]).unwrap();
                product = (product * &op).unwrap();
                adjoint = adjoint.adjoint();
                op_scale = op_scale.scaled("1");
                op_sum = O::sum([op_sum, O::zero()]).unwrap();
                tensor = S::tensor_in(&h, [S::unit(), tensor]).unwrap();
            }
            assert_eq!(length(&local_action), 4 * DEPTH + 2);
            assert_eq!(length(&local_scale), 8 * DEPTH + 2);
            assert_eq!(length(&local_sum), 6 * DEPTH + 2);
            assert_eq!(length(&local_product), 4 * DEPTH + 1);
            assert_eq!(length(&local_adjoint), 5 * DEPTH + 1); // † is three UTF-8 bytes
            assert_eq!(length(&local_op_scale), 8 * DEPTH - 1);
            assert_eq!(length(&local_op_sum), 6 * DEPTH - 1);
            assert_eq!(length(&action), 7 * DEPTH + 5);
            assert_eq!(length(&scale), 8 * DEPTH + 5);
            assert_eq!(length(&sum), 6 * DEPTH + 5);
            assert_eq!(length(&product), 7 * DEPTH + 4);
            assert_eq!(length(&adjoint), 5 * DEPTH + 4);
            assert_eq!(length(&op_scale), 8 * DEPTH + 2);
            assert_eq!(length(&op_sum), 6 * DEPTH + 2);
            assert_eq!(length(&tensor), 11 * DEPTH + 5); // ⊗ is three UTF-8 bytes
            // Cross local/composite boundaries without recursive formatting.
            assert!(length(&local_action.at(&h).unwrap().act(&product).unwrap()) > DEPTH);
            assert!(length(&leaf.act(&local_adjoint.at(&h).unwrap()).unwrap()) > DEPTH);
            // Errors must also drop a deep pending stack without recursing.
            assert!(write!(FailsAfter { bytes: 32 }, "{local_adjoint:#}").is_err());
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn index_display_errors_are_not_hidden_after_reaching_the_prefix_limit() {
    struct BadIndex;
    impl fmt::Display for BadIndex {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("12345")?;
            f.write_str("6789")?;
            f.write_str("discarded")?;
            Err(fmt::Error)
        }
    }
    let h = HilbertSpace::factor(BadIndex);
    let state = LS::primitive("up").at(&h).unwrap();
    let mut out = String::new();
    assert!(write!(out, "{state}").is_err());
    assert_eq!(out, "|up[12345678");
}
