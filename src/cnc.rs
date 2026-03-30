use std::collections::{HashMap, HashSet};

/// Result structure for CNC containing both the concepts and debug information
#[derive(Debug)]
pub struct CncResult {
    pub concepts: Vec<(String, String, Vec<usize>, HashMap<String, String>)>,
    pub pertinent_attrs: Vec<String>,
}

/// Result structure for CNC-BP containing both the concepts and debug information
/// Uses CncResult to avoid duplication and maintain consistency
#[derive(Debug)]
pub struct CncBpResult {
    pub cnc_result: CncResult,
    pub minority_classes: HashSet<String>,
    pub original_size: usize,
    pub filtered_size: usize,
}

/// CNC (Classifier Nominal Concept)
/// A classifier that uses Formal Concept Analysis to extract concepts from nominal (multi-valued) data.
/// The algorithm finds the most pertinent attribute and computes its closure (concept).

/// Nominal dataset structure for CNC
#[derive(Debug, Clone)]
pub struct NominalDataset {
    pub objects: Vec<String>,
    pub attributes: Vec<String>,
    pub class_attribute: String,
    pub data: Vec<HashMap<String, String>>, // Each object's attribute values
}

impl std::fmt::Display for NominalDataset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print header - include all attributes with class clearly separated
        write!(f, "{:>10}", "")?;
        for attr in &self.attributes {
            if attr != &self.class_attribute {
                write!(f, "{:>15}", attr)?;
            }
        }
        // Add visual separator and class column together
        write!(f, "       │  {:>8}", self.class_attribute)?;
        writeln!(f)?;
        
        // Print separator
        write!(f, "{:>10}", "")?;
        for attr in &self.attributes {
            if attr != &self.class_attribute {
                write!(f, "{:>15}", "")?;
            }
        }
        // Add visual separator in the separator line
        write!(f, "       │  {:>8}", "")?;
        writeln!(f)?;
        
        // Print each row
        for (i, obj) in self.objects.iter().enumerate() {
            write!(f, "{:>10}", obj)?;
            // Print descriptive attributes (exclude class)
            for attr in &self.attributes {
                if attr != &self.class_attribute {
                    if let Some(val) = self.data[i].get(attr) {
                        write!(f, "{:>15}", val)?;
                    } else {
                        write!(f, "{:>15}", "?")?;
                    }
                }
            }
            // Add visual separator and class value together
            write!(f, "       │  ")?;
            if let Some(class_val) = self.data[i].get(&self.class_attribute) {
                write!(f, "{:>8}", class_val)?;
            } else {
                write!(f, "{:>8}", "?")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl NominalDataset {
    /// Create a new nominal dataset
    pub fn new(objects: Vec<String>, attributes: Vec<String>, class_attribute: String, data: Vec<HashMap<String, String>>) -> Self {
        Self {
            objects,
            attributes,
            class_attribute,
            data,
        }
    }
    
    /// Get all unique values for an attribute
    pub fn get_attribute_values(&self, attr_name: &str) -> Vec<String> {
        let mut values = HashSet::new();
        for obj_data in &self.data {
            if let Some(val) = obj_data.get(attr_name) {
                values.insert(val.clone());
            }
        }
        let mut result: Vec<String> = values.into_iter().collect();
        result.sort();
        result
    }
    
    /// Group objects by attribute value
    pub fn group_by_attribute_value(&self, attr_name: &str) -> HashMap<String, Vec<usize>> {
        let mut groups = HashMap::new();
        for (obj_idx, obj_data) in self.data.iter().enumerate() {
            if let Some(val) = obj_data.get(attr_name) {
                groups.entry(val.clone()).or_insert_with(Vec::new).push(obj_idx);
            }
        }
        groups
    }
    
    /// Get class values for a group of objects
    pub fn get_class_values(&self, object_indices: &[usize]) -> Vec<String> {
        object_indices.iter()
            .filter_map(|&obj_idx| self.data[obj_idx].get(&self.class_attribute).cloned())
            .collect()
    }
    
    /// Get the majority class from a class distribution
    /// Returns (majority_class, count, percentage)
    pub fn get_majority_class(class_values: &[String]) -> Option<(String, usize, f64)> {
        if class_values.is_empty() {
            return None;
        }
        
        let mut class_counts = HashMap::new();
        
        // Count occurrences of each class
        for class_val in class_values {
            *class_counts.entry(class_val.clone()).or_insert(0) += 1;
        }
        
        // Find the class with maximum count
        let (majority_class, count) = class_counts.into_iter()
            .max_by_key(|(_, count)| *count)?;
        
        let percentage = (count as f64 / class_values.len() as f64) * 100.0;
        
        Some((majority_class, count, percentage))
    }
    
    /// Display summary statistics about the dataset
    pub fn display_summary(&self) {

        println!("Context:\n{}", &self);

        // Count descriptive attributes (excluding class)
        let desc_attrs: Vec<_> = self.attributes.iter()
            .filter(|attr| attr != &&self.class_attribute)
            .collect();
            
        println!("Dataset Summary:");
        println!("- Objects: {}", self.objects.len());
        println!("- Descriptive attributes: {}", desc_attrs.len());
        println!("- Class attribute: {}", self.class_attribute);
        
        for attr in &desc_attrs {
            let values = self.get_attribute_values(attr);
            println!("- Attribute '{}': {} unique values", attr, values.len());
        }
        
        // Show class distribution
        let class_values = self.get_class_values(&(0..self.objects.len()).collect::<Vec<_>>());
        let mut class_counts = HashMap::new();
        for class_val in class_values {
            *class_counts.entry(class_val).or_insert(0) += 1;
        }
        
        println!("- Class distribution:");
        for (class_val, count) in class_counts {
            println!("  {}: {} ({:.1}%)", class_val, count, (count as f64 / self.objects.len() as f64) * 100.0);
        }
    }
}

/// Calculate entropy for nominal values
fn calculate_entropy(values: &[String]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    
    let total = values.len() as f64;
    let mut value_counts = HashMap::new();
    
    // Count occurrences of each value
    for value in values {
        *value_counts.entry(value.clone()).or_insert(0) += 1;
    }
    
    // Calculate entropy: -Σ p_i * log2(p_i)
    value_counts.values().map(|&count| {
        let probability = count as f64 / total;
        -probability * probability.log2()
    }).sum()
}

/// Calculate information gain for a nominal attribute
fn information_gain(dataset: &NominalDataset, attr_name: &str) -> f64 {
    let total_objects = dataset.objects.len() as f64;
    if total_objects == 0.0 {
        return 0.0;
    }
    
    // Get all class values for total entropy calculation
    let all_class_values = dataset.get_class_values(&(0..dataset.objects.len()).collect::<Vec<_>>());
    let total_entropy = calculate_entropy(&all_class_values);
    
    // Group by attribute value and calculate weighted entropy
    let groups = dataset.group_by_attribute_value(attr_name);
    let mut weighted_entropy = 0.0;
    
    for (_, object_indices) in &groups {
        let group_size = object_indices.len() as f64;
        let weight = group_size / total_objects;
        
        // Get class values for this group
        let group_class_values = dataset.get_class_values(object_indices);
        let group_entropy = calculate_entropy(&group_class_values);
        weighted_entropy += weight * group_entropy;
    }
    
    total_entropy - weighted_entropy
}

/// Find the most pertinent attribute using information gain
/// Returns all attributes with maximum gain (handles ties)
fn find_most_pertinent_attributes(dataset: &NominalDataset) -> Vec<String> {
    let mut attr_gains = Vec::new();
    
    for attr_name in &dataset.attributes {
        if attr_name == &dataset.class_attribute {
            continue; // Skip class attribute
        }
        
        let gain = information_gain(dataset, attr_name);
        attr_gains.push((attr_name.clone(), gain));
    }
    
    // Find maximum gain
    let max_gain = attr_gains.iter()
        .map(|(_, gain)| *gain)
        .fold(f64::MIN, f64::max);
    
    // Return all attributes with maximum gain
    attr_gains.into_iter()
        .filter(|(_, gain)| *gain == max_gain)
        .map(|(attr, _)| attr)
        .collect()
}

/// Find the most frequent values for an attribute
/// Returns all values with maximum frequency (handles ties)
fn find_most_frequent_values(dataset: &NominalDataset, attr_name: &str) -> Vec<String> {
    let groups = dataset.group_by_attribute_value(attr_name);
    
    if groups.is_empty() {
        return Vec::new();
    }
    
    // Find maximum frequency
    let max_freq = groups.values()
        .map(|indices| indices.len())
        .max()
        .unwrap_or(0);
    
    // Return all values with maximum frequency
    groups.into_iter()
        .filter(|(_, indices)| indices.len() == max_freq)
        .map(|(value, _)| value)
        .collect()
}

/// Compute closure for nominal data (group objects by attribute value and find common attributes)
pub fn compute_nominal_closure(dataset: &NominalDataset, attr_name: &str, attr_value: &str) -> (Vec<usize>, HashMap<String, String>) {
    // Step 1: Find all objects with this attribute value (extent)
    let extent: Vec<usize> = dataset.data.iter()
        .enumerate()
        .filter(|(_, obj_data)| obj_data.get(attr_name) == Some(&attr_value.to_string()))
        .map(|(idx, _)| idx)
        .collect();
    
    // Step 2: Find common attributes for these objects (intent)
    if extent.is_empty() {
        return (extent, HashMap::new());
    }
    
    let mut intent = HashMap::new();
    let first_obj = &dataset.data[extent[0]];
    
    // Check which attributes have the same value across all objects in extent
    // Only include descriptive attributes (exclude class) in the intent
    for attr in &dataset.attributes {
        if attr == &dataset.class_attribute {
            continue; // Skip class attribute - it's not part of the formal context
        }
        
        let first_value = first_obj.get(attr);
        if first_value.is_none() {
            continue;
        }
        
        let all_same = extent.iter().all(|&obj_idx| {
            dataset.data[obj_idx].get(attr) == first_value
        });
        
        if all_same {
            intent.insert(attr.clone(), first_value.unwrap().clone());
        }
    }
    
    (extent, intent)
}

/// CNC algorithm.
/// Returns all concepts when there are ties in pertinence or frequency
pub fn cnc(dataset: &NominalDataset) -> CncResult {
    
    // Step 1: Find all most pertinent attributes (handle ties)
    let pertinent_attrs = find_most_pertinent_attributes(dataset);
    
    if pertinent_attrs.is_empty() {
        return CncResult {
            concepts: Vec::new(),
            pertinent_attrs: Vec::new(),
        };
    }
    
    // Steps 2 and 3 there.
    cnc_core(pertinent_attrs, dataset)
}

fn cnc_core(pertinent_attrs: Vec<String>, dataset: &NominalDataset) -> CncResult {
    let mut results = Vec::new();

    // Step 2 of the CNC algorithm: For each pertinent attribute, find all most frequent values (handle ties)
    for pertinent_attr in &pertinent_attrs {
        let most_frequent_values = find_most_frequent_values(dataset, pertinent_attr);
        
        // Step 3: Compute closure for each attribute-value pair
        for value in &most_frequent_values {
            let (extent, intent) = compute_nominal_closure(dataset, pertinent_attr, value);
            results.push((pertinent_attr.clone(), value.clone(), extent, intent));
        }
    }
    
    CncResult {
        concepts: results,
        pertinent_attrs,
    }
}

/// CNC-BP : CNC Bottom-Pertinent Attributes. Keeps only the n most minority classes.
///
/// n: number of minority classes to keep.
///
/// When classes have identical frequencies, all classes at the same frequency level are included.
/// The function selects complete frequency tiers until reaching or exceeding the requested n classes.
/// In case all classes share the same frequency (complete tie), all classes are retained regardless of n.
///
/// Example: If we have classes A(3), B(2), C(2), D(1) and n=2:
/// - Keeps D (most minority with 1 object)
/// - Keeps both B and C (both have 2 objects, tie at second most minority)
/// - Total: 3 classes kept (D, B, C)
pub fn cnc_bp(dataset: &NominalDataset, n: usize) -> CncBpResult {

    // We first get the G.I to not interfere on CNC.
    let pertinent_attrs = find_most_pertinent_attributes(&dataset);
    
    // Step 1: Get all class values and their distribution
    let all_class_values = dataset.get_class_values(&(0..dataset.objects.len()).collect::<Vec<_>>());
    let mut class_counts: HashMap<String, usize> = HashMap::new();
    for class_val in &all_class_values {
        *class_counts.entry(class_val.clone()).or_insert(0) += 1;
    }
    
    // Step 2: Sort classes by frequency (ascending) to find minority classes
    let mut sorted_classes: Vec<_> = class_counts.into_iter().collect();
    sorted_classes.sort_by_key(|(_, count)| *count);
    
    // Get the n most minority class names
    let minority_classes: HashSet<String> = sorted_classes.into_iter()
        .take(n)
        .map(|(class_name, _)| class_name)
        .collect();
    
    // Step 3: Create filtered dataset keeping only objects from minority classes
    let filtered_objects: Vec<usize> = dataset.data.iter().enumerate()
        .filter(|(_, obj_data)| {
            if let Some(class_val) = obj_data.get(&dataset.class_attribute) {
                minority_classes.contains(class_val)
            } else {
                false // Exclude objects with missing class
            }
        })
        .map(|(idx, _)| idx)
        .collect();
    
    // Create filtered dataset
    let filtered_objects_names: Vec<String> = filtered_objects.iter()
        .map(|&obj_idx| dataset.objects[obj_idx].clone())
        .collect();
    
    let filtered_data: Vec<HashMap<String, String>> = filtered_objects.iter()
        .map(|&obj_idx| dataset.data[obj_idx].clone())
        .collect();
    
    let filtered_dataset = NominalDataset {
        objects: filtered_objects_names,
        attributes: dataset.attributes.clone(),
        class_attribute: dataset.class_attribute.clone(),
        data: filtered_data,
    };

    // Apply CNC on filtered dataset
    let cnc_result = if pertinent_attrs.is_empty() {
        CncResult {
            concepts: Vec::new(),
            pertinent_attrs: Vec::new(),
        }
    } else {
        cnc_core(pertinent_attrs.clone(), &filtered_dataset)
    };
    
    CncBpResult {
        cnc_result,
        minority_classes,
        original_size: dataset.objects.len(),
        filtered_size: filtered_dataset.objects.len(),
    }
}

pub fn display_cnc_chosen_attribute(dataset : &NominalDataset, results : &CncResult) {

    println!("Most pertinent attribute(s): {:?}", 
        results.pertinent_attrs);

    for pertinent_attr in &results.pertinent_attrs {

        let most_frequent_values = dataset.get_attribute_values(pertinent_attr);
        println!("  Most frequent value(s) for '{}': {:?}", 
            pertinent_attr, most_frequent_values);
    }
}

/// Display CNC results in a standardized format
pub fn display_cnc_results(dataset: &NominalDataset, results: &[(String, String, Vec<usize>, HashMap<String, String>)]) {
    if results.is_empty() {
        println!("No concepts found");
        return;
    }
    
    // There is a theroem ("connexion de Galois") saying that A''' = A'.
    println!("\nCNC Results ({} concept(s) found):", results.len());
    
    for (i, (pertinent_attr, attr_value, extent, intent)) in results.iter().enumerate() {
        println!("\nConcept {}:", i + 1);
        println!("  Pertinent attribute: '{}' with value '{}'", pertinent_attr, attr_value);
        
        // Show extent (objects)
        let extent_objects: Vec<String> = extent.iter()
            .map(|&obj_idx| dataset.objects[obj_idx].clone())
            .collect();
        println!("  Extent of the pertinent attribute(s): {:?}", extent_objects);
        println!("  Extent size: {}/{} objects ({:.1}%)", 
                 extent.len(), dataset.objects.len(), 
                 (extent.len() as f64 / dataset.objects.len() as f64) * 100.0);
        
        // Show intent (common attributes)
        println!("  Intent (common attributes) of the found extent : {:?}", intent);
        let desc_attrs_count = dataset.attributes.iter().filter(|a| *a != &dataset.class_attribute).count();
        println!("  Intent size: {}/{} attributes ({:.1}%)", 
                 intent.len(), desc_attrs_count,
                 (intent.len() as f64 / desc_attrs_count as f64) * 100.0);
        
        // Show class distribution in extent
        let class_values = dataset.get_class_values(&extent);
        println!("  Class distribution in extent: {:?}", class_values);
        
        // Show majority class
        if let Some((majority_class, count, percentage)) = NominalDataset::get_majority_class(&class_values) {
            println!("  Majority class: '{}' ({}/{}, {:.1}%)", 
                     majority_class, count, extent.len(), percentage);
        }
    }
}
