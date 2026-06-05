mod invariants;
mod regressions;
mod runner;
mod scenario;

use rand::{RngExt, SeedableRng, rngs::StdRng};
use std::time::{SystemTime, UNIX_EPOCH};

use self::runner::run_randomized_case;

#[derive(Clone, Copy, Debug)]
pub struct RandomTestConfig {
    pub seed: u64,
    pub case_count: usize,
    pub max_actions: usize,
    pub replay_case_index: Option<usize>,
    pub generated_seed: bool,
}

impl RandomTestConfig {
    pub fn from_env() -> Self {
        let seed = std::env::var("TEST_SEED")
            .ok()
            .and_then(|value| value.parse::<u64>().ok());
        let generated_seed = seed.is_none();
        let seed = seed.unwrap_or_else(fallback_seed);
        let case_count = std::env::var("RANDOM_TEST_CASES")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|count| *count > 0)
            .unwrap_or(250);
        let max_actions = std::env::var("RANDOM_TEST_MAX_ACTIONS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|count| *count > 0)
            .unwrap_or(500);
        let replay_case_index = std::env::var("RANDOM_TEST_CASE_INDEX")
            .ok()
            .and_then(|value| value.parse::<usize>().ok());

        Self {
            seed,
            case_count,
            max_actions,
            replay_case_index,
            generated_seed,
        }
    }
}

fn fallback_seed() -> u64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    duration.as_nanos() as u64
}

pub(super) fn derive_case_seed(root_seed: u64, case_index: usize) -> u64 {
    let mut root_rng = StdRng::seed_from_u64(root_seed);

    for _ in 0..case_index {
        let _ = root_rng.random::<u64>();
    }

    root_rng.random::<u64>()
}

#[test]
fn randomized_backend_engine_cases_are_deterministic_and_replayable() {
    let config = RandomTestConfig::from_env();
    let case_indexes = if let Some(case_index) = config.replay_case_index {
        vec![case_index]
    } else {
        (0..config.case_count).collect::<Vec<_>>()
    };

    eprintln!(
        "running randomized jeopardy backend tests with seed={} cases={} max_actions={} replay_case_index={:?}{}",
        config.seed,
        config.case_count,
        config.max_actions,
        config.replay_case_index,
        if config.generated_seed {
            " (generated seed)"
        } else {
            ""
        }
    );

    for case_index in case_indexes {
        eprintln!("{}", case_index);
        let case_seed = derive_case_seed(config.seed, case_index);
        if let Err(report) =
            run_randomized_case(config.seed, case_index, case_seed, config.max_actions)
        {
            panic!("{report}");
        }
    }
}
