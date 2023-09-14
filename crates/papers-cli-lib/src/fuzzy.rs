use papers_core::paper::ExportPaperData;
use skim::prelude::*;
use std::sync::Arc;

struct FuzzyPaper(ExportPaperData);

/// Select a paper by fuzzy searching them.
pub fn select_paper(papers: Vec<ExportPaperData>) -> Option<ExportPaperData> {
    select_papers_inner(papers, false).first().cloned()
}

/// Select multiple papers by fuzzy searching them.
pub fn select_papers(papers: Vec<ExportPaperData>) -> Vec<ExportPaperData> {
    select_papers_inner(papers, true)
}

fn select_papers_inner(papers: Vec<ExportPaperData>, multi: bool) -> Vec<ExportPaperData> {
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
        let title = &self.0.title;
        let authors = self
            .0
            .authors
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("title:{:?} authors:{:?}", title, authors).into()
    }
}
