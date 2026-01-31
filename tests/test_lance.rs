use gpig::lane::LaneManager;

#[cfg(test)]
mod test_lance {
    use git2::Oid;

    use super::*;

    #[test]
    fn straight_line() {
        let mut lane_manager = LaneManager::new();
        lane_manager.assign_commit(
            &Oid::from_str("0001").unwrap(),
            &vec![Oid::from_str("0002").unwrap()],
        );
        lane_manager.assign_commit(
            &Oid::from_str("0002").unwrap(),
            &vec![Oid::from_str("0003").unwrap()],
        );
        assert_eq!(lane_manager.get_lanes().iter().len(), 1);
    }

    #[test]
    fn merge_line() {
        let mut lane_manager = LaneManager::new();
        //         1 _   _
        //         \   \  \
        //         2   \  \
        //         \   3  \
        //         \   \   4
        //         5  _
        //
        lane_manager.assign_commit(
            &Oid::from_str("0001").unwrap(),
            &vec![
                Oid::from_str("0002").unwrap(),
                Oid::from_str("0003").unwrap(),
                Oid::from_str("0004").unwrap(),
            ],
        );

        assert_eq!(lane_manager.get_lanes().iter().len(), 3);

        lane_manager.assign_commit(
            &Oid::from_str("0002").unwrap(),
            &vec![Oid::from_str("0005").unwrap()],
        );

        assert_eq!(lane_manager.get_lanes().iter().len(), 3);
        lane_manager.assign_commit(
            &Oid::from_str("0003").unwrap(),
            &vec![Oid::from_str("0005").unwrap()],
        );

        println!("{:?}", lane_manager.get_lanes());
        assert_eq!(lane_manager.get_lanes().iter().len(), 3);

        lane_manager.assign_commit(&Oid::from_str("0004").unwrap(), &vec![]);
        assert_eq!(lane_manager.get_lanes().iter().len(), 1);
        lane_manager.assign_commit(&Oid::from_str("0005").unwrap(), &vec![]);
        assert_eq!(lane_manager.get_lanes().iter().len(), 0);
    }
}
