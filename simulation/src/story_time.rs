use bevy::prelude::*;
use chrono::Datelike;
use chrono::{DateTime, LocalResult, NaiveDate, TimeZone, Timelike, Utc};

pub const DEFAULT_TICKS_PER_SECOND: isize = 10;
pub const MAX_USER_TICKS_PER_SECOND: isize = 1_500;
pub const MAX_SYSTEM_TICKS_PER_SECOND: isize = 50_000;
pub const SECONDS_PER_HOUR: isize = 3_600;
pub const SECONDS_PER_DAY: isize = 86_400;

// NOTE: `bevy_reflect` doesn't support DateTime<Utc> without manually implement Reflect (which is hard)
// So, use a timestamp instead and convert to DateTime<Utc> when needed.
// Also, Time/Instant/Duration aren't serializable.
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct StoryRealWorldTime(pub i64);

impl StoryRealWorldTime {
    pub fn as_datetime(&self) -> DateTime<Utc> {
        match Utc.timestamp_millis_opt(self.0) {
            LocalResult::Single(datetime) => datetime,
            LocalResult::Ambiguous(a, b) => {
                panic!("Ambiguous DateTime<Utc> values: {} and {}", a, b);
            }
            LocalResult::None => {
                panic!("Invalid timestamp");
            }
        }
    }
}

#[derive(Default)]
pub struct TimeInfo {
    days: isize,
    hours: isize,
    minutes: isize,
}

impl TimeInfo {
    pub fn days(&self) -> isize {
        self.days
    }

    pub fn hours(&self) -> isize {
        self.hours
    }

    pub fn minutes(&self) -> isize {
        self.minutes
    }

    pub fn get_decimal_hours(&self) -> f32 {
        self.hours() as f32 + self.minutes() as f32 / 60.0
    }
}

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct StoryTime {
    elapsed_ticks: isize,
    pub is_real_time: bool,
    pub is_real_sun: bool,
    pub latitude: f32,
    pub longitude: f32,
    real_time_offset: isize,
    demo_time_offset: isize,
}

impl Default for StoryTime {
    fn default() -> Self {
        StoryTime {
            elapsed_ticks: 0,
            is_real_time: false,
            is_real_sun: false,
            // Might as well default to San Francisco
            latitude: 37.0,
            longitude: -122.0,
            // Real time wants to know how many seconds into the real world day have passed when the story started.
            real_time_offset: chrono::Local::now().time().num_seconds_from_midnight() as isize,
            // Offset by an assumption that, for Sandbox Mode, the story starts at 8AM the first day not at Midnight.
            demo_time_offset: 8 * SECONDS_PER_HOUR,
        }
    }
}

impl StoryTime {
    pub fn elapsed_ticks(&self) -> isize {
        self.elapsed_ticks
    }

    pub fn as_time_info(&self) -> TimeInfo {
        let start_time_offset = if self.is_real_time {
            self.real_time_offset
        } else {
            self.demo_time_offset
        };

        let seconds_total =
            self.elapsed_ticks as f32 / DEFAULT_TICKS_PER_SECOND as f32 + start_time_offset as f32;
        let days = (seconds_total / SECONDS_PER_DAY as f32).floor() as isize;

        // Calculate hours and minutes
        let hours_total = (seconds_total % SECONDS_PER_DAY as f32) / SECONDS_PER_HOUR as f32;
        let hours = hours_total.floor() as isize;
        let minutes = ((hours_total - hours as f32) * 60.0).floor() as isize;

        TimeInfo {
            days,
            hours,
            minutes,
        }
    }

    pub fn is_within_schedule_window(&self) -> bool {
        let time_info = self.as_time_info();

        let (sunrise, _) = self.get_sunrise_sunset_decimal_hours();
        let (hours, _) = decimal_hours_to_hours_minutes(sunrise);

        (time_info.hours() - hours as isize).abs() < 2
    }

    // TODO: Could use an enum or something
    pub fn is_nighttime(&self) -> bool {
        let (sunrise, sunset) = self.get_sunrise_sunset_decimal_hours();

        let time_info = self.as_time_info();

        // TODO: edgecase where sunset is past 10pm or sunrise is before 2am?
        time_info.hours < (sunrise - 2.0) as isize || time_info.hours >= (sunset + 2.0) as isize
    }

    // Use local because trying to reflect user's sunrise/sunset time not Greenwich's.
    pub fn get_sunrise_sunset_decimal_hours(&self) -> (f32, f32) {
        if !self.is_real_time || !self.is_real_sun {
            return (8.0, 20.0);
        }

        // TODO: Base this off of StoryTime's elapsed_ticks + time offset rather than current day so that sun renders correctly when fast-forwarding.
        let today = chrono::Local::now().date_naive();

        let date = NaiveDate::from_ymd_opt(today.year(), today.month(), today.day()).unwrap();

        let sun_times =
            sun_times::sun_times(date, self.latitude as f64, self.longitude as f64, 0.0).unwrap();

        let sunrise: DateTime<chrono::Local> = DateTime::from(sun_times.0);
        let sunset: DateTime<chrono::Local> = DateTime::from(sun_times.1);

        let sunrise_decimal_hours =
            sunrise.time().hour() as f32 + sunrise.time().minute() as f32 / 60.0;

        let sunset_decimal_hours =
            sunset.time().hour() as f32 + sunset.time().minute() as f32 / 60.0;

        (sunrise_decimal_hours, sunset_decimal_hours)
    }
}

/// Store TicksPerSecond separately from SimulationTime because when we're fast forwarding time we won't update TicksPerSecond.
/// This enables resetting back to a user-defined ticks-per-second (adjusted via UI) rather than the default ticks-per-second.
#[derive(Resource)]
pub struct TicksPerSecond(pub isize);

impl Default for TicksPerSecond {
    fn default() -> Self {
        TicksPerSecond(DEFAULT_TICKS_PER_SECOND)
    }
}

#[derive(Resource, Default)]
pub struct FastForwardPendingTicks {
    initial: isize,
    remaining: isize,
}

impl FastForwardPendingTicks {
    pub fn set(&mut self, ticks: isize) {
        self.remaining = ticks;
        self.initial = ticks;
    }

    pub fn tick(&mut self) {
        self.remaining -= 1;

        if self.remaining == 0 {
            self.initial = 0;
        }
    }

    pub fn remaining(&self) -> isize {
        self.remaining
    }

    pub fn initial(&self) -> isize {
        self.initial
    }
}

#[derive(States, Default, Hash, Clone, Copy, Eq, PartialEq, Debug)]
pub enum StoryPlaybackState {
    #[default]
    Stopped,
    Paused,
    Playing,
    FastForwarding,
}

pub fn register_story_time(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<StoryRealWorldTime>();
    app_type_registry.write().register::<StoryTime>();
}

pub fn initialize_story_time_resources(mut commands: Commands) {
    commands.init_resource::<StoryRealWorldTime>();
    commands.init_resource::<StoryTime>();
    commands.init_resource::<FastForwardPendingTicks>();
    commands.init_resource::<TicksPerSecond>();
    commands.insert_resource(Time::<Fixed>::from_seconds(
        1.0 / DEFAULT_TICKS_PER_SECOND as f64,
    ));
}

pub fn remove_story_time_resources(mut commands: Commands) {
    commands.remove_resource::<StoryRealWorldTime>();
    commands.remove_resource::<StoryTime>();
    commands.remove_resource::<FastForwardPendingTicks>();
    commands.remove_resource::<TicksPerSecond>();
    // Can't remove this resource because it's owned by Bevy.
    // Just assume they'll get reset to default when calling `initialize`. It's not a big deal
    // as long as Time::<Fixed> is reset when the app is restarted.
    // commands.remove_resource::<Time<Fixed>>();
}

/// On startup, determine how much real-world time has passed since the last time the app ran,
/// record this value into SimulationTime, and anticipate further processing.
/// Write to SimulationTime because, in another scenario where the app is paused not closed, SimulationTime
/// will be used by Bevy internally to track how de-synced the FixedUpdate schedule is from real-world time.
pub fn setup_story_time(
    mut story_real_world_time: ResMut<StoryRealWorldTime>,
    mut next_story_playback_state: ResMut<NextState<StoryPlaybackState>>,
    mut story_elapsed_ticks: ResMut<StoryTime>,
    ticks_per_second: Res<TicksPerSecond>,
    mut fast_forward_pending_ticks: ResMut<FastForwardPendingTicks>,
    mut fixed_time: ResMut<Time<Fixed>>,
) {
    // Setup story_real_world_time here, rather than as a Default, so that delta_seconds doesn't grow while idling in main menu
    if story_real_world_time.0 == 0 {
        story_real_world_time.0 = Utc::now().timestamp_millis();

        next_story_playback_state.set(StoryPlaybackState::Playing);
    } else {
        let mut delta_seconds = Utc::now()
            .signed_duration_since(story_real_world_time.as_datetime())
            .num_seconds() as isize;

        let seconds_past_max = delta_seconds - SECONDS_PER_DAY;

        if seconds_past_max > 0 {
            // Increment elapsed ticks by the amount not being simulated to keep game clock synced with real-world clock
            if story_elapsed_ticks.is_real_time {
                let missed_ticks = seconds_past_max * ticks_per_second.0;
                story_elapsed_ticks.elapsed_ticks += missed_ticks;
            }

            // Enforce a max of 24 hours because it's impossible to quickly simulate an arbitrary amount of time missed.
            delta_seconds = SECONDS_PER_DAY;
        }

        fast_forward_seconds(
            delta_seconds,
            &mut fixed_time,
            &mut next_story_playback_state,
            &ticks_per_second,
            &mut fast_forward_pending_ticks,
        );
    }
}
/// Control whether the app runs at the default or fast tick rate.
/// Checks if SimulationTime is showing a time de-sync and adjusts tick rate to compensate.
/// Once compensated tick rate has been processed then reset back to default tick rate.
pub fn set_rate_of_time(
    real_time: Res<Time<Real>>,
    mut fixed_time: ResMut<Time<Fixed>>,
    mut fast_forward_pending_ticks: ResMut<FastForwardPendingTicks>,
    ticks_per_second: Res<TicksPerSecond>,
    story_playback_state: Res<State<StoryPlaybackState>>,
    mut next_story_playback_state: ResMut<NextState<StoryPlaybackState>>,
) {
    if fast_forward_pending_ticks.remaining() == 0 {
        if *story_playback_state == StoryPlaybackState::FastForwarding {
            fixed_time.set_timestep_seconds(1.0 / (ticks_per_second.0 as f64));
            next_story_playback_state.set(StoryPlaybackState::Playing);
        } else {
            let seconds = real_time.delta().as_secs() as isize;

            // TODO: This is a magic number. The goal is to say, "If the simulation was paused (due to tab being inactive), play catch-up"
            // If the simulation wasn't playing then time was lost, but there's no changes that would've occurred, so no need to fast forward.
            if seconds > 1 && *story_playback_state.get() == StoryPlaybackState::Playing {
                fast_forward_seconds(
                    seconds,
                    &mut fixed_time,
                    &mut next_story_playback_state,
                    &ticks_per_second,
                    &mut fast_forward_pending_ticks,
                );
            }
        }
    } else {
        fast_forward_pending_ticks.tick();
    }
}

fn fast_forward_seconds(
    seconds: isize,
    fixed_time: &mut ResMut<Time<Fixed>>,
    next_story_playback_state: &mut ResMut<NextState<StoryPlaybackState>>,
    ticks_per_second: &Res<TicksPerSecond>,
    fast_forward_pending_ticks: &mut ResMut<FastForwardPendingTicks>,
) {
    fixed_time.set_timestep_seconds(1.0 / MAX_SYSTEM_TICKS_PER_SECOND as f64);
    fast_forward_pending_ticks.set(ticks_per_second.0 * seconds);
    next_story_playback_state.set(StoryPlaybackState::FastForwarding);
}

// Track real-world time to be able to derive how much time elapsed while app was closed.
// Keep this updated, rather than capture JIT, because running Bevy systems JIT as app closing isn't viable.
pub fn update_story_real_world_time(mut story_real_world_time: ResMut<StoryRealWorldTime>) {
    story_real_world_time.0 = Utc::now().timestamp_millis();
}

// Track in-game time by counting elapsed ticks.
pub fn update_story_elapsed_ticks(mut story_time: ResMut<StoryTime>) {
    story_time.elapsed_ticks += 1;
}

pub fn update_time_scale(
    mut fixed_time: ResMut<Time<Fixed>>,
    ticks_per_second: Res<TicksPerSecond>,
    story_playback_state: Res<State<StoryPlaybackState>>,
    next_story_playback_state: Res<NextState<StoryPlaybackState>>,
) {
    // Don't unintentionally overwrite simulation_time.period when shifting into FastForwarding.
    if *story_playback_state == StoryPlaybackState::FastForwarding
        || next_story_playback_state.0 == Some(StoryPlaybackState::FastForwarding)
    {
        return;
    }

    fixed_time.set_timestep_seconds(1.0 / (ticks_per_second.0 as f64));
}

fn decimal_hours_to_hours_minutes(decimal_hours: f32) -> (f32, f32) {
    let hours = decimal_hours.trunc();
    let minutes = (decimal_hours.fract() * 60.0).round();
    (hours, minutes)
}
