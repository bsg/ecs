#[cfg(test)]
mod tests {
    use crate::{self as ecs};
    use codegen::Component;

    use crate::{Entity, World};

    #[derive(Component)]
    struct A(u32);

    #[derive(Component)]
    struct B(bool);

    // FIXME this fails. string probably gets dropped after C is written to store
    // #[derive(Component)]
    // struct C(Option<String>);

    #[derive(Component)]
    struct C(Option<&'static str>);

    #[test]
    fn get_component() {
        let world = World::new();

        let entity_ref = world.spawn(&[&A(42u32), &B(false), &C(Some("a"))]);
        assert_eq!(entity_ref.0, 1);

        let (_, index) = unsafe {
            world
                .entities
                .get()
                .as_ref()
                .unwrap()
                .get(1)
                .unwrap()
                .clone()
                .unwrap()
        };
        assert_eq!(index, 0);

        assert_eq!(world.get_component::<A>(Entity(1)).unwrap().0, 42);
        assert_eq!(world.get_component::<B>(Entity(1)).unwrap().0, false);
        assert_eq!(world.get_component::<C>(Entity(1)).unwrap().0, Some("a"));

        // repeat in different order
        assert_eq!(world.get_component::<C>(Entity(1)).unwrap().0, Some("a"));
        assert_eq!(world.get_component::<B>(Entity(1)).unwrap().0, false);
        assert_eq!(world.get_component::<A>(Entity(1)).unwrap().0, 42);

        world.get_component_mut::<A>(Entity(1)).unwrap().0 = 123u32;
        world.get_component_mut::<B>(Entity(1)).unwrap().0 = true;

        assert_eq!(world.get_component::<A>(Entity(1)).unwrap().0, 123u32);
        assert_eq!(world.get_component::<B>(Entity(1)).unwrap().0, true);
    }

    #[test]
    fn get_nonexisting_component() {
        let world = World::new();

        let e = world.spawn(&[&A(42u32)]);
        assert!(world.get_component::<B>(e).is_none());
        assert!(world.get_component_mut::<B>(e).is_none());
    }

    #[test]
    fn query() {
        let world = World::new();

        world.spawn(&[&A(1u32), &C(Some("1"))]);
        world.spawn(&[&A(2u32), &C(Some("2")), &B(true)]);
        world.spawn(&[&A(10u32), &C(Some("10"))]);

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
        let world = World::new();

        world.spawn(&[&B(true), &A(1)]);
        world.spawn(&[&B(true)]);
        world.spawn(&[&B(true), &A(5)]);
        world.spawn(&[&B(true)]);

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
        let world = World::new();

        world.spawn(&[&B(true), &A(1)]);
        world.spawn(&[&B(true)]);
        world.spawn(&[&B(true), &A(5)]);
        world.spawn(&[&B(true)]);

        let mut sum = 0;
        world.run(|entity: &Entity, _: &B| {
            sum += **entity - 1;
        });

        assert_eq!(sum, 6);
    }

    #[test]
    fn spawn_inside_system() {
        let world = World::new();

        world.spawn(&[&C("1".into())]);
        world.spawn(&[&C("2".into())]);
        world.spawn(&[&C("3".into())]);

        world.run(|c: &C| {
            world.spawn(&[&A(str::parse::<u32>(c.0.unwrap()).unwrap())]);
        });

        let mut sum = 0;
        world.run(|a: &A| {
            sum += a.0;
        });

        assert_eq!(sum, 6);
    }

    #[test]
    fn spawn_inside_system_subset() {
        let world = World::new();

        world.spawn(&[&A(1), &C("1".into())]);
        world.spawn(&[&A(2), &C("2".into())]);
        world.spawn(&[&A(3), &C("3".into())]);

        world.run(|c: &C| {
            world.spawn(&[&A(str::parse::<u32>(c.0.unwrap()).unwrap() * 10)]);
        });

        let mut sum = 0;
        world.run(|a: &A| {
            sum += a.0;
        });

        assert_eq!(sum, 66);
    }

    #[test]
    #[should_panic]
    fn spawn_inside_system_should_panic() {
        let world = World::new();

        world.spawn(&[&A(1)]);

        world.run(|_: &A| {
            world.spawn(&[&A(1)]);
        });
    }

    #[test]
    fn despawn() {
        let world = World::new();

        world.spawn(&[&A(1)]);
        world.spawn(&[&A(2)]);
        let e = world.spawn(&[&A(3)]);
        world.spawn(&[&A(4)]);

        world.despawn(e);

        let mut sum = 0;
        world.run(|a: &A| {
            sum += a.0;
        });

        assert_eq!(sum, 7);
    }
}
