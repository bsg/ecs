#[cfg(test)]
mod tests {
    use crate::{self as ecs, component, ArchetypeBuilder, With, Without};

    use crate::{Entity, World};

    #[component]
    struct A(u32);

    #[component]
    struct B(bool);

    #[component]
    struct C(Option<&'static str>);

    #[component]
    struct Z {}

    struct Ctx;
    impl ecs::Ctx for Ctx {}

    #[test]
    fn get_component() {
        let world: World<Ctx> = World::new();

        let entity_ref = world.spawn((A(42u32), B(false), C(Some("a"))));
        assert_eq!(entity_ref.0, 1);

        let (_, index) = (*world.inner().entities.get(1).unwrap()).unwrap();
        assert_eq!(index, 0);

        assert_eq!(world.component::<A>(Entity(1)).unwrap().0, 42);
        assert!(!world.component::<B>(Entity(1)).unwrap().0);
        assert_eq!(world.component::<C>(Entity(1)).unwrap().0, Some("a"));

        // repeat in different order
        assert_eq!(world.component::<C>(Entity(1)).unwrap().0, Some("a"));
        assert!(!world.component::<B>(Entity(1)).unwrap().0,);
        assert_eq!(world.component::<A>(Entity(1)).unwrap().0, 42);

        world.component_mut::<A>(Entity(1)).unwrap().0 = 123u32;
        world.component_mut::<B>(Entity(1)).unwrap().0 = true;

        assert_eq!(world.component::<A>(Entity(1)).unwrap().0, 123u32);
        assert!(world.component::<B>(Entity(1)).unwrap().0);

        assert!(world.has_component::<A>(Entity(1)));
        assert!(!world.has_component::<Z>(Entity(1)));
    }

    #[test]
    fn get_nonexisting_component() {
        let world: World<Ctx> = World::new();

        let e = world.spawn(A(42u32));
        assert!(world.component::<B>(e).is_none());
        assert!(world.component_mut::<B>(e).is_none());
    }

    #[test]
    fn query() {
        let world: World<Ctx> = World::new();

        world.spawn((A(1u32), C(Some("1"))));
        world.spawn((A(2u32), C(Some("2")), B(true)));
        world.spawn((A(10u32), C(Some("10"))));

        let mut count = 0;
        world.run(|a: &mut A, c: &C| {
            assert_eq!(Some(a.0.to_string().as_str()), c.0);
            a.0 = 123;
            count += 1;
        });
        assert_eq!(count, 3);

        world.run(|c1: &mut A| {
            assert_eq!(c1.0, 123);
        });
    }

    #[test]
    fn query_with_optional() {
        let world: World<Ctx> = World::new();

        world.spawn((B(true), A(1)));
        world.spawn(B(true));
        world.spawn((B(true), A(5)));
        world.spawn(B(true));

        world.run(|_: &B, a: Option<&mut A>| {
            if let Some(a) = a {
                a.0 = 4;
            }
        });

        let mut sum = 0;
        world.run(|_: &B, a: Option<&A>| {
            if let Some(a) = a {
                sum += a.0;
            }
        });
        assert_eq!(sum, 8);
    }

    #[test]
    fn query_with_entity() {
        let world: World<Ctx> = World::new();

        world.spawn((B(true), A(1)));
        world.spawn(B(true));
        world.spawn((B(true), A(5)));
        world.spawn(B(true));

        let mut sum = 0;
        world.run(|entity: &Entity, _: &B| {
            sum += **entity - 1;
        });

        assert_eq!(sum, 6);
    }

    #[test]
    fn despawn() {
        let world: World<Ctx> = World::new();

        world.spawn(A(1));
        world.spawn(A(2));
        let e = world.spawn(A(3));
        world.spawn(A(4));

        world.despawn(e);

        let mut sum = 0;
        world.run(|a: &A| {
            sum += a.0;
        });

        assert_eq!(sum, 7);
    }

    #[test]
    fn reuse_entity() {
        let world: World<Ctx> = World::new();

        world.spawn(A(1));
        world.spawn(A(2));
        world.spawn(A(3));
        world.spawn(A(4));
        world.spawn(A(5));

        world.despawn(Entity(3));
        assert_eq!(world.spawn(A(3)), Entity(3));
        assert_eq!(world.spawn(A(6)), Entity(6));
    }

    #[test]
    fn with_without() {
        let world: World<Ctx> = World::new();

        world.spawn((A(1), B(false)));
        world.spawn(A(2));
        world.spawn((A(3), B(false)));
        world.spawn(A(4));

        let mut sum = 0;
        world.run(|a: &A, _: Without<B>| {
            sum += a.0;
        });
        assert_eq!(sum, 6);

        let mut sum = 0;
        world.run(|a: &A, _: With<B>| {
            sum += a.0;
        });
        assert_eq!(sum, 4);
    }

    #[test]
    fn add_remove_component() {
        let world: World<Ctx> = World::new();

        let e1 = world.spawn(A(1));
        let e2 = world.spawn(A(2));
        let e3 = world.spawn(A(3));
        let e4 = world.spawn((A(4), C(Some("bar"))));

        world.add_component(e2, C(Some("foo")));
        world.add_component(e2, C(Some("foo")));
        assert!(world.has_component::<A>(e2));
        assert!(world.has_component::<C>(e2));
        assert_eq!(world.component::<A>(e2).unwrap().0, 2);
        assert!(world.has_component::<Entity>(e1));
        assert!(world.has_component::<Entity>(e2));
        assert!(world.has_component::<Entity>(e3));
        assert!(world.has_component::<A>(e1));
        assert!(world.has_component::<A>(e2));

        world.remove_component::<A>(e2);
        world.remove_component::<A>(e2);
        assert!(!world.has_component::<A>(e2));
        assert!(world.has_component::<C>(e2));
        assert!(world.has_component::<Entity>(e2));
        assert!(world.has_component::<Entity>(e4));
        assert!(world.has_component::<A>(e4));
        assert!(world.has_component::<C>(e4));

        world.remove_component::<C>(e2);
    }

    #[test]
    fn for_each_with_archetype() {
        let world: World<Ctx> = World::new();

        world.spawn(A(1));
        world.spawn(A(2));
        let ent = world.spawn((A(1), B(false)));
        world.spawn((A(1), B(false)));
        world.spawn(A(3));
        world.spawn(A(4));

        world.despawn(ent);

        let archetype = ArchetypeBuilder::new().set::<A>().build();
        let mut acc = 0;
        world.for_each_with_archetype(archetype, |ent| {
            acc += world.component::<A>(ent).unwrap().0;
        });
        assert_eq!(10, acc);
    }
}
