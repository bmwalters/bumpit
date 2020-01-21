use crate::chart;

#[derive(Copy, Clone)]
pub enum Fret {
	G = 0, R, Y, B, O
}

pub struct GuitarNote {
	pub ticks: u64,
	pub chord: [bool; 5],
	pub duration: u64,
}

// TODO: refactor
pub struct GuitarChart {
	pub ticks_per_beat: u64, // aka Resolution
	pub beats_per_minute: u64,
	/* Vector of notes sorted by their tick */
	pub notes: std::vec::Vec<GuitarNote>,
}

impl GuitarChart {
	pub fn ticks_to_ms(self: &Self, ticks: u64) -> f32 {
		((ticks as f32) / (self.ticks_per_beat as f32)) / (self.beats_per_minute as f32) * 60f32 * 1000f32
	}
}

pub struct GuitarPlaythrough {
	pub chart: GuitarChart,
	pub score: u64,
	pub notes_hit: u64,
	pub streak: u64,
	pub sp_phrases: u64,
	pub avg_multiplier: u64,
	// TODO: section scores

	// state
	pub frets: [bool; 5],
	time: f32,
	next_note_index: usize, // TODO: are you only contesting the very next note?
}

impl GuitarPlaythrough {
	pub fn new(chart: chart::Chart) -> Result<GuitarPlaythrough, &'static str> {
		let guitar_chart = GuitarChart {
			ticks_per_beat: chart.sync_track.iter().filter_map(|st| {
				match st {
					chart::SyncTrack::BeatsPerMinute { bpm1000, .. } => Some(bpm1000 / 1000),
					chart::SyncTrack::TimeSignature { .. } => None,
				}
			}).nth(0).ok_or_else(|| "no BPM found")?, // TODO: handle
			beats_per_minute: chart.song.resolution,
			notes: chart.parts
			.iter()
			.filter(|part| {
				match (&part.instrument, &part.difficulty) {
					(chart::Instrument::Guitar, chart::Difficulty::Expert) => true,
					_ => false
				}
			})
			.nth(0)
			.ok_or_else(|| "no Expert Guitar part found")? // TODO: handle
			.notes
			.iter()
			.map(|note| Ok(GuitarNote {
				ticks: note.ticks,
				lane: match note.note {
					0 => Some(Fret::G),
					1 => Some(Fret::R),
					2 => Some(Fret::Y),
					3 => Some(Fret::B),
					4 => Some(Fret::O),
					_ => None,
				}.unwrap_or(Fret::G), // TODO: handle
				duration: note.duration,
			}))
			.collect::<Result<Vec<GuitarNote>, &'static str>>()?,
		};

		return Ok(GuitarPlaythrough {
			chart: guitar_chart,
			score: 0,
			notes_hit: 0,
			streak: 0,
			sp_phrases: 0,
			avg_multiplier: 0,
			frets: [false, false, false, false, false],
			time: 0.0, // TODO: negative start time?
			next_note_index: 0,
		})
	}
}

pub enum GuitarInputAction {
	FretDown(Fret),
	FretUp(Fret),
	Strum,
}

// when a note is hit (strum and earlier fret
//                     or fret with hopo and streak > 0
//                     or fret with tap):
//    update score, streak
//    unmute track if muted
// when a note exits the hit window:
//    mute track
//    if you have a streak, shake the screen, play miss sound
//    update streak
//    if enabled, play a miss sound
// when you release a sustain:
//    mute track
pub enum GuitarGameEffect {
	Hit,
	Overstrum,
	MissStreak,
	MissNoStreak,
	ReleaseSustain,
}

const HALF_HIT_WINDOW_MS: f32 = 40.0;

fn frets_match(frets: [bool; 5], chord: [bool; 5]) -> bool {
	return frets[0] == chord[0]
		&& frets[1] == chord[1]
		&& frets[2] == chord[2]
		&& frets[3] == chord[3]
		&& frets[4] == chord[4];
}

impl GuitarPlaythrough {
	pub fn apply(self: &mut Self, action: &GuitarInputAction, time_ms: f32) -> Option<GuitarGameEffect> {
		match action {
			GuitarInputAction::FretDown(fret) => {
				// TODO: taps/hopos
				self.frets[*fret as usize] = true;
				None
			},
			GuitarInputAction::FretUp(fret) => {
				self.frets[*fret as usize] = false;
				None
			},
			GuitarInputAction::Strum => {
				if self.next_note_index >= self.chart.notes.len() {
					return Some(GuitarGameEffect::Overstrum);
				}

				let note = &self.chart.notes[self.next_note_index];

				// TODO: pull real chord from GuitarNote
				let chord: [bool; 5] = [(note.lane as usize) == 0,
										(note.lane as usize) == 1,
										(note.lane as usize) == 2,
										(note.lane as usize) == 3,
										(note.lane as usize) == 4];
				let fretted = frets_match(self.frets, chord);
				let on_time = f32::abs(time_ms - self.chart.ticks_to_ms(note.ticks)) <= HALF_HIT_WINDOW_MS;

				if fretted && on_time {
					self.notes_hit += 1;
					self.streak += 1;
					self.next_note_index += 1;
					None
				} else {
					self.streak = 0;
					Some(GuitarGameEffect::Overstrum)
				}
			}
		}
	}

	// TODO: must handle pause -> set time back 5 seconds
	pub fn update_time(self: &mut Self, time_ms: f32) -> Option<GuitarGameEffect> {
		self.time = time_ms;

		if self.next_note_index >= self.chart.notes.len() {
			return None;
		}

		let mut missed = false;

		loop {
			let note = &self.chart.notes[self.next_note_index];

			if self.chart.ticks_to_ms(note.ticks) < (time_ms - HALF_HIT_WINDOW_MS) {
				missed = true;
				self.next_note_index += 1;
			} else {
				break;
			}
		}

		if missed {
			let effect = if self.streak > 0 { GuitarGameEffect::MissStreak } else { GuitarGameEffect::MissNoStreak };

			self.streak = 0;

			Some(effect)
		} else {
			None
		}
	}
}
