use askama::{Error as AskamaError, Template};
use std::collections::HashMap;

use crate::cnc::{
    CncBpResult, CncConcept, CncResult, NominalDataset, attribute_information_gains,
    filter_dataset_by_classes, most_frequent_values, summarize_dataset,
};

mod view;

use self::view::{
    ConceptView, CountShareValue, DatasetSectionView, InformationGainRowView, LabelValue,
    ReportPageView, ReportTemplate,
};

pub fn render_cnc_report_html(
    dataset: &NominalDataset,
    results: &CncResult,
    title: Option<&str>,
) -> Result<String, AskamaError> {
    let page = ReportPageView {
        title: title.unwrap_or("CNC Report").to_string(),
        lead: "Formal Concept Analysis report for a nominal dataset classified with CNC."
            .to_string(),
        filter_rows: Vec::new(),
        dataset_sections: vec![build_dataset_section_view("Dataset", dataset)],
        information_gain_rows: build_information_gain_rows(dataset, &results.pertinent_attrs),
        concept_count: results.concepts.len(),
        concepts: build_concepts(dataset, results),
    };

    ReportTemplate { page }.render()
}

pub fn render_cnc_bp_report_html(
    dataset: &NominalDataset,
    bp_result: &CncBpResult,
    title: Option<&str>,
) -> Result<String, AskamaError> {
    let filtered_dataset = filter_dataset_by_classes(dataset, &bp_result.minority_classes);
    let mut minority_classes = bp_result
        .minority_classes
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    minority_classes.sort();

    let page = ReportPageView {
        title: title.unwrap_or("CNC-BP Report").to_string(),
        lead: "Formal Concept Analysis report for CNC-BP. Pertinent attributes are selected on the original dataset, then concepts are computed on the minority-class subset.".to_string(),
        filter_rows: vec![
            LabelValue {
                label: "Original objects".to_string(),
                value: bp_result.original_size.to_string(),
            },
            LabelValue {
                label: "Filtered objects".to_string(),
                value: bp_result.filtered_size.to_string(),
            },
            LabelValue {
                label: "Minority classes kept".to_string(),
                value: minority_classes.join(", "),
            },
        ],
        dataset_sections: vec![
            build_dataset_section_view("Original Dataset", dataset),
            build_dataset_section_view("Filtered Dataset", &filtered_dataset),
        ],
        information_gain_rows: build_information_gain_rows(
            dataset,
            &bp_result.cnc_result.pertinent_attrs,
        ),
        concept_count: bp_result.cnc_result.concepts.len(),
        concepts: build_concepts(&filtered_dataset, &bp_result.cnc_result),
    };

    ReportTemplate { page }.render()
}

fn build_dataset_section_view(heading: &str, dataset: &NominalDataset) -> DatasetSectionView {
    let summary = summarize_dataset(dataset);
    let descriptive_headers = dataset
        .attributes
        .iter()
        .filter(|attr| *attr != &dataset.class_attribute)
        .cloned()
        .collect::<Vec<_>>();
    let mut context_headers = Vec::with_capacity(descriptive_headers.len() + 2);
    context_headers.push("Object".to_string());
    context_headers.extend(descriptive_headers.iter().cloned());
    context_headers.push(summary.class_attribute.clone());

    let context_rows = dataset
        .objects
        .iter()
        .enumerate()
        .map(|(index, object)| {
            let mut row = Vec::with_capacity(descriptive_headers.len() + 2);
            row.push(object.clone());
            row.extend(descriptive_headers.iter().map(|attr| {
                dataset.data[index]
                    .get(attr)
                    .cloned()
                    .unwrap_or_else(|| "?".to_string())
            }));
            row.push(
                dataset.data[index]
                    .get(&dataset.class_attribute)
                    .cloned()
                    .unwrap_or_else(|| "?".to_string()),
            );
            row
        })
        .collect();

    DatasetSectionView {
        heading: heading.to_string(),
        overview_rows: vec![
            LabelValue {
                label: "Objects".to_string(),
                value: summary.objects.to_string(),
            },
            LabelValue {
                label: "Descriptive attributes".to_string(),
                value: summary.descriptive_attributes.to_string(),
            },
            LabelValue {
                label: "Class attribute".to_string(),
                value: summary.class_attribute.clone(),
            },
        ],
        context_headers,
        context_rows,
        attribute_rows: summary
            .attribute_unique_values
            .into_iter()
            .map(|(attribute, unique_values)| LabelValue {
                label: attribute,
                value: unique_values.to_string(),
            })
            .collect(),
        class_distribution: summary
            .class_distribution
            .into_iter()
            .map(|(class_name, count, percentage)| CountShareValue {
                name: class_name,
                count,
                percentage: format_percentage(percentage),
            })
            .collect(),
    }
}

fn build_information_gain_rows(
    dataset: &NominalDataset,
    pertinent_attributes: &[String],
) -> Vec<InformationGainRowView> {
    attribute_information_gains(dataset)
        .into_iter()
        .map(|gain| InformationGainRowView {
            most_frequent_values: most_frequent_values(dataset, &gain.attribute),
            chosen: pertinent_attributes.contains(&gain.attribute),
            attribute: gain.attribute,
            gain: format!("{:.6}", gain.gain),
        })
        .collect()
}

fn build_concepts(dataset: &NominalDataset, results: &CncResult) -> Vec<ConceptView> {
    let descriptive_attribute_count = dataset
        .attributes
        .iter()
        .filter(|attr| *attr != &dataset.class_attribute)
        .count();

    results
        .concepts
        .iter()
        .enumerate()
        .map(|(index, concept)| build_concept_view(index, dataset, concept, descriptive_attribute_count))
        .collect()
}

fn build_concept_view(
    index: usize,
    dataset: &NominalDataset,
    concept: &CncConcept,
    descriptive_attribute_count: usize,
) -> ConceptView {
    let extent_objects = concept.extent
        .iter()
        .map(|&object_index| dataset.objects[object_index].clone())
        .collect::<Vec<_>>();
    let class_values = dataset.get_class_values(&concept.extent);
    let class_distribution = summarize_class_values(&class_values);
    let majority_summary = NominalDataset::get_majority_class(&class_values).map_or_else(
        String::new,
        |(majority_class, count, percentage)| {
            format!(
                "{} ({}/{}, {})",
                majority_class,
                count,
                concept.extent.len(),
                format_percentage(percentage)
            )
        },
    );

    let mut intent_entries = concept.intent
        .iter()
        .map(|(attribute, value)| LabelValue {
            label: attribute.clone(),
            value: value.clone(),
        })
        .collect::<Vec<_>>();
    intent_entries.sort_by(|left, right| left.label.cmp(&right.label));

    ConceptView {
        title: format!("Concept {}", index + 1),
        summary_rows: vec![
            LabelValue {
                label: "Pertinent attribute".to_string(),
                value: concept.pertinent_attribute.clone(),
            },
            LabelValue {
                label: "Chosen value".to_string(),
                value: concept.attribute_value.clone(),
            },
            LabelValue {
                label: "Extent size".to_string(),
                value: format!(
                    "{}/{} ({})",
                    concept.extent.len(),
                    dataset.objects.len(),
                    format_percentage(share(concept.extent.len(), dataset.objects.len()))
                ),
            },
            LabelValue {
                label: "Intent size".to_string(),
                value: format!(
                    "{}/{} ({})",
                    concept.intent.len(),
                    descriptive_attribute_count,
                    format_percentage(share(concept.intent.len(), descriptive_attribute_count))
                ),
            },
        ],
        extent_objects,
        intent_rows: intent_entries,
        class_distribution: class_distribution
            .into_iter()
            .map(|(class_name, count, percentage)| CountShareValue {
                name: class_name,
                count,
                percentage: format_percentage(percentage),
            })
            .collect(),
        majority_summary,
    }
}

fn summarize_class_values(class_values: &[String]) -> Vec<(String, usize, f64)> {
    let mut class_counts = HashMap::new();
    for class_val in class_values {
        *class_counts.entry(class_val.clone()).or_insert(0) += 1;
    }

    let total = class_values.len();
    let mut distribution = class_counts
        .into_iter()
        .map(|(class_name, count)| {
            let percentage = if total == 0 {
                0.0
            } else {
                (count as f64 / total as f64) * 100.0
            };
            (class_name, count, percentage)
        })
        .collect::<Vec<_>>();
    distribution.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    distribution
}

fn share(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (count as f64 / total as f64) * 100.0
    }
}

fn format_percentage(value: f64) -> String {
    format!("{value:.1}%")
}

#[cfg(test)]
mod tests {
    use super::{render_cnc_bp_report_html, render_cnc_report_html};
    use crate::cnc::{NominalDataset, cnc, cnc_bp};
    use std::collections::HashMap;

    fn create_weather_dataset() -> NominalDataset {
        let objects = vec![
            "o2".to_string(),
            "o6".to_string(),
            "o9".to_string(),
            "o10".to_string(),
            "o13".to_string(),
        ];

        let attributes = vec![
            "Outlook".to_string(),
            "Temperature".to_string(),
            "Humidity".to_string(),
            "Windy".to_string(),
            "Play".to_string(),
        ];

        let data = vec![
            HashMap::from([
                ("Outlook".to_string(), "Sunny".to_string()),
                ("Temperature".to_string(), "Hot".to_string()),
                ("Humidity".to_string(), "High".to_string()),
                ("Windy".to_string(), "True".to_string()),
                ("Play".to_string(), "No".to_string()),
            ]),
            HashMap::from([
                ("Outlook".to_string(), "Rainy".to_string()),
                ("Temperature".to_string(), "Cool".to_string()),
                ("Humidity".to_string(), "Normal".to_string()),
                ("Windy".to_string(), "True".to_string()),
                ("Play".to_string(), "No".to_string()),
            ]),
            HashMap::from([
                ("Outlook".to_string(), "Sunny".to_string()),
                ("Temperature".to_string(), "Cool".to_string()),
                ("Humidity".to_string(), "Normal".to_string()),
                ("Windy".to_string(), "False".to_string()),
                ("Play".to_string(), "Yes".to_string()),
            ]),
            HashMap::from([
                ("Outlook".to_string(), "Rainy".to_string()),
                ("Temperature".to_string(), "Mild".to_string()),
                ("Humidity".to_string(), "Normal".to_string()),
                ("Windy".to_string(), "False".to_string()),
                ("Play".to_string(), "Yes".to_string()),
            ]),
            HashMap::from([
                ("Outlook".to_string(), "Overcast".to_string()),
                ("Temperature".to_string(), "Hot".to_string()),
                ("Humidity".to_string(), "Normal".to_string()),
                ("Windy".to_string(), "False".to_string()),
                ("Play".to_string(), "Yes".to_string()),
            ]),
        ];

        NominalDataset::new(objects, attributes, "Play".to_string(), data)
    }

    #[test]
    fn cnc_report_template_renders_information_gain() {
        let dataset = create_weather_dataset();
        let results = cnc(&dataset);
        let html = render_cnc_report_html(&dataset, &results, Some("Weather")).unwrap();

        assert!(html.contains("Information Gain"));
        assert!(html.contains("Windy"));
        assert!(html.contains("Concept 1"));
    }

    #[test]
    fn cnc_bp_report_template_renders_filter_section() {
        let dataset = create_weather_dataset();
        let results = cnc_bp(&dataset, 1);
        let html = render_cnc_bp_report_html(&dataset, &results, Some("Weather BP")).unwrap();

        assert!(html.contains("Minority-Class Filter"));
        assert!(html.contains("Filtered Dataset"));
        assert!(html.contains("Concept 1"));
    }
}
