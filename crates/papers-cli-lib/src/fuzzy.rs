use papers_core::paper::{LoadedPaper, PaperMeta};
use skim::prelude::*;
use std::sync::Arc;

struct FuzzyPaper(LoadedPaper);

/// Select a paper by fuzzy searching them.
pub fn select_paper(papers: &[LoadedPaper]) -> Option<LoadedPaper> {
    select_papers_inner(papers, false).first().cloned()
}

/// Select multiple papers by fuzzy searching them.
pub fn select_papers(papers: &[LoadedPaper]) -> Vec<LoadedPaper> {
    select_papers_inner(papers, true)
}

fn select_papers_inner(papers: &[LoadedPaper], multi: bool) -> Vec<LoadedPaper> {
    // lines skim adds
    let ui_lines = 2;
    let height = papers.len() + ui_lines;
    let height = height.to_string();

    let options = SkimOptionsBuilder::default()
        .height(Some(&height))
        .multi(multi)
        .build()
        .unwrap();

    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
    for paper in papers {
        let p = FuzzyPaper(paper.clone());
        tx_item.send(Arc::new(p)).unwrap();
    }
    drop(tx_item);

    let skim_result = match Skim::run_with(&options, Some(rx_item)) {
        Some(result) => result,
        None => return Vec::new(),
    };

    // don't continue if the user actually aborted rather than selecting
    if skim_result.is_abort {
        return Vec::new();
    }

    let selected_papers = skim_result.selected_items.iter().map(|item| {
        (**item)
            .as_any()
            .downcast_ref::<FuzzyPaper>()
            .unwrap()
            .to_owned()
    });

    selected_papers.map(|p| p.0.clone()).collect()
}

impl SkimItem for FuzzyPaper {
    fn text(&self) -> Cow<str> {
        let PaperMeta {
            title,
            url: _,
            filename: _,
            tags,
            labels,
            authors,
            created_at: _,
            modified_at: _,
            last_review: _,
            next_review: _,
        } = &self.0.meta;
        let authors = authors
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let tags = tags
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let labels = labels
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "title:{:?} authors:{:?} tags:{:?} labels:{:?}",
            title, authors, tags, labels
        )
        .into()
    }
}
