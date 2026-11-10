mod test {
    use crate::scenes::common::FadeController;

    #[test]
    fn opacity_starts_at_zero() {
        let fade_controller = FadeController::new_with_fade_duration(None, 255);

        assert_eq!(fade_controller.opacity(), 0);
    }

    #[test]
    fn opacity_ramps_up_during_fadein() {
        let mut fade_controller = FadeController::new_with_fade_duration(None, 255);

        for fixture in [0, 10, 100, 255] {
            fade_controller.tick(fixture);

            assert_eq!(fade_controller.opacity(), fixture as u8);
        }
    }

    #[test]
    fn opacity_stays_at_max_after_fadein() {
        let mut fade_controller = FadeController::new_with_fade_duration(None, 255);

        fade_controller.tick(500);

        assert_eq!(fade_controller.opacity(), 255);
    }

    #[test]
    fn fadeout_has_to_be_started_explicitly() {
        let mut fade_controller = FadeController::new_with_fade_duration(None, 255);

        for fixture in [0, 10, 100, 255, 500] {
            fade_controller.tick(fixture);

            assert!(!fade_controller.fadeout_started());
            assert!(!fade_controller.fadeout_done());
        }
    }

    #[test]
    fn opacity_ramps_down_during_fadeout() {
        let mut fade_controller = FadeController::new_with_fade_duration(None, 255);

        fade_controller.tick(500);
        fade_controller.start_fadeout();

        assert_eq!(fade_controller.opacity(), 255);

        for fixture in [510, 600, 755] {
            fade_controller.tick(fixture);

            assert_eq!(fade_controller.opacity(), (755 - fixture) as u8);
        }
    }

    #[test]
    fn opacity_stays_at_zero_after_fadeout() {
        let mut fade_controller = FadeController::new_with_fade_duration(None, 255);

        fade_controller.tick(500);
        fade_controller.start_fadeout();
        fade_controller.tick(800);

        assert_eq!(fade_controller.opacity(), 0);
    }

    #[test]
    fn fadein_started_trips_when_fadein_is_started() {
        let mut fade_controller = FadeController::new_with_fade_duration(None, 255);

        fade_controller.tick(500);
        fade_controller.start_fadeout();

        assert!(fade_controller.fadeout_started());
    }

    #[test]
    fn fadeout_done_trips_when_fadout_is_done() {
        let mut fade_controller = FadeController::new_with_fade_duration(None, 255);

        fade_controller.tick(500);
        fade_controller.start_fadeout();

        for fixture in [510, 600, 755] {
            fade_controller.tick(fixture);

            assert!(!fade_controller.fadeout_done());
        }

        for fixture in [756, 800] {
            fade_controller.tick(fixture);

            assert!(fade_controller.fadeout_done());
        }
    }

    #[test]
    fn starting_fadeout_during_fadein_starts_from_current_opacity() {
        let mut fade_controller = FadeController::new_with_fade_duration(None, 255);

        fade_controller.tick(100);
        fade_controller.start_fadeout();

        assert!(fade_controller.fadeout_started());
        assert!(!fade_controller.fadeout_done());

        for fixture in [100, 150, 180, 200] {
            fade_controller.tick(fixture);

            assert_eq!(fade_controller.opacity(), (200 - fixture) as u8);
            assert!(!fade_controller.fadeout_done());
        }

        fade_controller.tick(201);
        assert!(fade_controller.fadeout_done());
    }

    #[test]
    fn finishing_and_recreating_resumes_with_clock_offset() {
        let mut fade_controller = FadeController::new_with_fade_duration(None, 255);

        fade_controller.tick(100);
        assert_eq!(fade_controller.opacity(), 100);

        let mut fade_controller =
            FadeController::new_with_fade_duration(Some(fade_controller.finish()), 255);
        assert_eq!(fade_controller.opacity(), 100);

        fade_controller.tick(100);

        assert_eq!(fade_controller.opacity(), 200);

        let mut fade_controller =
            FadeController::new_with_fade_duration(Some(fade_controller.finish()), 255);
        assert_eq!(fade_controller.opacity(), 200);

        fade_controller.tick(100);

        assert_eq!(fade_controller.opacity(), 255);

        let mut fade_controller =
            FadeController::new_with_fade_duration(Some(fade_controller.finish()), 255);
        assert_eq!(fade_controller.opacity(), 255);

        assert!(!fade_controller.fadeout_started());
        assert!(!fade_controller.fadeout_done());

        fade_controller.start_fadeout();
        fade_controller.tick(100);

        assert_eq!(fade_controller.opacity(), 155);
        assert!(fade_controller.fadeout_started());

        let mut fade_controller =
            FadeController::new_with_fade_duration(Some(fade_controller.finish()), 255);
        assert_eq!(fade_controller.opacity(), 155);
        assert!(fade_controller.fadeout_started());

        fade_controller.tick(100);

        assert_eq!(fade_controller.opacity(), 55);

        let mut fade_controller =
            FadeController::new_with_fade_duration(Some(fade_controller.finish()), 255);
        assert_eq!(fade_controller.opacity(), 55);

        fade_controller.tick(100);

        assert_eq!(fade_controller.opacity(), 0);
        assert!(fade_controller.fadeout_done());

        let fade_controller =
            FadeController::new_with_fade_duration(Some(fade_controller.finish()), 255);
        assert_eq!(fade_controller.opacity(), 0);
        assert!(fade_controller.fadeout_done());
    }
}
