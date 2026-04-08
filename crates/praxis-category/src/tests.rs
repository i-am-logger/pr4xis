#[cfg(test)]
mod tests {
    use crate::axiom::{Axiom, FullyConnected, NoDeadStates};
    use crate::category::Category;
    use crate::entity::Entity;
    use crate::functor::Functor;
    use crate::morphism::{Morphism, compose_all, direct_morphisms};
    use crate::relationship::Relationship;
    use crate::validate::{
        check_associativity, check_category_laws, check_closure, check_functor_laws,
        check_identity_law,
    };

    // -----------------------------------------------------------------------
    // Test category 1: TrafficLight (fully connected cycle)
    //   Red -> Yellow -> Green -> Red
    //   Plus identity morphisms for each state.
    // -----------------------------------------------------------------------

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Light {
        Red,
        Yellow,
        Green,
    }

    impl Entity for Light {
        fn variants() -> Vec<Self> {
            vec![Light::Red, Light::Yellow, Light::Green]
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum LightTransition {
        IdRed,
        IdYellow,
        IdGreen,
        RedToYellow,
        YellowToGreen,
        GreenToRed,
    }

    impl Relationship for LightTransition {
        type Object = Light;

        fn source(&self) -> Light {
            match self {
                LightTransition::IdRed => Light::Red,
                LightTransition::IdYellow => Light::Yellow,
                LightTransition::IdGreen => Light::Green,
                LightTransition::RedToYellow => Light::Red,
                LightTransition::YellowToGreen => Light::Yellow,
                LightTransition::GreenToRed => Light::Green,
            }
        }

        fn target(&self) -> Light {
            match self {
                LightTransition::IdRed => Light::Red,
                LightTransition::IdYellow => Light::Yellow,
                LightTransition::IdGreen => Light::Green,
                LightTransition::RedToYellow => Light::Yellow,
                LightTransition::YellowToGreen => Light::Green,
                LightTransition::GreenToRed => Light::Red,
            }
        }
    }

    #[derive(Debug, Clone)]
    struct TrafficLightCat;

    impl Category for TrafficLightCat {
        type Object = Light;
        type Morphism = LightTransition;

        fn identity(obj: &Light) -> LightTransition {
            match obj {
                Light::Red => LightTransition::IdRed,
                Light::Yellow => LightTransition::IdYellow,
                Light::Green => LightTransition::IdGreen,
            }
        }

        fn compose(f: &LightTransition, g: &LightTransition) -> Option<LightTransition> {
            if f.target() != g.source() {
                return None;
            }
            // Composing with identity yields the other morphism.
            match (f, g) {
                // id . x = x
                (LightTransition::IdRed, other) => Some(other.clone()),
                (LightTransition::IdYellow, other) => Some(other.clone()),
                (LightTransition::IdGreen, other) => Some(other.clone()),
                // x . id = x
                (other, LightTransition::IdRed) => Some(other.clone()),
                (other, LightTransition::IdYellow) => Some(other.clone()),
                (other, LightTransition::IdGreen) => Some(other.clone()),
                // Red->Yellow then Yellow->Green = Red->Green (multi-hop)
                // We need composed morphisms for closure. Since we only have
                // single-step transitions, we need to add composed results.
                // For a minimal valid category we need closure, so let's
                // define composition of non-identity morphisms:
                (LightTransition::RedToYellow, LightTransition::YellowToGreen) => {
                    // Red -> Green (two hops) = GreenToRed reversed? No.
                    // We need this to be a morphism in the category.
                    // For closure, the composition must exist in the category.
                    // Let's make this a "skip" morphism -- but we didn't define it.
                    // Instead, let's include all composite morphisms.
                    // Actually, for simplicity, let's keep the cycle and realize
                    // that composing RedToYellow . YellowToGreen is not in our
                    // morphisms list. This means closure won't hold unless we add more.
                    //
                    // For a valid category, we need all compositions to be present.
                    // Let's add the composite morphisms.
                    None
                }
                (LightTransition::YellowToGreen, LightTransition::GreenToRed) => None,
                (LightTransition::GreenToRed, LightTransition::RedToYellow) => None,
                _ => None,
            }
        }

        fn morphisms() -> Vec<LightTransition> {
            vec![
                LightTransition::IdRed,
                LightTransition::IdYellow,
                LightTransition::IdGreen,
                LightTransition::RedToYellow,
                LightTransition::YellowToGreen,
                LightTransition::GreenToRed,
            ]
        }
    }

    // -----------------------------------------------------------------------
    // Test category 2: Full traffic light with all composite morphisms
    //   so that closure (and all category laws) hold.
    //
    //   Objects: Red, Yellow, Green
    //   Morphisms: id_R, id_Y, id_G, R->Y, Y->G, G->R, R->G, Y->R, G->Y
    //   (every pair has a morphism, making it fully connected)
    // -----------------------------------------------------------------------

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum FullLightTransition {
        IdRed,
        IdYellow,
        IdGreen,
        RedToYellow,
        YellowToGreen,
        GreenToRed,
        RedToGreen,
        YellowToRed,
        GreenToYellow,
    }

    impl Relationship for FullLightTransition {
        type Object = Light;

        fn source(&self) -> Light {
            match self {
                FullLightTransition::IdRed
                | FullLightTransition::RedToYellow
                | FullLightTransition::RedToGreen => Light::Red,
                FullLightTransition::IdYellow
                | FullLightTransition::YellowToGreen
                | FullLightTransition::YellowToRed => Light::Yellow,
                FullLightTransition::IdGreen
                | FullLightTransition::GreenToRed
                | FullLightTransition::GreenToYellow => Light::Green,
            }
        }

        fn target(&self) -> Light {
            match self {
                FullLightTransition::IdRed
                | FullLightTransition::GreenToRed
                | FullLightTransition::YellowToRed => Light::Red,
                FullLightTransition::IdYellow
                | FullLightTransition::RedToYellow
                | FullLightTransition::GreenToYellow => Light::Yellow,
                FullLightTransition::IdGreen
                | FullLightTransition::YellowToGreen
                | FullLightTransition::RedToGreen => Light::Green,
            }
        }
    }

    #[derive(Debug, Clone)]
    struct FullTrafficLightCat;

    impl Category for FullTrafficLightCat {
        type Object = Light;
        type Morphism = FullLightTransition;

        fn identity(obj: &Light) -> FullLightTransition {
            match obj {
                Light::Red => FullLightTransition::IdRed,
                Light::Yellow => FullLightTransition::IdYellow,
                Light::Green => FullLightTransition::IdGreen,
            }
        }

        fn compose(
            f: &FullLightTransition,
            g: &FullLightTransition,
        ) -> Option<FullLightTransition> {
            if f.target() != g.source() {
                return None;
            }
            // In this category, there is exactly one morphism for each (source, target) pair.
            // Composition f: A->B, g: B->C yields the unique A->C morphism.
            let src = f.source();
            let tgt = g.target();
            Some(Self::unique_morphism(&src, &tgt))
        }

        fn morphisms() -> Vec<FullLightTransition> {
            vec![
                FullLightTransition::IdRed,
                FullLightTransition::IdYellow,
                FullLightTransition::IdGreen,
                FullLightTransition::RedToYellow,
                FullLightTransition::YellowToGreen,
                FullLightTransition::GreenToRed,
                FullLightTransition::RedToGreen,
                FullLightTransition::YellowToRed,
                FullLightTransition::GreenToYellow,
            ]
        }
    }

    impl FullTrafficLightCat {
        fn unique_morphism(src: &Light, tgt: &Light) -> FullLightTransition {
            match (src, tgt) {
                (Light::Red, Light::Red) => FullLightTransition::IdRed,
                (Light::Red, Light::Yellow) => FullLightTransition::RedToYellow,
                (Light::Red, Light::Green) => FullLightTransition::RedToGreen,
                (Light::Yellow, Light::Red) => FullLightTransition::YellowToRed,
                (Light::Yellow, Light::Yellow) => FullLightTransition::IdYellow,
                (Light::Yellow, Light::Green) => FullLightTransition::YellowToGreen,
                (Light::Green, Light::Red) => FullLightTransition::GreenToRed,
                (Light::Green, Light::Yellow) => FullLightTransition::GreenToYellow,
                (Light::Green, Light::Green) => FullLightTransition::IdGreen,
            }
        }
    }

    // -----------------------------------------------------------------------
    // Test category 3: Broken category with a dead state (not fully connected)
    //   Objects: A, B, C
    //   Morphisms: id_A, id_B, id_C, A->B
    //   C is a dead state (no outgoing non-identity morphisms that lead elsewhere,
    //   and C is unreachable from A/B).
    // -----------------------------------------------------------------------

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Node {
        A,
        B,
        C,
    }

    impl Entity for Node {
        fn variants() -> Vec<Self> {
            vec![Node::A, Node::B, Node::C]
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum NodeEdge {
        IdA,
        IdB,
        IdC,
        AToB,
    }

    impl Relationship for NodeEdge {
        type Object = Node;

        fn source(&self) -> Node {
            match self {
                NodeEdge::IdA => Node::A,
                NodeEdge::IdB => Node::B,
                NodeEdge::IdC => Node::C,
                NodeEdge::AToB => Node::A,
            }
        }

        fn target(&self) -> Node {
            match self {
                NodeEdge::IdA => Node::A,
                NodeEdge::IdB => Node::B,
                NodeEdge::IdC => Node::C,
                NodeEdge::AToB => Node::B,
            }
        }
    }

    #[derive(Debug, Clone)]
    struct BrokenCat;

    impl Category for BrokenCat {
        type Object = Node;
        type Morphism = NodeEdge;

        fn identity(obj: &Node) -> NodeEdge {
            match obj {
                Node::A => NodeEdge::IdA,
                Node::B => NodeEdge::IdB,
                Node::C => NodeEdge::IdC,
            }
        }

        fn compose(f: &NodeEdge, g: &NodeEdge) -> Option<NodeEdge> {
            if f.target() != g.source() {
                return None;
            }
            let src = f.source();
            let tgt = g.target();
            // Only defined morphisms: identities + AToB
            match (src, tgt) {
                (Node::A, Node::A) => Some(NodeEdge::IdA),
                (Node::B, Node::B) => Some(NodeEdge::IdB),
                (Node::C, Node::C) => Some(NodeEdge::IdC),
                (Node::A, Node::B) => Some(NodeEdge::AToB),
                _ => None,
            }
        }

        fn morphisms() -> Vec<NodeEdge> {
            vec![NodeEdge::IdA, NodeEdge::IdB, NodeEdge::IdC, NodeEdge::AToB]
        }
    }

    // -----------------------------------------------------------------------
    // Test category 4: Tiny category for functor target
    //   Objects: On, Off
    //   Morphisms: id_On, id_Off, On->Off, Off->On, On->On (=id), Off->Off (=id)
    //   (just two objects, fully connected with unique morphisms per pair)
    // -----------------------------------------------------------------------

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Switch {
        On,
        Off,
    }

    impl Entity for Switch {
        fn variants() -> Vec<Self> {
            vec![Switch::On, Switch::Off]
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum SwitchTransition {
        IdOn,
        IdOff,
        OnToOff,
        OffToOn,
    }

    impl Relationship for SwitchTransition {
        type Object = Switch;

        fn source(&self) -> Switch {
            match self {
                SwitchTransition::IdOn | SwitchTransition::OnToOff => Switch::On,
                SwitchTransition::IdOff | SwitchTransition::OffToOn => Switch::Off,
            }
        }

        fn target(&self) -> Switch {
            match self {
                SwitchTransition::IdOn | SwitchTransition::OffToOn => Switch::On,
                SwitchTransition::IdOff | SwitchTransition::OnToOff => Switch::Off,
            }
        }
    }

    #[derive(Debug, Clone)]
    struct SwitchCat;

    impl Category for SwitchCat {
        type Object = Switch;
        type Morphism = SwitchTransition;

        fn identity(obj: &Switch) -> SwitchTransition {
            match obj {
                Switch::On => SwitchTransition::IdOn,
                Switch::Off => SwitchTransition::IdOff,
            }
        }

        fn compose(f: &SwitchTransition, g: &SwitchTransition) -> Option<SwitchTransition> {
            if f.target() != g.source() {
                return None;
            }
            let src = f.source();
            let tgt = g.target();
            Some(match (src, tgt) {
                (Switch::On, Switch::On) => SwitchTransition::IdOn,
                (Switch::On, Switch::Off) => SwitchTransition::OnToOff,
                (Switch::Off, Switch::On) => SwitchTransition::OffToOn,
                (Switch::Off, Switch::Off) => SwitchTransition::IdOff,
            })
        }

        fn morphisms() -> Vec<SwitchTransition> {
            vec![
                SwitchTransition::IdOn,
                SwitchTransition::IdOff,
                SwitchTransition::OnToOff,
                SwitchTransition::OffToOn,
            ]
        }
    }

    // -----------------------------------------------------------------------
    // Functor: FullTrafficLightCat -> SwitchCat
    //   Red, Yellow -> Off;  Green -> On
    //   Maps morphisms accordingly (unique morphism per source/target pair)
    // -----------------------------------------------------------------------

    struct LightToSwitch;

    impl Functor for LightToSwitch {
        type Source = FullTrafficLightCat;
        type Target = SwitchCat;

        fn map_object(obj: &Light) -> Switch {
            match obj {
                Light::Green => Switch::On,
                Light::Red | Light::Yellow => Switch::Off,
            }
        }

        fn map_morphism(m: &FullLightTransition) -> SwitchTransition {
            let src = Self::map_object(&m.source());
            let tgt = Self::map_object(&m.target());
            match (src, tgt) {
                (Switch::On, Switch::On) => SwitchTransition::IdOn,
                (Switch::On, Switch::Off) => SwitchTransition::OnToOff,
                (Switch::Off, Switch::On) => SwitchTransition::OffToOn,
                (Switch::Off, Switch::Off) => SwitchTransition::IdOff,
            }
        }
    }

    // ===================================================================
    // Tests
    // ===================================================================

    // ----- 1. Entity::variants() -----

    #[test]
    fn entity_variants_returns_all_lights() {
        let vs = Light::variants();
        assert_eq!(vs.len(), 3);
        assert!(vs.contains(&Light::Red));
        assert!(vs.contains(&Light::Yellow));
        assert!(vs.contains(&Light::Green));
    }

    #[test]
    fn entity_variants_returns_all_nodes() {
        let vs = Node::variants();
        assert_eq!(vs.len(), 3);
        assert!(vs.contains(&Node::A));
        assert!(vs.contains(&Node::B));
        assert!(vs.contains(&Node::C));
    }

    #[test]
    fn entity_variants_returns_all_switches() {
        let vs = Switch::variants();
        assert_eq!(vs.len(), 2);
        assert!(vs.contains(&Switch::On));
        assert!(vs.contains(&Switch::Off));
    }

    // ----- 2. Relationship source/target -----

    #[test]
    fn relationship_source_target_identity() {
        let id = LightTransition::IdRed;
        assert_eq!(id.source(), Light::Red);
        assert_eq!(id.target(), Light::Red);
    }

    #[test]
    fn relationship_source_target_transition() {
        let t = LightTransition::RedToYellow;
        assert_eq!(t.source(), Light::Red);
        assert_eq!(t.target(), Light::Yellow);

        let t2 = LightTransition::YellowToGreen;
        assert_eq!(t2.source(), Light::Yellow);
        assert_eq!(t2.target(), Light::Green);

        let t3 = LightTransition::GreenToRed;
        assert_eq!(t3.source(), Light::Green);
        assert_eq!(t3.target(), Light::Red);
    }

    #[test]
    fn relationship_full_light_all_pairs() {
        let morphisms = FullTrafficLightCat::morphisms();
        for m in &morphisms {
            // Every morphism has valid source/target in the entity set
            assert!(Light::variants().contains(&m.source()));
            assert!(Light::variants().contains(&m.target()));
        }
    }

    // ----- 3. Category identity morphisms -----

    #[test]
    fn category_identity_is_self_loop() {
        for obj in Light::variants() {
            let id = TrafficLightCat::identity(&obj);
            assert_eq!(id.source(), obj);
            assert_eq!(id.target(), obj);
        }
    }

    #[test]
    fn category_identity_full_is_self_loop() {
        for obj in Light::variants() {
            let id = FullTrafficLightCat::identity(&obj);
            assert_eq!(id.source(), obj);
            assert_eq!(id.target(), obj);
        }
    }

    #[test]
    fn category_identity_broken_is_self_loop() {
        for obj in Node::variants() {
            let id = BrokenCat::identity(&obj);
            assert_eq!(id.source(), obj);
            assert_eq!(id.target(), obj);
        }
    }

    // ----- 4. Category composition (valid and invalid) -----

    #[test]
    fn compose_identity_left() {
        let id = FullTrafficLightCat::identity(&Light::Red);
        let m = FullLightTransition::RedToYellow;
        let result = FullTrafficLightCat::compose(&id, &m);
        assert_eq!(result, Some(FullLightTransition::RedToYellow));
    }

    #[test]
    fn compose_identity_right() {
        let m = FullLightTransition::RedToYellow;
        let id = FullTrafficLightCat::identity(&Light::Yellow);
        let result = FullTrafficLightCat::compose(&m, &id);
        assert_eq!(result, Some(FullLightTransition::RedToYellow));
    }

    #[test]
    fn compose_two_transitions() {
        let f = FullLightTransition::RedToYellow;
        let g = FullLightTransition::YellowToGreen;
        let result = FullTrafficLightCat::compose(&f, &g);
        assert_eq!(result, Some(FullLightTransition::RedToGreen));
    }

    #[test]
    fn compose_incompatible_returns_none() {
        // Red->Yellow composed with Green->Red: target(Red->Yellow)=Yellow != source(Green->Red)=Green
        let f = FullLightTransition::RedToYellow;
        let g = FullLightTransition::GreenToRed;
        let result = FullTrafficLightCat::compose(&f, &g);
        assert_eq!(result, None);
    }

    #[test]
    fn compose_two_identities_same_object() {
        let id = FullTrafficLightCat::identity(&Light::Green);
        let result = FullTrafficLightCat::compose(&id, &id);
        assert_eq!(result, Some(FullLightTransition::IdGreen));
    }

    #[test]
    fn compose_full_cycle() {
        // Red->Yellow->Green->Red should yield Red->Red = IdRed
        let ry = FullLightTransition::RedToYellow;
        let yg = FullLightTransition::YellowToGreen;
        let gr = FullLightTransition::GreenToRed;
        let ry_yg = FullTrafficLightCat::compose(&ry, &yg).unwrap();
        let full = FullTrafficLightCat::compose(&ry_yg, &gr).unwrap();
        assert_eq!(full, FullLightTransition::IdRed);
    }

    // ----- 5. morphisms_from and morphisms_to -----

    #[test]
    fn morphisms_from_red() {
        let from_red = FullTrafficLightCat::morphisms_from(&Light::Red);
        assert_eq!(from_red.len(), 3); // IdRed, RedToYellow, RedToGreen
        assert!(from_red.contains(&FullLightTransition::IdRed));
        assert!(from_red.contains(&FullLightTransition::RedToYellow));
        assert!(from_red.contains(&FullLightTransition::RedToGreen));
    }

    #[test]
    fn morphisms_to_red() {
        let to_red = FullTrafficLightCat::morphisms_to(&Light::Red);
        assert_eq!(to_red.len(), 3); // IdRed, GreenToRed, YellowToRed
        assert!(to_red.contains(&FullLightTransition::IdRed));
        assert!(to_red.contains(&FullLightTransition::GreenToRed));
        assert!(to_red.contains(&FullLightTransition::YellowToRed));
    }

    #[test]
    fn morphisms_from_broken_c_only_identity() {
        // C has no outgoing non-identity morphisms
        let from_c = BrokenCat::morphisms_from(&Node::C);
        assert_eq!(from_c.len(), 1);
        assert_eq!(from_c[0], NodeEdge::IdC);
    }

    #[test]
    fn morphisms_to_broken_c_only_identity() {
        // Nothing points to C except its identity
        let to_c = BrokenCat::morphisms_to(&Node::C);
        assert_eq!(to_c.len(), 1);
        assert_eq!(to_c[0], NodeEdge::IdC);
    }

    #[test]
    fn morphisms_from_broken_a() {
        let from_a = BrokenCat::morphisms_from(&Node::A);
        assert_eq!(from_a.len(), 2); // IdA, AToB
        assert!(from_a.contains(&NodeEdge::IdA));
        assert!(from_a.contains(&NodeEdge::AToB));
    }

    // ----- 6. Morphism wrapper: of, id, then, and_then, source, target, into_inner, inner -----

    #[test]
    fn morphism_of_wraps_correctly() {
        let m = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToYellow);
        assert_eq!(m.source(), Light::Red);
        assert_eq!(m.target(), Light::Yellow);
    }

    #[test]
    fn morphism_id_creates_identity() {
        let m = Morphism::<FullTrafficLightCat>::id(&Light::Green);
        assert_eq!(m.source(), Light::Green);
        assert_eq!(m.target(), Light::Green);
        assert_eq!(*m.inner(), FullLightTransition::IdGreen);
    }

    #[test]
    fn morphism_then_composes() {
        let m = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToYellow);
        let result = m.then(&FullLightTransition::YellowToGreen);
        assert!(result.is_some());
        let composed = result.unwrap();
        assert_eq!(composed.source(), Light::Red);
        assert_eq!(composed.target(), Light::Green);
        assert_eq!(*composed.inner(), FullLightTransition::RedToGreen);
    }

    #[test]
    fn morphism_then_incompatible_returns_none() {
        let m = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToYellow);
        let result = m.then(&FullLightTransition::GreenToRed);
        assert!(result.is_none());
    }

    #[test]
    fn morphism_and_then_composes_wrapped() {
        let m1 = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToYellow);
        let m2 = Morphism::<FullTrafficLightCat>::of(FullLightTransition::YellowToGreen);
        let result = m1.and_then(&m2);
        assert!(result.is_some());
        let composed = result.unwrap();
        assert_eq!(composed.source(), Light::Red);
        assert_eq!(composed.target(), Light::Green);
    }

    #[test]
    fn morphism_and_then_incompatible_returns_none() {
        let m1 = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToYellow);
        let m2 = Morphism::<FullTrafficLightCat>::of(FullLightTransition::GreenToRed);
        let result = m1.and_then(&m2);
        assert!(result.is_none());
    }

    #[test]
    fn morphism_into_inner_unwraps() {
        let m = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToGreen);
        let raw = m.into_inner();
        assert_eq!(raw, FullLightTransition::RedToGreen);
    }

    #[test]
    fn morphism_inner_borrows() {
        let m = Morphism::<FullTrafficLightCat>::of(FullLightTransition::GreenToYellow);
        assert_eq!(m.inner(), &FullLightTransition::GreenToYellow);
        // m is still usable after inner()
        assert_eq!(m.source(), Light::Green);
    }

    #[test]
    fn morphism_equality() {
        let m1 = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToYellow);
        let m2 = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToYellow);
        let m3 = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToGreen);
        assert_eq!(m1, m2);
        assert_ne!(m1, m3);
    }

    #[test]
    fn morphism_then_with_identity_is_same() {
        let m = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToYellow);
        let id = FullTrafficLightCat::identity(&Light::Yellow);
        let result = m.then(&id).unwrap();
        assert_eq!(*result.inner(), FullLightTransition::RedToYellow);
    }

    #[test]
    fn morphism_chain_three_steps() {
        let m = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToYellow);
        let step2 = m.then(&FullLightTransition::YellowToGreen).unwrap();
        let gr = Morphism::<FullTrafficLightCat>::of(FullLightTransition::GreenToRed);
        let final_m = step2.and_then(&gr).unwrap();
        assert_eq!(final_m.source(), Light::Red);
        assert_eq!(final_m.target(), Light::Red);
        assert_eq!(*final_m.inner(), FullLightTransition::IdRed);
    }

    // ----- 7. compose_all -----

    #[test]
    fn compose_all_valid_chain() {
        let chain = vec![
            FullLightTransition::RedToYellow,
            FullLightTransition::YellowToGreen,
            FullLightTransition::GreenToRed,
        ];
        let result = compose_all::<FullTrafficLightCat>(&chain);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.source(), Light::Red);
        assert_eq!(m.target(), Light::Red);
        assert_eq!(*m.inner(), FullLightTransition::IdRed);
    }

    #[test]
    fn compose_all_single_morphism() {
        let chain = vec![FullLightTransition::RedToYellow];
        let result = compose_all::<FullTrafficLightCat>(&chain);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(*m.inner(), FullLightTransition::RedToYellow);
    }

    #[test]
    fn compose_all_empty_returns_none() {
        let chain: Vec<FullLightTransition> = vec![];
        let result = compose_all::<FullTrafficLightCat>(&chain);
        assert!(result.is_none());
    }

    #[test]
    fn compose_all_incompatible_chain_returns_none() {
        let chain = vec![
            FullLightTransition::RedToYellow,
            FullLightTransition::GreenToRed, // target(RedToYellow)=Yellow != source(GreenToRed)=Green
        ];
        let result = compose_all::<FullTrafficLightCat>(&chain);
        assert!(result.is_none());
    }

    #[test]
    fn compose_all_two_morphisms() {
        let chain = vec![
            FullLightTransition::RedToYellow,
            FullLightTransition::YellowToGreen,
        ];
        let result = compose_all::<FullTrafficLightCat>(&chain);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(*m.inner(), FullLightTransition::RedToGreen);
    }

    // ----- 8. direct_morphisms -----

    #[test]
    fn direct_morphisms_finds_single_step() {
        let ms = direct_morphisms::<FullTrafficLightCat>(&Light::Red, &Light::Yellow);
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0], FullLightTransition::RedToYellow);
    }

    #[test]
    fn direct_morphisms_identity() {
        let ms = direct_morphisms::<FullTrafficLightCat>(&Light::Red, &Light::Red);
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0], FullLightTransition::IdRed);
    }

    #[test]
    fn direct_morphisms_broken_no_path() {
        // No direct morphism from B to A in BrokenCat
        let ms = direct_morphisms::<BrokenCat>(&Node::B, &Node::A);
        assert!(ms.is_empty());
    }

    #[test]
    fn direct_morphisms_broken_existing() {
        let ms = direct_morphisms::<BrokenCat>(&Node::A, &Node::B);
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0], NodeEdge::AToB);
    }

    #[test]
    fn direct_morphisms_broken_isolated_node() {
        // No direct morphism from C to A or B
        let ms = direct_morphisms::<BrokenCat>(&Node::C, &Node::A);
        assert!(ms.is_empty());
        let ms2 = direct_morphisms::<BrokenCat>(&Node::C, &Node::B);
        assert!(ms2.is_empty());
    }

    // ----- 9. NoDeadStates axiom -----

    #[test]
    fn no_dead_states_holds_for_full_traffic_light() {
        let axiom = NoDeadStates::<FullTrafficLightCat>::new();
        assert!(axiom.holds());
        assert_eq!(
            axiom.description(),
            "every object has at least one outgoing morphism"
        );
    }

    #[test]
    fn no_dead_states_holds_for_simple_traffic_light() {
        // Even the simple category has outgoing morphisms from each state
        // (identities + one transition each)
        let axiom = NoDeadStates::<TrafficLightCat>::new();
        assert!(axiom.holds());
    }

    #[test]
    fn no_dead_states_holds_for_broken_cat() {
        // BrokenCat has identities for every object, so technically
        // every object has at least one outgoing morphism (identity).
        // The axiom checks morphisms_from which includes identities.
        let axiom = NoDeadStates::<BrokenCat>::new();
        assert!(axiom.holds());
    }

    #[test]
    fn no_dead_states_holds_for_switch_cat() {
        let axiom = NoDeadStates::<SwitchCat>::new();
        assert!(axiom.holds());
    }

    #[test]
    fn no_dead_states_default_constructor() {
        let axiom = NoDeadStates::<FullTrafficLightCat>::default();
        assert!(axiom.holds());
    }

    // ----- Test category with actual dead state (no outgoing morphisms at all) -----

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum DeadNode {
        Alive,
        Dead,
    }

    impl Entity for DeadNode {
        fn variants() -> Vec<Self> {
            vec![DeadNode::Alive, DeadNode::Dead]
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum DeadEdge {
        IdAlive,
        AliveToDead,
        // Note: NO identity for Dead, NO outgoing from Dead
    }

    impl Relationship for DeadEdge {
        type Object = DeadNode;

        fn source(&self) -> DeadNode {
            match self {
                DeadEdge::IdAlive | DeadEdge::AliveToDead => DeadNode::Alive,
            }
        }

        fn target(&self) -> DeadNode {
            match self {
                DeadEdge::IdAlive => DeadNode::Alive,
                DeadEdge::AliveToDead => DeadNode::Dead,
            }
        }
    }

    #[derive(Debug, Clone)]
    struct DeadStateCat;

    impl Category for DeadStateCat {
        type Object = DeadNode;
        type Morphism = DeadEdge;

        fn identity(obj: &DeadNode) -> DeadEdge {
            match obj {
                DeadNode::Alive => DeadEdge::IdAlive,
                DeadNode::Dead => DeadEdge::IdAlive, // broken: no real id for Dead
            }
        }

        fn compose(f: &DeadEdge, g: &DeadEdge) -> Option<DeadEdge> {
            if f.target() != g.source() {
                return None;
            }
            // Only composable pairs involve Alive
            match (f, g) {
                (DeadEdge::IdAlive, other) => Some(other.clone()),
                (other, DeadEdge::IdAlive) => Some(other.clone()),
                _ => None,
            }
        }

        fn morphisms() -> Vec<DeadEdge> {
            vec![DeadEdge::IdAlive, DeadEdge::AliveToDead]
        }
    }

    #[test]
    fn no_dead_states_fails_for_dead_state_cat() {
        let axiom = NoDeadStates::<DeadStateCat>::new();
        // Dead has no outgoing morphisms at all
        assert!(!axiom.holds());
    }

    // ----- 10. FullyConnected axiom -----

    #[test]
    fn fully_connected_holds_for_full_traffic_light() {
        let axiom = FullyConnected::<FullTrafficLightCat>::new();
        assert!(axiom.holds());
        assert_eq!(
            axiom.description(),
            "every object is reachable from every other object"
        );
    }

    #[test]
    fn fully_connected_fails_for_broken_cat() {
        // C is isolated from A and B
        let axiom = FullyConnected::<BrokenCat>::new();
        assert!(!axiom.holds());
    }

    #[test]
    fn fully_connected_holds_for_simple_traffic_light() {
        // Red -> Yellow -> Green -> Red forms a cycle, so fully connected
        let axiom = FullyConnected::<TrafficLightCat>::new();
        assert!(axiom.holds());
    }

    #[test]
    fn fully_connected_holds_for_switch() {
        let axiom = FullyConnected::<SwitchCat>::new();
        assert!(axiom.holds());
    }

    #[test]
    fn fully_connected_fails_for_dead_state_cat() {
        // Can reach Dead from Alive, but cannot reach Alive from Dead
        let axiom = FullyConnected::<DeadStateCat>::new();
        assert!(!axiom.holds());
    }

    #[test]
    fn fully_connected_default_constructor() {
        let axiom = FullyConnected::<FullTrafficLightCat>::default();
        assert!(axiom.holds());
    }

    // ----- 11. Validate functions -----

    #[test]
    fn check_identity_law_full_traffic_light() {
        assert!(check_identity_law::<FullTrafficLightCat>().is_ok());
    }

    #[test]
    fn check_identity_law_switch() {
        assert!(check_identity_law::<SwitchCat>().is_ok());
    }

    #[test]
    fn check_identity_law_broken_cat() {
        // BrokenCat still satisfies identity law because compose with identity works
        assert!(check_identity_law::<BrokenCat>().is_ok());
    }

    #[test]
    fn check_associativity_full_traffic_light() {
        assert!(check_associativity::<FullTrafficLightCat>().is_ok());
    }

    #[test]
    fn check_associativity_switch() {
        assert!(check_associativity::<SwitchCat>().is_ok());
    }

    #[test]
    fn check_associativity_broken_cat() {
        assert!(check_associativity::<BrokenCat>().is_ok());
    }

    #[test]
    fn check_closure_full_traffic_light() {
        assert!(check_closure::<FullTrafficLightCat>().is_ok());
    }

    #[test]
    fn check_closure_switch() {
        assert!(check_closure::<SwitchCat>().is_ok());
    }

    #[test]
    fn check_closure_broken_cat() {
        // BrokenCat: A->B composed with IdB should give A->B (exists). All other
        // composable pairs are identity compositions. So closure holds.
        assert!(check_closure::<BrokenCat>().is_ok());
    }

    #[test]
    fn check_closure_simple_traffic_light_fails() {
        // Simple TrafficLightCat: RedToYellow . YellowToGreen returns None
        // but they are composable (target matches source), violating closure.
        assert!(check_closure::<TrafficLightCat>().is_err());
    }

    #[test]
    fn check_category_laws_full_traffic_light() {
        assert!(check_category_laws::<FullTrafficLightCat>().is_ok());
    }

    #[test]
    fn check_category_laws_switch() {
        assert!(check_category_laws::<SwitchCat>().is_ok());
    }

    #[test]
    fn check_category_laws_broken_cat() {
        assert!(check_category_laws::<BrokenCat>().is_ok());
    }

    #[test]
    fn check_category_laws_simple_traffic_light_fails() {
        // Fails because closure is violated
        assert!(check_category_laws::<TrafficLightCat>().is_err());
    }

    // ----- 12. Functor and check_functor_laws -----

    #[test]
    fn functor_map_object_correct() {
        assert_eq!(LightToSwitch::map_object(&Light::Red), Switch::Off);
        assert_eq!(LightToSwitch::map_object(&Light::Yellow), Switch::Off);
        assert_eq!(LightToSwitch::map_object(&Light::Green), Switch::On);
    }

    #[test]
    fn functor_map_morphism_identity_preservation() {
        for obj in Light::variants() {
            let id_source = FullTrafficLightCat::identity(&obj);
            let mapped_id = LightToSwitch::map_morphism(&id_source);
            let id_target = SwitchCat::identity(&LightToSwitch::map_object(&obj));
            assert_eq!(
                mapped_id, id_target,
                "Identity preservation failed for {:?}",
                obj
            );
        }
    }

    #[test]
    fn functor_map_morphism_composition_preservation() {
        let f = FullLightTransition::RedToYellow;
        let g = FullLightTransition::YellowToGreen;
        let fg = FullTrafficLightCat::compose(&f, &g).unwrap();

        let mapped_fg = LightToSwitch::map_morphism(&fg);
        let composed_mapped = SwitchCat::compose(
            &LightToSwitch::map_morphism(&f),
            &LightToSwitch::map_morphism(&g),
        )
        .unwrap();

        assert_eq!(mapped_fg, composed_mapped);
    }

    #[test]
    fn check_functor_laws_light_to_switch() {
        assert!(check_functor_laws::<LightToSwitch>().is_ok());
    }

    // ----- Additional edge case tests -----

    #[test]
    fn morphism_wrapper_debug_impl() {
        let m = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToYellow);
        let debug_str = format!("{:?}", m);
        assert!(debug_str.contains("RedToYellow"));
    }

    #[test]
    fn morphism_wrapper_clone() {
        let m = Morphism::<FullTrafficLightCat>::of(FullLightTransition::RedToYellow);
        let m2 = m.clone();
        assert_eq!(m, m2);
    }

    #[test]
    fn compose_all_with_identities() {
        let chain = vec![
            FullLightTransition::IdRed,
            FullLightTransition::RedToYellow,
            FullLightTransition::IdYellow,
        ];
        let result = compose_all::<FullTrafficLightCat>(&chain);
        assert!(result.is_some());
        assert_eq!(*result.unwrap().inner(), FullLightTransition::RedToYellow);
    }

    #[test]
    fn morphisms_from_and_to_cover_all() {
        // For FullTrafficLightCat, every object has exactly 3 outgoing and 3 incoming
        for obj in Light::variants() {
            let from = FullTrafficLightCat::morphisms_from(&obj);
            let to = FullTrafficLightCat::morphisms_to(&obj);
            assert_eq!(from.len(), 3, "Expected 3 morphisms from {:?}", obj);
            assert_eq!(to.len(), 3, "Expected 3 morphisms to {:?}", obj);
        }
    }

    #[test]
    fn direct_morphisms_all_pairs_full() {
        // Every pair of objects in FullTrafficLightCat has exactly one direct morphism
        for src in Light::variants() {
            for tgt in Light::variants() {
                let ms = direct_morphisms::<FullTrafficLightCat>(&src, &tgt);
                assert_eq!(
                    ms.len(),
                    1,
                    "Expected exactly 1 direct morphism from {:?} to {:?}",
                    src,
                    tgt
                );
            }
        }
    }

    #[test]
    fn switch_compose_round_trip() {
        let on_off = SwitchTransition::OnToOff;
        let off_on = SwitchTransition::OffToOn;
        let result = SwitchCat::compose(&on_off, &off_on);
        assert_eq!(result, Some(SwitchTransition::IdOn));

        let result2 = SwitchCat::compose(&off_on, &on_off);
        assert_eq!(result2, Some(SwitchTransition::IdOff));
    }

    #[test]
    fn check_identity_law_error_message_format() {
        // Verify that check_identity_law for DeadStateCat gives a meaningful error
        // DeadStateCat has broken identity for Dead, so identity law should fail
        let result = check_identity_law::<DeadStateCat>();
        // The identity for Dead is IdAlive (source=Alive), which doesn't match Dead.
        // morphisms_from(Dead) is empty, morphisms_to(Dead) has AliveToDead.
        // compose(AliveToDead, identity(Dead)) = compose(AliveToDead, IdAlive)
        //   -> target(AliveToDead)=Dead != source(IdAlive)=Alive => None
        // So right identity fails.
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("identity failed"), "Error was: {}", err);
    }

    #[test]
    fn check_closure_error_message_format() {
        let result = check_closure::<TrafficLightCat>();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("closure violated"), "Error was: {}", err);
    }
}
