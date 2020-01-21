extern crate regex;
use std::collections::HashMap;
use regex::Regex;

pub struct SongStreams {
    pub music: Option<String>,
    pub guitar: Option<String>,
    pub bass: Option<String>,
    pub rhythm: Option<String>,
    pub drum: Option<String>,
}

pub enum SongPlayer2 {
    Bass,
    Rhythm,
}

pub struct Song {
    pub name: Option<String>,
    pub artist: Option<String>,
    pub charter: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
    pub offset: Option<u64>,
    pub resolution: u64,
    pub player2: Option<SongPlayer2>,
    pub difficulty: Option<u64>,
    pub preview_start: Option<f32>,
    pub preview_end: Option<f32>,
    pub genre: Option<String>,
    pub media_type: Option<String>,
    pub streams: SongStreams,
}

pub enum SyncTrack {
    TimeSignature { ticks: u64, upper: u64, lower: u64 },
    BeatsPerMinute { ticks: u64, bpm1000: u64 },
}

pub enum Event {
    Section { ticks: u64, name: String },
    // TODO: support additional event types
    // Rationale for not passing strings:
    // The consumer of this chart library is the game.
    // Moonscraper allows typing strings because it is not the consumer.
    // Entirely strings -> game has to match on strings
    // Some enums + General string -> what's the point of the enums?
    // and anyone who matches on strings has to update when we support.
    // No strings -> We parse exactly what we can deal with.
    // Would be easier if .chart wasn't a moving target
    // i.e. specific version with set list of features
}

pub enum SpecialEvent {
    // type 2: boost / star power / overdrive
    StarPower { ticks: u64, duration: u64 },
}

pub struct Note {
    pub ticks: u64,
    pub note: u64,
    pub duration: u64,
}

pub enum Instrument {
    Guitar,
    GuitarCoop,
    Bass,
    Rhythm,
    GHLGuitar,
    GHLBass,
    Drums,
    Keyboard,
    RealBass,
    RealGuitar,
    RealKeys,
    // Vocals,
    // Harmony1,
    // Harmony2,
    // Harmony3,
}

pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

pub struct Part {
    pub instrument: Instrument,
    pub difficulty: Difficulty,
    pub notes: Vec<Note>,
    pub special_events: Vec<SpecialEvent>,
}

pub struct Chart {
    pub song: Song,
    pub sync_track: Vec<SyncTrack>,
    pub events: Vec<Event>,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone)]
pub enum SongError {
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
    MissingResolution,
}

impl std::convert::From<std::num::ParseIntError> for SongError {
    fn from(err: std::num::ParseIntError) -> SongError {
        SongError::ParseIntError(err)
    }
}

impl std::convert::From<std::num::ParseFloatError> for SongError {
    fn from(err: std::num::ParseFloatError) -> SongError {
        SongError::ParseFloatError(err)
    }
}

#[derive(Debug, Clone)]
pub enum SyncTrackError {
    ParseIntError(std::num::ParseIntError),
    TSMissingUpper,
    BMissingBPM,
}

impl std::convert::From<std::num::ParseIntError> for SyncTrackError {
    fn from(err: std::num::ParseIntError) -> SyncTrackError {
        SyncTrackError::ParseIntError(err)
    }
}

#[derive(Debug, Clone)]
pub enum EventError {
    ParseIntError(std::num::ParseIntError),
    ESectionMissingSectionName,
}

impl std::convert::From<std::num::ParseIntError> for EventError {
    fn from(err: std::num::ParseIntError) -> EventError {
        EventError::ParseIntError(err)
    }
}

#[derive(Debug, Clone)]
pub enum PartError {
    UnknownInstrumentDifficulty,
    ParseIntError(std::num::ParseIntError),
    NMissingNote,
    NMissingDuration,
}

impl std::convert::From<std::num::ParseIntError> for PartError {
    fn from(err: std::num::ParseIntError) -> PartError {
        PartError::ParseIntError(err)
    }
}

#[derive(Debug, Clone)]
pub enum ChartParseError {
    BadSection,
    MissingSongSection,
    MissingSyncTrackSection,
    SongSectionError(SongError),
    SyncTrackSectionError(SyncTrackError),
    EventSectionError(EventError),
    PartSectionError(PartError),
}

impl std::convert::From<SongError> for ChartParseError {
    fn from(err: SongError) -> ChartParseError {
        ChartParseError::SongSectionError(err)
    }
}

impl std::convert::From<SyncTrackError> for ChartParseError {
    fn from(err: SyncTrackError) -> ChartParseError {
        ChartParseError::SyncTrackSectionError(err)
    }
}

impl std::convert::From<EventError> for ChartParseError {
    fn from(err: EventError) -> ChartParseError {
        ChartParseError::EventSectionError(err)
    }
}

impl std::convert::From<PartError> for ChartParseError {
    fn from(err: PartError) -> ChartParseError {
        ChartParseError::PartSectionError(err)
    }
}

fn build_song<'a>(song_entries: impl Iterator<Item = (&'a str, &'a str)>) -> Result<Song, SongError> {
    let mut fields: HashMap<String, String> = HashMap::new();

    song_entries.for_each(|(key, value)| {
        fields.insert(key.to_ascii_lowercase(), value.to_string());
    });

    type Fields<'a> = &'a mut HashMap<String, String>;

    let take = |fields: Fields, key| fields.remove(key);
    // FIXME: trim in place
    let take_quoted = |fields: Fields, key| take(fields, key).map(|s| s.trim_matches('"').to_string());
    let take_int = |fields: Fields, key| take(fields, key).map(|s| s.parse::<u64>())
        .map_or(Ok(None), |r| r.map(Some));
    let take_float = |fields: Fields, key| take(fields, key).map(|s| s.parse::<f32>())
        .map_or(Ok(None), |r| r.map(Some));

    // FIXME: substring in place
    let cleaned_year = |year_str: String|
        if year_str.starts_with(", ") { year_str[2..].to_string() } else { year_str };

    let parse_player_2 = |player2_str: String|
        match player2_str.to_lowercase().as_ref() {
            "bass" => Some(SongPlayer2::Bass),
            "rhythm" => Some(SongPlayer2::Rhythm),
            _ => None,
        };

    // resolution is the only non-optional field. An error is raised if it is not present.
    // All other fields are optional; however if a field is present and fails to parse, an error is raised.
    return Ok(Song {
        name:          take_quoted(&mut fields, "name"),
        artist:        take_quoted(&mut fields, "artist"),
        charter:       take_quoted(&mut fields, "charter"),
        album:         take_quoted(&mut fields, "album"),
        year:          take_quoted(&mut fields, "year").map(cleaned_year),
        offset:           take_int(&mut fields, "offset")?,
        resolution:       take_int(&mut fields, "resolution")?.ok_or_else(|| SongError::MissingResolution)?,
        player2:              take(&mut fields, "player2").and_then(parse_player_2),
        difficulty:       take_int(&mut fields, "difficulty")?,
        preview_start:  take_float(&mut fields, "previewstart")?,
        preview_end:    take_float(&mut fields, "previewend")?,
        genre:         take_quoted(&mut fields, "genre"),
        media_type:    take_quoted(&mut fields, "mediatype"),
        streams: SongStreams {
            music:  take_quoted(&mut fields, "musicstream"),
            guitar: take_quoted(&mut fields, "guitarstream"),
            bass:   take_quoted(&mut fields, "bassstream"),
            rhythm: take_quoted(&mut fields, "rhythmstream"),
            drum:   take_quoted(&mut fields, "drumstream"),
        },
    });
}

fn build_synctrack<'a>(entries: impl Iterator<Item = (&'a str, &'a str)>) -> Result<Vec<SyncTrack>, SyncTrackError> {
    return entries
        .map(|(key, value)| -> Result<Option<SyncTrack>, SyncTrackError> {
            let parts: Vec<&str> = value.split(' ').collect();
            match parts.first().map(|s| s.as_ref()) {
                Some("TS") => Ok(Some(SyncTrack::TimeSignature {
                    ticks: key.parse::<u64>()?,
                    upper: parts.get(1).ok_or_else(|| SyncTrackError::TSMissingUpper)?.parse::<u64>()?,
                    // From what I can gather, the original Feedback .chart files only had a numerator.
                    // Moonscraper .chart files introduced the convention of storing log_2(denominator).
                    lower: parts.get(2).map(|s| s.parse::<u32>()).unwrap_or(Ok(2)).map(|n| 2u64.pow(n))?,
                })),
                Some("B") => Ok(Some(SyncTrack::BeatsPerMinute {
                    ticks: key.parse::<u64>()?,
                    // The BPM is stored as an integer BPM * 1000
                    bpm1000: parts.get(1).ok_or_else(|| SyncTrackError::BMissingBPM)?.parse::<u64>()?,
                })),
                // Ignore unknown event types
                _ => Ok(None),
            }
        })
        .filter_map(|s| s.map_or_else(|e| Some(Err(e)), |s| s.map(Ok)))
        .collect();
}

fn build_events<'a>(entries: impl Iterator<Item = (&'a str, &'a str)>) -> Result<Vec<Event>, EventError> {
    return entries
        .map(|(key, value)| -> Result<Option<Event>, EventError> {
            let mut parts = value.splitn(2, ' ');
            let event_type = parts.next();
            let event_str = parts.next().map(|s| s.trim_matches('"'));

            let (event_subtype, event_param) = (|| {
                let event_parts = event_str.map(|s| s.splitn(2, ' '));
                match event_parts {
                    None => (None, None),
                    Some(mut parts) => {
                        let event_subtype = parts.next();
                        let event_param = parts.next();
                        return (event_subtype, event_param);
                    }
                }
            })();

            match (event_type, event_subtype) {
                (Some("E"), Some("section")) => Ok(Some(Event::Section {
                    ticks: key.parse::<u64>()?,
                    name: event_param.map(|n| n.to_string()).ok_or_else(|| EventError::ESectionMissingSectionName)?,
                })),
                // Ignore unknown event types
                (_, _) => Ok(None),
            }
        })
        .filter_map(|s| s.map_or_else(|e| Some(Err(e)), |s| s.map(Ok)))
        .collect();
}

fn build_part<'a>(name: &'a str, entries: impl Iterator<Item = (&'a str, &'a str)>) -> Result<Part, ChartParseError> {
    let (instrument, difficulty) = match name {
        "ExpertSingle" => (Instrument::Guitar, Difficulty::Expert),
        "HardSingle" => (Instrument::Guitar, Difficulty::Hard),
        "MediumSingle" => (Instrument::Guitar, Difficulty::Medium),
        "EasySingle" => (Instrument::Guitar, Difficulty::Easy),
        "ExpertDoubleBass" => (Instrument::Bass, Difficulty::Expert),
        "HardDoubleBass" => (Instrument::Bass, Difficulty::Hard),
        "MediumDoubleBass" => (Instrument::Bass, Difficulty::Medium),
        "EasyDoubleBass" => (Instrument::Bass, Difficulty::Easy),
        "ExpertKeyboard" => (Instrument::Keyboard, Difficulty::Expert),
        "HardKeyboard" => (Instrument::Keyboard, Difficulty::Hard),
        "MediumKeyboard" => (Instrument::Keyboard, Difficulty::Medium),
        "EasyKeyboard" => (Instrument::Keyboard, Difficulty::Easy),
        "ExpertDrums" => (Instrument::Drums, Difficulty::Expert),
        "HardDrums" => (Instrument::Drums, Difficulty::Hard),
        "MediumDrums" => (Instrument::Drums, Difficulty::Medium),
        "EasyDrums" => (Instrument::Drums, Difficulty::Easy),
        "PART REAL_GUITAR" => (Instrument::RealGuitar, Difficulty::Expert),
        "PART REAL_BASS" => (Instrument::RealBass, Difficulty::Expert),
        "PART REAL_KEYS_X" => (Instrument::RealKeys, Difficulty::Expert),
        "PART REAL_KEYS_H" => (Instrument::RealKeys, Difficulty::Hard),
        "PART REAL_KEYS_M" => (Instrument::RealKeys, Difficulty::Medium),
        "PART REAL_KEYS_E" => (Instrument::RealKeys, Difficulty::Easy),
        // TODO
        _ => (Instrument::Guitar, Difficulty::Expert),
    };

    let notes: Result<Vec<Note>, PartError> = entries
        .map(|(key, value)| -> Result<Option<Note>, PartError> {
            let parts: Vec<&str> = value.split(' ').collect();
            match parts.first().map(|s| s.as_ref()) {
                Some("N") => Ok(Some(Note {
                    ticks: key.parse::<u64>()?,
                    note: parts.get(1).ok_or_else(|| PartError::NMissingNote)?.parse::<u64>()?,
                    duration: parts.get(2).ok_or_else(|| PartError::NMissingDuration)?.parse::<u64>()?,
                })),
                // TODO: handle star power
                // Ignore unknown event types
                _ => Ok(None),
            }
        })
        .filter_map(|s| s.map_or_else(|e| Some(Err(e)), |s| s.map(Ok)))
        .collect();

    // Feedback .chart files have several types of special event,
    // https://github.com/FeedBackDevs/feedback/blob/534d90f266/src/db/chart/event.d#L29
    // while Moonscraper .chart files only have star power (type 2).
    Ok(Part {
        instrument: instrument,
        difficulty: difficulty,
        notes: notes?,
        special_events: vec![], // TODO
    })
}

/// A parser for Moonscraper .chart files.
///
/// Attempts to support all features from chart files understood by Moonscraper.
/// Silently ignores chart sections and events that are present but not known.
/// Converts all machine-intended strings to structured data types.
/// Raises an error on known chart sections and events that are unparseable.
///
/// There are many differences between Moonscraper and Feedback .chart files.
/// This parser adopts the Moonscraper conventions where there is a distinction.
pub fn read(contents: &str) -> Result<Chart, ChartParseError> {
    let mut song: Option<Song> = None;
    let mut sync_track: Option<Vec<SyncTrack>> = None;
    let mut events: Option<Vec<Event>> = None;
    let mut parts: Vec<Part> = Vec::new();

    let re_section: Regex = Regex::new(r"(?m)^\s*\[([^\]]+)\]\s*\{([\w\W]*?\n)\}\s*$").unwrap();
    let re_entry: Regex = Regex::new(r"(?m)^\s*(\w+)\s*=\s*([\w\W]+?)\s*$").unwrap();

    for section_cap in re_section.captures_iter(contents) {
        let name = section_cap.get(1)
            .map(|m| m.as_str())
            .ok_or_else(|| ChartParseError::BadSection)?;
        let entries_str = section_cap.get(2)
            .map(|m| m.as_str())
            .ok_or_else(|| ChartParseError::BadSection)?;

        let entries = re_entry.captures_iter(entries_str)
            .filter_map(|cap| {
                match (cap.get(1), cap.get(2)) {
                    (Some(a), Some(b)) => Some((a.as_str(), b.as_str())),
                    _ => None
                }
            });

        match name {
            "Song" => song = Some(build_song(entries)?),
            "SyncTrack" => sync_track = Some(build_synctrack(entries)?),
            "Events" => events = Some(build_events(entries)?),
            _ => parts.push(build_part(name, entries)?),
        }
    }

    return Ok(Chart {
        song: song.ok_or_else(|| ChartParseError::MissingSongSection)?,
        sync_track: sync_track.ok_or_else(|| ChartParseError::MissingSyncTrackSection)?,
        events: events.unwrap_or_else(|| Vec::new()),
        parts: parts,
    });
}
