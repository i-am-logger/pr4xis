use super::category::Category;
use super::entity::Entity;
use super::functor::Functor;
use super::relationship::Relationship;

/// Verify the identity law: for every object A and every morphism f from A,
/// compose(identity(A), f) == f and compose(f, identity(target(f))) == f.
pub fn check_identity_law<C: Category>() -> Result<(), String>
where
    C::Morphism: PartialEq,
{
    for obj in C::Object::variants() {
        let id = C::identity(&obj);

        for m in C::morphisms_from(&obj) {
            let left = C::compose(&id, &m);
            if left.as_ref() != Some(&m) {
                return Err(format!(
                    "left identity failed for object {:?}, morphism {:?}: compose(id, m) = {:?}",
                    obj, m, left
                ));
            }
        }

        for m in C::morphisms_to(&obj) {
            let right = C::compose(&m, &id);
            if right.as_ref() != Some(&m) {
                return Err(format!(
                    "right identity failed for object {:?}, morphism {:?}: compose(m, id) = {:?}",
                    obj, m, right
                ));
            }
        }
    }

    Ok(())
}

/// Verify associativity: for all composable triples f, g, h:
/// compose(compose(f, g), h) == compose(f, compose(g, h))
pub fn check_associativity<C: Category>() -> Result<(), String>
where
    C::Morphism: PartialEq,
{
    let morphisms = C::morphisms();

    for f in &morphisms {
        for g in &morphisms {
            if f.target() != g.source() {
                continue;
            }
            for h in &morphisms {
                if g.target() != h.source() {
                    continue;
                }

                let fg = C::compose(f, g);
                let gh = C::compose(g, h);

                let left = fg.as_ref().and_then(|fg| C::compose(fg, h));
                let right = gh.as_ref().and_then(|gh| C::compose(f, gh));

                if left != right {
                    return Err(format!(
                        "associativity failed: ({:?} ∘ {:?}) ∘ {:?} != {:?} ∘ ({:?} ∘ {:?})",
                        f, g, h, f, g, h
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Verify closure: for all composable pairs f: A→B and g: B→C in morphisms(),
/// compose(f, g) must return Some.
pub fn check_closure<C: Category>() -> Result<(), String> {
    let morphisms = C::morphisms();

    for f in &morphisms {
        for g in &morphisms {
            if f.target() != g.source() {
                continue;
            }
            if C::compose(f, g).is_none() {
                return Err(format!(
                    "closure violated: {:?} (→ {:?}) and {:?} ({:?} →) are composable but compose returned None",
                    f,
                    f.target(),
                    g,
                    g.source()
                ));
            }
        }
    }

    Ok(())
}

/// Verify all category laws (identity + associativity + closure).
pub fn check_category_laws<C: Category>() -> Result<(), String>
where
    C::Morphism: PartialEq,
{
    check_closure::<C>()?;
    check_identity_law::<C>()?;
    check_associativity::<C>()?;
    Ok(())
}

/// Verify functor laws: preserves identity and composition.
pub fn check_functor_laws<F: Functor>() -> Result<(), String>
where
    <F::Source as Category>::Morphism: PartialEq,
    <F::Target as Category>::Morphism: PartialEq,
{
    // Identity preservation: F(id_A) == id_{F(A)}
    for obj in <F::Source as Category>::Object::variants() {
        let id_source = F::Source::identity(&obj);
        let mapped_id = F::map_morphism(&id_source);
        let id_target = F::Target::identity(&F::map_object(&obj));

        if mapped_id != id_target {
            return Err(format!(
                "functor identity law failed for object {:?}: F(id) != id_F",
                obj
            ));
        }
    }

    // Composition preservation: F(g∘f) == F(g)∘F(f)
    let morphisms = F::Source::morphisms();
    for f in &morphisms {
        for g in &morphisms {
            if f.target() != g.source() {
                continue;
            }
            if let Some(gf) = F::Source::compose(f, g) {
                let f_mapped_composed = F::map_morphism(&gf);
                let composed_mapped = F::Target::compose(&F::map_morphism(f), &F::map_morphism(g));

                if composed_mapped.as_ref() != Some(&f_mapped_composed) {
                    return Err(format!(
                        "functor composition law failed for {:?}, {:?}",
                        f, g
                    ));
                }
            }
        }
    }

    Ok(())
}
