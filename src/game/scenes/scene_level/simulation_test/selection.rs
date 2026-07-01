use rustlings::game_data::{SKILLS, Skill};

use crate::{
    scenes::scene_level::simulation::{BOMBER_COUNTDOWN_TICKS, SelectionResult},
    state::{Activity, Direction, LemmingAnimation, LemmingHealth, LemmingState},
};

const TERRAIN_SKILLS: [Skill; 4] = [Skill::Basher, Skill::Miner, Skill::Digger, Skill::Builder];

fn is_terrain_skill(skill: Skill) -> bool {
    TERRAIN_SKILLS.contains(&skill)
}

fn expected_rejection(skill: Skill) -> SelectionResult {
    if is_terrain_skill(skill) {
        SelectionResult::Fallback
    } else {
        SelectionResult::Abort
    }
}

fn fixture_in(activity: Activity) -> LemmingState {
    LemmingState::fixture(0, 0, Direction::Right, activity)
}

fn assign_on(activity: Activity, skill: Skill) -> SelectionResult {
    fixture_in(activity).assign_skill(skill)
}

fn assign_with_health(activity: Activity, health: LemmingHealth, skill: Skill) -> SelectionResult {
    let mut lemming = fixture_in(activity);
    lemming.health = health;
    lemming.assign_skill(skill)
}

#[test]
fn tier2_fail_terrain_skills_return_fallback() {
    for skill in TERRAIN_SKILLS {
        assert_eq!(
            assign_on(Activity::Splatting, skill),
            SelectionResult::Fallback,
            "skill={}",
            skill
        );
    }
}

#[test]
fn tier2_fail_non_terrain_skills_return_abort() {
    for skill in [
        Skill::Climber,
        Skill::Floater,
        Skill::Bomber,
        Skill::Blocker,
    ] {
        assert_eq!(
            assign_on(Activity::Splatting, skill),
            SelectionResult::Abort,
            "skill={}",
            skill
        );
    }
}

#[test]
fn walking_accepts_all_skills() {
    for skill in SKILLS {
        assert_eq!(
            assign_on(Activity::Walking, skill),
            SelectionResult::Success,
            "skill={}",
            skill
        );
    }
}

#[test]
fn splatting_rejects_all_skills() {
    for skill in SKILLS {
        assert_eq!(
            assign_on(Activity::Splatting, skill),
            expected_rejection(skill),
            "skill={}",
            skill
        );
    }
}

#[test]
fn blocking_accepts_only_bomber() {
    for skill in SKILLS {
        let expected = if skill == Skill::Bomber {
            SelectionResult::Success
        } else {
            expected_rejection(skill)
        };
        assert_eq!(
            assign_on(Activity::Blocking, skill),
            expected,
            "skill={}",
            skill
        );
    }
}

#[test]
fn climbing_accepts_only_floater_and_bomber() {
    for skill in SKILLS {
        let expected = if matches!(skill, Skill::Floater | Skill::Bomber) {
            SelectionResult::Success
        } else {
            expected_rejection(skill)
        };
        assert_eq!(
            assign_on(Activity::Climbing, skill),
            expected,
            "skill={}",
            skill
        );
    }
}

#[test]
fn floating_accepts_only_climber_and_bomber() {
    for skill in SKILLS {
        let expected = if matches!(skill, Skill::Climber | Skill::Bomber) {
            SelectionResult::Success
        } else {
            expected_rejection(skill)
        };
        assert_eq!(
            assign_on(Activity::Floating(Default::default()), skill),
            expected,
            "skill={}",
            skill
        );
    }
}

#[test]
fn drowning_accepts_climber_floater_bomber() {
    for skill in SKILLS {
        let expected = if matches!(skill, Skill::Climber | Skill::Floater | Skill::Bomber) {
            SelectionResult::Success
        } else {
            expected_rejection(skill)
        };
        assert_eq!(
            assign_on(Activity::Drowning, skill),
            expected,
            "skill={}",
            skill
        );
    }
}

#[test]
fn exiting_accepts_climber_floater_bomber() {
    for skill in SKILLS {
        let expected = if matches!(skill, Skill::Climber | Skill::Floater | Skill::Bomber) {
            SelectionResult::Success
        } else {
            expected_rejection(skill)
        };
        assert_eq!(
            assign_on(Activity::Exiting, skill),
            expected,
            "skill={}",
            skill
        );
    }
}

#[test]
fn frying_accepts_climber_and_floater_not_bomber() {
    for skill in SKILLS {
        let expected = if matches!(skill, Skill::Climber | Skill::Floater) {
            SelectionResult::Success
        } else {
            expected_rejection(skill)
        };
        assert_eq!(
            assign_on(Activity::Frying, skill),
            expected,
            "skill={}",
            skill
        );
    }
}

#[test]
fn falling_accepts_climber_floater_bomber() {
    for skill in SKILLS {
        let expected = if matches!(skill, Skill::Climber | Skill::Floater | Skill::Bomber) {
            SelectionResult::Success
        } else {
            expected_rejection(skill)
        };
        assert_eq!(
            assign_on(Activity::Falling(Default::default()), skill),
            expected,
            "skill={}",
            skill
        );
    }
}

#[test]
fn jumping_accepts_climber_floater_bomber() {
    for skill in SKILLS {
        let expected = if matches!(skill, Skill::Climber | Skill::Floater | Skill::Bomber) {
            SelectionResult::Success
        } else {
            expected_rejection(skill)
        };
        assert_eq!(
            assign_on(Activity::Jumping, skill),
            expected,
            "skill={}",
            skill
        );
    }
}

#[test]
fn terrain_activity_rejects_own_skill() {
    let pairs = [
        (Activity::Bashing, Skill::Basher),
        (Activity::Building, Skill::Builder),
        (Activity::Mining, Skill::Miner),
        (Activity::Digging(Default::default()), Skill::Digger),
    ];

    for (activity, own_skill) in pairs {
        for skill in SKILLS {
            let expected = if skill == own_skill {
                SelectionResult::Fallback
            } else {
                SelectionResult::Success
            };
            assert_eq!(
                assign_on(activity.clone(), skill),
                expected,
                "activity={:?}, skill={}",
                activity,
                skill
            );
        }
    }
}

#[test]
fn healthy_walker_transitions() {
    for skill in SKILLS {
        assert_eq!(
            assign_with_health(Activity::Walking, LemmingHealth::Healthy, skill),
            SelectionResult::Success,
            "skill={}",
            skill
        );
    }
}

#[test]
fn ohno_accepts_climber_and_floater_only() {
    for skill in SKILLS {
        let expected = match skill {
            Skill::Climber | Skill::Floater => SelectionResult::Success,
            _ => expected_rejection(skill),
        };
        assert_eq!(
            assign_with_health(Activity::Walking, LemmingHealth::OhNo, skill),
            expected,
            "skill={}",
            skill
        );
    }
}

#[test]
fn exploding_rejects_all_skills() {
    for skill in SKILLS {
        assert_eq!(
            assign_with_health(Activity::Walking, LemmingHealth::Exploding, skill),
            expected_rejection(skill),
            "skill={}",
            skill
        );
    }
}

#[test]
fn climber_flag_rejects_climber_skill() {
    let mut lemming = fixture_in(Activity::Walking);
    lemming.climber = true;

    assert_eq!(lemming.assign_skill(Skill::Climber), SelectionResult::Abort);
}

#[test]
fn floater_flag_rejects_floater_skill() {
    let mut lemming = fixture_in(Activity::Walking);
    lemming.floater = true;

    assert_eq!(lemming.assign_skill(Skill::Floater), SelectionResult::Abort);
}

#[test]
fn countdown_rejects_bomber_skill() {
    let mut lemming = fixture_in(Activity::Walking);
    lemming.countdown = Some(42);

    assert_eq!(lemming.assign_skill(Skill::Bomber), SelectionResult::Abort);
}

#[test]
fn athlete_walking_rejects_climber_and_floater_but_admits_terrain() {
    for skill in SKILLS {
        let mut lemming = fixture_in(Activity::Walking);
        lemming.climber = true;
        lemming.floater = true;

        let expected = match skill {
            Skill::Climber | Skill::Floater => SelectionResult::Abort,
            _ => SelectionResult::Success,
        };
        assert_eq!(lemming.assign_skill(skill), expected, "skill={}", skill);
    }
}

#[test]
fn climbing_with_floater_false_accepts_floater() {
    let mut lemming = fixture_in(Activity::Climbing);

    assert_eq!(
        lemming.assign_skill(Skill::Floater),
        SelectionResult::Success
    );
}

#[test]
fn floating_with_climber_false_accepts_climber() {
    let mut lemming = fixture_in(Activity::Floating(Default::default()));

    assert_eq!(
        lemming.assign_skill(Skill::Climber),
        SelectionResult::Success
    );
}

#[test]
fn assign_climber_sets_climber_flag_only() {
    let fixture = fixture_in(Activity::Walking);
    let mut lemming = fixture.clone();

    let result = lemming.assign_skill(Skill::Climber);

    assert_eq!(result, SelectionResult::Success);
    assert_eq!(
        lemming,
        LemmingState {
            climber: true,
            ..fixture
        }
    );
}

#[test]
fn assign_floater_sets_floater_flag_only() {
    let fixture = fixture_in(Activity::Walking);
    let mut lemming = fixture.clone();

    let result = lemming.assign_skill(Skill::Floater);

    assert_eq!(result, SelectionResult::Success);
    assert_eq!(
        lemming,
        LemmingState {
            floater: true,
            ..fixture
        }
    );
}

#[test]
fn assign_bomber_sets_countdown_only() {
    let fixture = fixture_in(Activity::Walking);
    let mut lemming = fixture.clone();

    let result = lemming.assign_skill(Skill::Bomber);

    assert_eq!(result, SelectionResult::Success);
    assert_eq!(
        lemming,
        LemmingState {
            countdown: Some(BOMBER_COUNTDOWN_TICKS),
            ..fixture
        }
    );
}

#[test]
fn bomber_countdown_constant_is_79() {
    assert_eq!(BOMBER_COUNTDOWN_TICKS, 79);
}

#[test]
fn assign_blocker_transitions_to_blocking() {
    let fixture = fixture_in(Activity::Walking);
    let mut lemming = fixture.clone();

    let result = lemming.assign_skill(Skill::Blocker);

    assert_eq!(result, SelectionResult::Success);
    assert_eq!(
        lemming,
        LemmingState {
            activity: Activity::Blocking,
            animation: LemmingAnimation::Blocking,
            frame: 0,
            ..fixture
        }
    );
}

#[test]
fn assign_builder_transitions_to_building() {
    let fixture = fixture_in(Activity::Walking);
    let mut lemming = fixture.clone();

    let result = lemming.assign_skill(Skill::Builder);

    assert_eq!(result, SelectionResult::Success);
    assert_eq!(
        lemming,
        LemmingState {
            activity: Activity::Building,
            animation: LemmingAnimation::Building,
            frame: 0,
            ..fixture
        }
    );
}

#[test]
fn assign_basher_transitions_to_bashing() {
    let fixture = fixture_in(Activity::Walking);
    let mut lemming = fixture.clone();

    let result = lemming.assign_skill(Skill::Basher);

    assert_eq!(result, SelectionResult::Success);
    assert_eq!(
        lemming,
        LemmingState {
            activity: Activity::Bashing,
            animation: LemmingAnimation::Bashing,
            frame: 0,
            ..fixture
        }
    );
}

#[test]
fn assign_miner_transitions_to_mining() {
    let fixture = fixture_in(Activity::Walking);
    let mut lemming = fixture.clone();

    let result = lemming.assign_skill(Skill::Miner);

    assert_eq!(result, SelectionResult::Success);
    assert_eq!(
        lemming,
        LemmingState {
            activity: Activity::Mining,
            animation: LemmingAnimation::Mining,
            frame: 0,
            ..fixture
        }
    );
}

#[test]
fn assign_digger_transitions_to_digging() {
    let fixture = fixture_in(Activity::Walking);
    let mut lemming = fixture.clone();

    let result = lemming.assign_skill(Skill::Digger);

    assert_eq!(result, SelectionResult::Success);
    assert_eq!(
        lemming,
        LemmingState {
            activity: Activity::Digging(Default::default()),
            animation: LemmingAnimation::Digging,
            frame: 15,
            ..fixture
        }
    );
}
