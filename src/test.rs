#[cfg(test)]
mod tests {
    use crate::{self as ecs};
    use codegen::Component;

    use crate::{
        Entity, World,
    };

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
        let mut world = World::new();

        let entity_ref = world.spawn(&[&A(42u32), &B(false), &C(Some("a"))]);
        assert_eq!(entity_ref.0, 0);

        let (_, index) = world.entities.get(0).unwrap().clone().unwrap();
        assert_eq!(index, Entity(0));

        assert_eq!(world.get_component::<A>(Entity(0)).unwrap().0, 42);
        assert_eq!(world.get_component::<B>(Entity(0)).unwrap().0, false);
        assert_eq!(world.get_component::<C>(Entity(0)).unwrap().0, Some("a"));

        // repeat in different order
        assert_eq!(world.get_component::<C>(Entity(0)).unwrap().0, Some("a"));
        assert_eq!(world.get_component::<B>(Entity(0)).unwrap().0, false);
        assert_eq!(world.get_component::<A>(Entity(0)).unwrap().0, 42);

        world.get_component_mut::<A>(Entity(0)).unwrap().0 = 123u32;
        world.get_component_mut::<B>(Entity(0)).unwrap().0 = true;

        assert_eq!(world.get_component::<A>(Entity(0)).unwrap().0, 123u32);
        assert_eq!(world.get_component::<B>(Entity(0)).unwrap().0, true);
    }

    #[test]
    fn query() {
        let mut world = World::new();

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
        let mut world = World::new();

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
}
