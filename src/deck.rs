use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum DeckStatus {
    InProgress,
    Completed,
    Abandoned,
}

impl DeckStatus {
    pub fn as_str(&self) -> &str {
        match self {
            DeckStatus::InProgress => "in_progress",
            DeckStatus::Completed => "completed",
            DeckStatus::Abandoned => "abandoned",
        }
    }

    pub fn from(s: &str) -> Option<Self> {
        match s {
            "in_progress" => Some(DeckStatus::InProgress),
            "completed" => Some(DeckStatus::Completed),
            "abandoned" => Some(DeckStatus::Abandoned),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Deck {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: DeckStatus,
    pub total_questions: i32,
    pub correct_answers: i32,
    pub incorrect_answers: i32,
    pub total_time_seconds: f64,
    pub average_time_seconds: Option<f64>,
    pub accuracy_percentage: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct DeckSummary {
    pub total_questions: i32,
    pub correct_answers: i32,
    pub incorrect_answers: i32,
    pub total_time_seconds: f64,
    pub average_time_seconds: f64,
    pub accuracy_percentage: f64,
}

impl DeckSummary {
    pub fn from_results(results: &[(bool, f64)]) -> Self {
        let total_questions = results.len() as i32;
        let correct_answers = results.iter().filter(|(correct, _)| *correct).count() as i32;
        let incorrect_answers = total_questions - correct_answers;
        let total_time_seconds: f64 = results.iter().map(|(_, time)| time).sum();
        let average_time_seconds = if total_questions > 0 {
            total_time_seconds / total_questions as f64
        } else {
            0.0
        };
        let accuracy_percentage = if total_questions > 0 {
            (correct_answers as f64 / total_questions as f64) * 100.0
        } else {
            0.0
        };

        DeckSummary {
            total_questions,
            correct_answers,
            incorrect_answers,
            total_time_seconds,
            average_time_seconds,
            accuracy_percentage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_status_as_str() {
        assert_eq!(DeckStatus::InProgress.as_str(), "in_progress");
        assert_eq!(DeckStatus::Completed.as_str(), "completed");
        assert_eq!(DeckStatus::Abandoned.as_str(), "abandoned");
    }

    #[test]
    fn test_deck_status_from_str() {
        assert_eq!(
            DeckStatus::from("in_progress"),
            Some(DeckStatus::InProgress)
        );
        assert_eq!(DeckStatus::from("completed"), Some(DeckStatus::Completed));
        assert_eq!(DeckStatus::from("abandoned"), Some(DeckStatus::Abandoned));
        assert_eq!(DeckStatus::from("invalid"), None);
    }

    #[test]
    fn test_deck_summary_calculation() {
        let results = vec![(true, 2.0), (true, 3.0), (false, 5.0), (true, 1.0)];

        let summary = DeckSummary::from_results(&results);

        assert_eq!(summary.total_questions, 4);
        assert_eq!(summary.correct_answers, 3);
        assert_eq!(summary.incorrect_answers, 1);
        assert_eq!(summary.total_time_seconds, 11.0);
        assert_eq!(summary.average_time_seconds, 2.75);
        assert_eq!(summary.accuracy_percentage, 75.0);
    }

    #[test]
    fn test_deck_summary_all_correct() {
        let results = vec![(true, 1.0), (true, 2.0), (true, 3.0)];
        let summary = DeckSummary::from_results(&results);

        assert_eq!(summary.accuracy_percentage, 100.0);
        assert_eq!(summary.correct_answers, 3);
        assert_eq!(summary.incorrect_answers, 0);
    }

    #[test]
    fn test_deck_summary_all_incorrect() {
        let results = vec![(false, 1.0), (false, 2.0), (false, 3.0)];
        let summary = DeckSummary::from_results(&results);

        assert_eq!(summary.accuracy_percentage, 0.0);
        assert_eq!(summary.correct_answers, 0);
        assert_eq!(summary.incorrect_answers, 3);
    }

    #[test]
    fn test_deck_summary_empty() {
        let results: Vec<(bool, f64)> = vec![];
        let summary = DeckSummary::from_results(&results);

        assert_eq!(summary.total_questions, 0);
        assert_eq!(summary.accuracy_percentage, 0.0);
    }
}
