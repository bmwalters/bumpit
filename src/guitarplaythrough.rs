use crate::chart;

#[derive(Copy, Clone)]
pub enum Fret {
	G = 0, R, Y, B, O
}

pub enum GuitarNoteStrumType {
	Strum,
	Hopo,
	Tap,
}

pub struct GuitarNote {
	pub ticks: u64,
	pub chord: [bool; 5],
	pub strum_type: GuitarNoteStrumType,
	pub duration: u64,
}

impl GuitarNote {
	pub fn is_open(self: &Self) -> bool {
		!(self.chord[0] || self.chord[1] || self.chord[2] || self.chord[3] || self.chord[4])
	}
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
			.fold(Vec::new(), |mut notes, note| {
				let prev_note_in_chord = match notes.last_mut() {
					None => None,
					Some(prev_note) => if note.ticks == prev_note.ticks { Some(prev_note) } else { None },
				};

				let note_to_modify: &mut GuitarNote = match prev_note_in_chord {
					Some(prev_note) => prev_note,
					None => {
						notes.push(GuitarNote {
							ticks: note.ticks,
							chord: [false, false, false, false, false],
							strum_type: GuitarNoteStrumType::Strum,
							duration: note.duration,
						});
						notes.last_mut().unwrap()
					}
				};

				match note.note {
					0 => note_to_modify.chord[Fret::G as usize] = true,
					1 => note_to_modify.chord[Fret::R as usize] = true,
					2 => note_to_modify.chord[Fret::Y as usize] = true,
					3 => note_to_modify.chord[Fret::B as usize] = true,
					4 => note_to_modify.chord[Fret::O as usize] = true,
					5 => note_to_modify.strum_type = GuitarNoteStrumType::Hopo, // TODO: this really means FORCE - calculate hopo or not
					6 => note_to_modify.strum_type = GuitarNoteStrumType::Tap,
					7 => note_to_modify.chord = [false, false, false, false, false],
					_ => (), // TODO: warn or something
				}

				return notes;
			})
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

				let fretted = frets_match(self.frets, note.chord);
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
