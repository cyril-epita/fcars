use arff::{ArffReader, Attribute, Value};
use fcars::cnc::{NominalDataset, cnc_bp, display_cnc_results, CncBpResult};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

/// Convert ARFF data to NominalDataset
/// This function handles only nominal (categorical) attributes
fn arff_to_nominal_dataset(
    file_path: &str,
    class_attr_name: &str,
) -> Result<NominalDataset, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let arff_data = ArffReader::new(reader)?;

    // Extract attribute names
    let attributes: Vec<String> = arff_data
        .attributes()
        .iter()
        .map(|attr| attr.name().to_string())
        .collect();

    // Verify class attribute exists
    if !attributes.contains(&class_attr_name.to_string()) {
        return Err(format!("Class attribute '{}' not found in ARFF file", class_attr_name).into());
    }

    // Generate object names (obj_0, obj_1, etc.)
    let mut objects = Vec::new();
    let mut data = Vec::new();

    for (idx, instance) in arff_data.instances().enumerate() {
        objects.push(format!("obj_{}", idx));

        let mut obj_data = HashMap::new();

        for (attr_idx, value) in instance.iter().enumerate() {
            let attr_name = &attributes[attr_idx];

            // Convert ARFF value to string
            let value_str = match value {
                Value::Nominal(v) => v.to_string(),
                Value::Numeric(v) => {
                    // For numeric values, you might want to discretize them
                    // For now, we'll just convert to string
                    println!("Warning: Numeric attribute '{}' found. Consider discretizing it first.", attr_name);
                    v.to_string()
                }
                Value::Missing => {
                    println!("Warning: Missing value for attribute '{}' in object {}", attr_name, idx);
                    "?".to_string()
                }
                _ => {
                    println!("Warning: Unsupported value type for attribute '{}'", attr_name);
                    "?".to_string()
                }
            };

            obj_data.insert(attr_name.clone(), value_str);
        }

        data.push(obj_data);
    }

    Ok(NominalDataset::new(
        objects,
        attributes,
        class_attr_name.to_string(),
        data,
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Weather dataset (fully nominal)
    println!("=== Example 1: Weather Dataset ===\n");

    let dataset = arff_to_nominal_dataset(
        "weather.nominal.arff",
        "play"  // class attribute
    )?;

    dataset.display_summary();

    // Run CNC-BP with n=1 (keep only the most minority class)
    println!("\n--- Running CNC-BP (n=1) ---");
    let cnc_bp_result = cnc_bp(&dataset, 1);
    display_cnc_bp_results(&dataset, &cnc_bp_result);

    println!("\n");

    // Example 2: Iris dataset (needs to be discretized if numeric)
    // Note: iris.arff might have numeric attributes that need discretization
    println!("=== Example 2: Contact Lenses Dataset ===\n");

    let dataset2 = arff_to_nominal_dataset(
        "contact-lenses.arff",
        "contact-lenses"  // Adjust this to the actual class attribute name
    )?;

    dataset2.display_summary();

    println!("\n--- Running CNC-BP (n=2) ---");
    let cnc_bp_result2 = cnc_bp(&dataset2, 2);
    display_cnc_bp_results(&dataset2, &cnc_bp_result2);

    Ok(())
}

/// Display CNC-BP results including filtering information
fn display_cnc_bp_results(dataset: &NominalDataset, result: &CncBpResult) {
    println!("\nCNC-BP Results:");
    println!("- Original dataset size: {} objects", result.original_size);
    println!("- Filtered dataset size: {} objects ({:.1}%)",
             result.filtered_size,
             (result.filtered_size as f64 / result.original_size as f64) * 100.0);
    println!("- Minority classes kept: {:?}", result.minority_classes);

    display_cnc_results(dataset, &result.cnc_result.concepts);
}
