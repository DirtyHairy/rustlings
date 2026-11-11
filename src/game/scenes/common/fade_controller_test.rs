mod test {
    use crate::scenes::common::FadeController;

    #[test]
    fn opacity_starts_at_zero() {
        let fade_controller = FadeController::new(None, 255, 510);

        assert_eq!(fade_controller.opacity(), 0);
    }

    #[test]
    fn opacity_ramps_up_during_fadein() {
        let mut fade_controller = FadeController::new(None, 255, 510);

        for fixture in [0, 10, 100, 255] {
            fade_controller.tick(fixture);

            assert_eq!(fade_controller.opacity(), fixture as u8);
        }
    }

    #[test]
    fn opacity_stays_at_max_after_fadein() {
        let mut fade_controller = FadeController::new(None, 255, 510);

        fade_controller.tick(500);

        assert_eq!(fade_controller.opacity(), 255);
    }

    #[test]
    fn fadeout_has_to_be_started_explicitly() {
        let mut fade_controller = FadeController::new(None, 255, 510);

        for fixture in [0, 10, 100, 255, 500] {
            fade_controller.tick(fixture);

            assert!(!fade_controller.fadeout_started());
            assert!(!fade_controller.fadeout_done());
        }
    }

    #[test]
    fn opacity_ramps_down_during_fadeout() {
        let mut fade_controller = FadeController::new(None, 255, 510);

        fade_controller.tick(500);
        fade_controller.start_fadeout();

        assert_eq!(fade_controller.opacity(), 255);

        for fixture in [520, 600, 1010] {
            fade_controller.tick(fixture);

            assert_eq!(fade_controller.opacity(), (255 - (fixture - 500) / 2) as u8);
        }
    }

    #[test]
    fn opacity_stays_at_zero_after_fadeout() {
        let mut fade_controller = FadeController::new(None, 255, 510);

        fade_controller.tick(500);
        fade_controller.start_fadeout();
        fade_controller.tick(1500);

        assert_eq!(fade_controller.opacity(), 0);
    }

    #[test]
    fn fadein_started_trips_when_fadein_is_started() {
        let mut fade_controller = FadeController::new(None, 255, 510);

        fade_controller.tick(500);
        fade_controller.start_fadeout();

        assert!(fade_controller.fadeout_started());
    }

    #[test]
    fn fadeout_done_trips_when_fadout_is_done() {
        let mut fade_controller = FadeController::new(None, 255, 510);

        fade_controller.tick(500);
        fade_controller.start_fadeout();

        for fixture in [510, 600, 1009] {
            fade_controller.tick(fixture);

            assert!(!fade_controller.fadeout_done());
        }

        for fixture in [1010, 1500] {
            fade_controller.tick(fixture);

            assert!(fade_controller.fadeout_done());
        }
    }

    #[test]
    fn starting_fadeout_during_fadein_starts_from_current_opacity() {
        let mut fade_controller = FadeController::new(None, 255, 510);

        fade_controller.tick(100);
        fade_controller.start_fadeout();

        assert!(fade_controller.fadeout_started());
        assert!(!fade_controller.fadeout_done());

        for fixture in [100, 200, 280, 209] {
            fade_controller.tick(fixture);

            assert_eq!(
                fade_controller.opacity(),
                (100 - ((fixture - 100) * 255) / 510) as u8
            );
            assert!(!fade_controller.fadeout_done());
        }

        fade_controller.tick(300);

        assert_eq!(fade_controller.opacity(), 0);
        assert!(fade_controller.fadeout_done());
    }

    #[test]
    fn next_tick_at_msec_ramps_up_during_fadein() {
        let mut fade_controller = FadeController::new(None, 765, 1020);
        assert_eq!(fade_controller.next_tick_at_msec(), Some(3));

        fade_controller.tick(32);
        assert_eq!(fade_controller.next_tick_at_msec(), Some(33));

        fade_controller.tick(33);
        assert_eq!(fade_controller.next_tick_at_msec(), Some(36));

        fade_controller.tick(764);
        assert_eq!(fade_controller.next_tick_at_msec(), Some(765));
    }

    #[test]
    fn next_tick_at_msec_returns_none_after_fadein() {
        let mut fade_controller = FadeController::new(None, 765, 1020);

        fade_controller.tick(765);
        assert_eq!(fade_controller.next_tick_at_msec(), None);

        fade_controller.tick(800);
        assert_eq!(fade_controller.next_tick_at_msec(), None);
    }

    #[test]
    fn next_tick_at_msec_ramps_up_after_fadeout() {
        let mut fade_controller = FadeController::new(None, 765, 1020);
        fade_controller.tick(1000);

        fade_controller.start_fadeout();
        assert_eq!(fade_controller.next_tick_at_msec(), Some(1004));

        fade_controller.tick(1199);
        assert_eq!(fade_controller.next_tick_at_msec(), Some(1200));

        fade_controller.tick(1200);
        assert_eq!(fade_controller.next_tick_at_msec(), Some(1204));

        fade_controller.tick(2019);
        assert_eq!(fade_controller.next_tick_at_msec(), Some(2020));
    }

    #[test]
    fn next_tick_at_msec_returns_none_after_fadeout() {
        let mut fade_controller = FadeController::new(None, 765, 1020);
        fade_controller.tick(1000);

        fade_controller.start_fadeout();

        fade_controller.tick(2020);
        assert_eq!(fade_controller.next_tick_at_msec(), None);

        fade_controller.tick(4000);
        assert_eq!(fade_controller.next_tick_at_msec(), None);
    }

    #[test]
    fn next_tick_at_msec_correctly_switches_pace_if_fadeout_starts_during_fadein() {
        let mut fade_controller = FadeController::new(None, 765, 1020);
        fade_controller.tick(301);

        fade_controller.start_fadeout();
        assert_eq!(fade_controller.next_tick_at_msec(), Some(305));

        fade_controller.tick(316);
        assert_eq!(fade_controller.next_tick_at_msec(), Some(317));

        fade_controller.tick(700);
        assert_eq!(fade_controller.next_tick_at_msec(), Some(701));

        fade_controller.tick(701);
        assert_eq!(fade_controller.next_tick_at_msec(), None);

        fade_controller.tick(1000);
        assert_eq!(fade_controller.next_tick_at_msec(), None);
    }

    #[test]
    fn finishing_and_recreating_resumes_with_clock_offset() {
        let mut fade_controller = FadeController::new(None, 255, 510);

        fade_controller.tick(100);
        assert_eq!(fade_controller.opacity(), 100);

        let mut fade_controller = FadeController::new(Some(fade_controller.finish()), 255, 510);
        assert_eq!(fade_controller.opacity(), 100);

        fade_controller.tick(100);

        assert_eq!(fade_controller.opacity(), 200);

        let mut fade_controller = FadeController::new(Some(fade_controller.finish()), 255, 510);
        assert_eq!(fade_controller.opacity(), 200);

        fade_controller.tick(100);

        assert_eq!(fade_controller.opacity(), 255);

        let mut fade_controller = FadeController::new(Some(fade_controller.finish()), 255, 510);
        assert_eq!(fade_controller.opacity(), 255);

        assert!(!fade_controller.fadeout_started());
        assert!(!fade_controller.fadeout_done());

        fade_controller.start_fadeout();
        fade_controller.tick(200);

        assert_eq!(fade_controller.opacity(), 155);
        assert!(fade_controller.fadeout_started());

        let mut fade_controller = FadeController::new(Some(fade_controller.finish()), 255, 510);
        assert_eq!(fade_controller.opacity(), 155);
        assert!(fade_controller.fadeout_started());

        fade_controller.tick(200);

        assert_eq!(fade_controller.opacity(), 55);

        let mut fade_controller = FadeController::new(Some(fade_controller.finish()), 255, 510);
        assert_eq!(fade_controller.opacity(), 55);

        fade_controller.tick(200);

        assert_eq!(fade_controller.opacity(), 0);
        assert!(fade_controller.fadeout_done());

        let fade_controller = FadeController::new(Some(fade_controller.finish()), 255, 510);
        assert_eq!(fade_controller.opacity(), 0);
        assert!(fade_controller.fadeout_done());
    }
}
