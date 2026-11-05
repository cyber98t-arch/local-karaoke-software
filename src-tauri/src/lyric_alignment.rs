use crate::analysis_worker::TranscriptionSegment;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlignmentStatus {
    Matched,
    Unmatched,
    Ignored,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlignedLyricCharacter {
    pub text: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
    pub status: AlignmentStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LyricAlignment {
    pub original_text: String,
    pub characters: Vec<AlignedLyricCharacter>,
}

struct ComparisonCharacter {
    value: char,
    output_index: usize,
}
struct TimedCharacter {
    value: char,
    start_ms: u64,
    end_ms: u64,
}

pub fn align_japanese_lyrics(
    authoritative_lyrics: &str,
    transcript: &[TranscriptionSegment],
) -> Result<LyricAlignment, String> {
    if authoritative_lyrics.trim().is_empty() {
        return Err("authoritative lyrics must not be empty".to_owned());
    }
    validate_transcript(transcript)?;
    let mut output = authoritative_lyrics
        .char_indices()
        .map(|(byte_start, character)| AlignedLyricCharacter {
            text: character.to_string(),
            byte_start,
            byte_end: byte_start + character.len_utf8(),
            start_ms: None,
            end_ms: None,
            status: if comparison_character(character).is_some() {
                AlignmentStatus::Unmatched
            } else {
                AlignmentStatus::Ignored
            },
        })
        .collect::<Vec<_>>();
    let expected = output
        .iter()
        .enumerate()
        .filter_map(|(output_index, character)| {
            comparison_character(character.text.chars().next().expect("one character")).map(
                |value| ComparisonCharacter {
                    value,
                    output_index,
                },
            )
        })
        .collect::<Vec<_>>();
    if expected.is_empty() {
        return Err("authoritative lyrics contain no comparable characters".to_owned());
    }
    let observed = timed_characters(transcript);
    if observed.is_empty() {
        return Err("transcript contains no comparable characters".to_owned());
    }
    for (expected_index, observed_index) in matched_pairs(&expected, &observed) {
        let destination = &mut output[expected[expected_index].output_index];
        destination.status = AlignmentStatus::Matched;
        destination.start_ms = Some(observed[observed_index].start_ms);
        destination.end_ms = Some(observed[observed_index].end_ms);
    }
    Ok(LyricAlignment {
        original_text: authoritative_lyrics.to_owned(),
        characters: output,
    })
}

fn matched_pairs(
    expected: &[ComparisonCharacter],
    observed: &[TimedCharacter],
) -> Vec<(usize, usize)> {
    let rows = expected.len() + 1;
    let columns = observed.len() + 1;
    let mut costs = vec![0_usize; rows * columns];
    for row in 0..rows {
        costs[row * columns] = row;
    }
    for column in 0..columns {
        costs[column] = column;
    }
    for row in 1..rows {
        for column in 1..columns {
            let substitution = usize::from(expected[row - 1].value != observed[column - 1].value);
            costs[row * columns + column] = (costs[(row - 1) * columns + column - 1]
                + substitution)
                .min(costs[(row - 1) * columns + column] + 1)
                .min(costs[row * columns + column - 1] + 1);
        }
    }
    let mut aligned = Vec::new();
    let (mut row, mut column) = (expected.len(), observed.len());
    while row > 0 || column > 0 {
        if row > 0 && column > 0 {
            let substitution = usize::from(expected[row - 1].value != observed[column - 1].value);
            if costs[row * columns + column]
                == costs[(row - 1) * columns + column - 1] + substitution
            {
                if substitution == 0 {
                    aligned.push((row - 1, column - 1));
                }
                row -= 1;
                column -= 1;
                continue;
            }
        }
        if row > 0 && costs[row * columns + column] == costs[(row - 1) * columns + column] + 1 {
            row -= 1;
        } else {
            column -= 1;
        }
    }
    aligned.reverse();
    aligned
}

fn timed_characters(transcript: &[TranscriptionSegment]) -> Vec<TimedCharacter> {
    transcript
        .iter()
        .flat_map(|segment| {
            let characters = segment
                .text
                .chars()
                .filter_map(comparison_character)
                .collect::<Vec<_>>();
            let count = characters.len() as u64;
            characters
                .into_iter()
                .enumerate()
                .map(move |(index, value)| {
                    let index = index as u64;
                    let duration = segment.end_ms - segment.start_ms;
                    TimedCharacter {
                        value,
                        start_ms: segment.start_ms + duration * index / count,
                        end_ms: segment.start_ms + duration * (index + 1) / count,
                    }
                })
        })
        .collect()
}

fn comparison_character(character: char) -> Option<char> {
    if character.is_whitespace()
        || character.is_ascii_punctuation()
        || matches!(
            character,
            '、' | '。'
                | '・'
                | '「'
                | '」'
                | '『'
                | '』'
                | '【'
                | '】'
                | '！'
                | '？'
                | '〜'
                | '…'
        )
    {
        None
    } else {
        Some(character.to_ascii_lowercase())
    }
}

fn validate_transcript(transcript: &[TranscriptionSegment]) -> Result<(), String> {
    if transcript.is_empty() {
        return Err("transcript must contain segments".to_owned());
    }
    let mut previous_end = 0;
    for segment in transcript {
        if segment.start_ms >= segment.end_ms
            || segment.start_ms < previous_end
            || segment.text.trim().is_empty()
        {
            return Err("transcript segments must be ordered, bounded, and nonempty".to_owned());
        }
        previous_end = segment.end_ms;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{align_japanese_lyrics, AlignmentStatus};
    use crate::analysis_worker::TranscriptionSegment;
    fn segment(start_ms: u64, end_ms: u64, text: &str) -> TranscriptionSegment {
        TranscriptionSegment {
            start_ms,
            end_ms,
            text: text.to_owned(),
        }
    }

    #[test]
    fn aligns_exact_japanese_text_with_character_timings() {
        let aligned = align_japanese_lyrics("今日は", &[segment(100, 500, "今日は")]).unwrap();
        let matched = aligned
            .characters
            .iter()
            .filter(|item| item.status == AlignmentStatus::Matched)
            .collect::<Vec<_>>();
        assert_eq!(matched.len(), 3);
        assert_eq!(
            (matched[0].start_ms, matched[0].end_ms),
            (Some(100), Some(233))
        );
        assert_eq!(
            (matched[2].start_ms, matched[2].end_ms),
            (Some(366), Some(500))
        );
    }

    #[test]
    fn leaves_recognition_errors_unmatched_without_inventing_timing() {
        let aligned = align_japanese_lyrics("天気です", &[segment(0, 500, "電気です")]).unwrap();
        assert_eq!(aligned.characters[0].status, AlignmentStatus::Unmatched);
        assert_eq!(aligned.characters[0].start_ms, None);
        assert_eq!(aligned.characters[1].status, AlignmentStatus::Matched);
        assert_eq!(aligned.characters[2].status, AlignmentStatus::Matched);
    }

    #[test]
    fn preserves_original_spacing_and_punctuation_for_display() {
        let source = "今日は、 晴れ！";
        let aligned = align_japanese_lyrics(source, &[segment(0, 600, "今日は晴れ")]).unwrap();
        assert_eq!(aligned.original_text, source);
        assert_eq!(
            aligned
                .characters
                .iter()
                .map(|item| item.text.as_str())
                .collect::<String>(),
            source
        );
        assert_eq!(aligned.characters[3].status, AlignmentStatus::Ignored);
        assert_eq!(aligned.characters[4].status, AlignmentStatus::Ignored);
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(align_japanese_lyrics("", &[segment(0, 100, "歌")]).is_err());
        assert!(align_japanese_lyrics("歌", &[]).is_err());
        assert!(align_japanese_lyrics("歌", &[segment(100, 100, "歌")]).is_err());
    }
}
