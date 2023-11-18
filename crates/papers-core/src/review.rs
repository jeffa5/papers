use chrono::{Days, NaiveDateTime};

use crate::{paper::PaperMeta, repo::now_naive};

const REVIEW_POWER: f64 = 2.0;

impl PaperMeta {
    fn calculate_next_review_date(&self) -> NaiveDateTime {
        let now = now_naive();
        let wait_days = match (self.last_review, self.next_review) {
            (None, None) => 1,
            (None, Some(_next)) => 1,
            (Some(_last), None) => 1,
            (Some(last), Some(next)) => {
                let days_since_last = (next - last).num_days();
                if days_since_last > 1 {
                    (days_since_last as f64).powf(REVIEW_POWER).floor() as u64
                } else {
                    2
                }
            }
        };
        now + Days::new(wait_days)
    }

    pub fn update_review(&mut self) {
        let next_review_date = self.calculate_next_review_date();
        self.last_review = self.next_review;
        self.next_review = Some(next_review_date);
    }

    pub fn is_reviewable(&self) -> bool {
        let now = now_naive();
        // reviewable if next review date is in the past
        self.next_review.map_or(true, |r| r < now)
    }
}
