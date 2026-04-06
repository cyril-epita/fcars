use askama::Template;

#[derive(Template)]
#[template(path = "report.html")]
pub(super) struct ReportTemplate {
    pub(super) page: ReportPageView,
}

#[derive(Debug)]
pub(super) struct ReportPageView {
    pub(super) title: String,
    pub(super) lead: String,
    pub(super) filter_rows: Vec<LabelValue>,
    pub(super) dataset_sections: Vec<DatasetSectionView>,
    pub(super) information_gain_rows: Vec<InformationGainRowView>,
    pub(super) concept_count: usize,
    pub(super) concepts: Vec<ConceptView>,
}

#[derive(Debug)]
pub(super) struct DatasetSectionView {
    pub(super) heading: String,
    pub(super) overview_rows: Vec<LabelValue>,
    pub(super) context_headers: Vec<String>,
    pub(super) context_rows: Vec<Vec<String>>,
    pub(super) attribute_rows: Vec<LabelValue>,
    pub(super) class_distribution: Vec<CountShareValue>,
}

#[derive(Debug)]
pub(super) struct InformationGainRowView {
    pub(super) attribute: String,
    pub(super) gain: String,
    pub(super) chosen: bool,
    pub(super) most_frequent_values: Vec<String>,
}

#[derive(Debug)]
pub(super) struct ConceptView {
    pub(super) title: String,
    pub(super) summary_rows: Vec<LabelValue>,
    pub(super) extent_objects: Vec<String>,
    pub(super) intent_rows: Vec<LabelValue>,
    pub(super) class_distribution: Vec<CountShareValue>,
    pub(super) majority_summary: String,
}

#[derive(Debug)]
pub(super) struct LabelValue {
    pub(super) label: String,
    pub(super) value: String,
}

#[derive(Debug)]
pub(super) struct CountShareValue {
    pub(super) name: String,
    pub(super) count: usize,
    pub(super) percentage: String,
}
