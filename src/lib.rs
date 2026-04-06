//! This library implements Formal Concept Analysis (FCA) structures and algorithms.
//! It includes definitions for `FormalContext` and `FormalConcept`, along with methods for
//! computing intents and extents, checking for reduced contexts, and validating concepts.
//! The implementation uses bit vectors for efficient representation of relations.

mod bit_fiddling;
mod formal_concept;
mod formal_context;
mod pcbo;

pub use formal_concept::*;
pub use formal_context::*;

pub mod cnc;
pub mod cnc_report;

// Tests
#[cfg(test)]
mod tests {
    use cnc::*;
    use super::*;
    use bitvec::prelude::*;
    use std::collections::HashMap;

    #[test]
    fn test_pcbo_1() {
        let context = FormalContext::new(
            vec!["a", "b", "c"],
            vec!["1", "2", "3"],
            vec![
                bitvec![1, 0, 1], // a
                bitvec![1, 1, 1], // b
                bitvec![0, 1, 1], // c
            ],
        );
        assert_eq!(context.num_concepts(), 4);
    }
    #[test]
    fn test_pcbo_2() {
        // "Lives in Water"
        let context = FormalContext::new(
            vec![
                "fish leech",
                "bream",
                "frog",
                "dog",
                "water weeds",
                "reed",
                "bean",
                "corn",
            ],
            vec![
                "needs water to live",
                "lives in water",
                "lives on land",
                "needs chlorophyll",
                "dicotyledon",
                "monocotyledon",
                "can move",
                "has limbs",
                "breast feeds",
            ],
            vec![
                bitvec![1, 1, 0, 0, 0, 0, 1, 0, 0], // fish leech
                bitvec![1, 1, 0, 0, 0, 0, 1, 1, 0], // bream
                bitvec![1, 1, 1, 0, 0, 0, 1, 1, 0], // frog
                bitvec![1, 0, 1, 0, 0, 0, 1, 1, 1], // dog
                bitvec![1, 1, 0, 1, 0, 1, 0, 0, 0], // water weeds
                bitvec![1, 1, 1, 1, 0, 1, 0, 0, 0], // reed
                bitvec![1, 0, 1, 1, 1, 0, 0, 0, 0], // bean
                bitvec![1, 0, 1, 1, 0, 1, 0, 0, 0], // corn
            ],
        );
        assert_eq!(context.num_concepts(), 19);
    }


    // Helper function to create test data
    fn create_hashmap(entries: Vec<(String, String)>) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (key, value) in entries {
            map.insert(key, value);
        }
        map
    }
    
    fn create_foo_dataset() -> NominalDataset {
        let objects = vec![
            "humpty".to_string(), "dumpty".to_string(), "sat".to_string(),
            "on".to_string(), "a".to_string(), "wall".to_string()
        ];
        
        let attributes = vec![
            "StringAtt1".to_string(),
            "NominalAtt1".to_string(),
            "class".to_string()
        ];
        
        let class_attribute = "class".to_string();
        
        let data = vec![
            create_hashmap(vec![
                ("StringAtt1".to_string(), "humpty".to_string()),
                ("NominalAtt1".to_string(), "g".to_string()),
                ("class".to_string(), "A".to_string())
            ]),
            create_hashmap(vec![
                ("StringAtt1".to_string(), "dumpty".to_string()),
                ("NominalAtt1".to_string(), "g".to_string()),
                ("class".to_string(), "A".to_string())
            ]),
            create_hashmap(vec![
                ("StringAtt1".to_string(), "sat".to_string()),
                ("NominalAtt1".to_string(), "r".to_string()),
                ("class".to_string(), "B".to_string())
            ]),
            create_hashmap(vec![
                ("StringAtt1".to_string(), "on".to_string()),
                ("NominalAtt1".to_string(), "r".to_string()),
                ("class".to_string(), "B".to_string())
            ]),
            create_hashmap(vec![
                ("StringAtt1".to_string(), "a".to_string()),
                ("NominalAtt1".to_string(), "g".to_string()),
                ("class".to_string(), "A".to_string())
            ]),
            create_hashmap(vec![
                ("StringAtt1".to_string(), "wall".to_string()),
                ("NominalAtt1".to_string(), "r".to_string()),
                ("class".to_string(), "B".to_string())
            ]),
        ];
        
        NominalDataset::new(objects, attributes, class_attribute, data)
    }
    
    fn create_animal_dataset() -> NominalDataset {
        let animal_objects = vec![
            "animal1".to_string(), "animal2".to_string(), "animal3".to_string(),
            "animal4".to_string(), "animal5".to_string()
        ];
        
        let animal_attributes = vec![
            "has_fur".to_string(),
            "can_fly".to_string(), 
            "lives_in_water".to_string(),
            "class".to_string()
        ];
        
        let animal_data = vec![
            create_hashmap(vec![
                ("has_fur".to_string(), "yes".to_string()),
                ("can_fly".to_string(), "no".to_string()),
                ("lives_in_water".to_string(), "no".to_string()),
                ("class".to_string(), "mammal".to_string())
            ]),
            create_hashmap(vec![
                ("has_fur".to_string(), "no".to_string()),
                ("can_fly".to_string(), "yes".to_string()),
                ("lives_in_water".to_string(), "no".to_string()),
                ("class".to_string(), "bird".to_string())
            ]),
            create_hashmap(vec![
                ("has_fur".to_string(), "no".to_string()),
                ("can_fly".to_string(), "no".to_string()),
                ("lives_in_water".to_string(), "yes".to_string()),
                ("class".to_string(), "fish".to_string())
            ]),
            create_hashmap(vec![
                ("has_fur".to_string(), "yes".to_string()),
                ("can_fly".to_string(), "no".to_string()),
                ("lives_in_water".to_string(), "yes".to_string()),
                ("class".to_string(), "mammal".to_string())
            ]),
            create_hashmap(vec![
                ("has_fur".to_string(), "no".to_string()),
                ("can_fly".to_string(), "yes".to_string()),
                ("lives_in_water".to_string(), "yes".to_string()),
                ("class".to_string(), "bird".to_string())
            ]),
        ];
        
        NominalDataset::new(animal_objects, animal_attributes, "class".to_string(), animal_data)
    }
    
    fn create_weather_dataset() -> NominalDataset {
        let objects = vec![
            "o2".to_string(), "o6".to_string(), "o9".to_string(),
            "o10".to_string(), "o13".to_string()
        ];
        
        let attributes = vec![
            "Outlook".to_string(),
            "Temperature".to_string(),
            "Humidity".to_string(),
            "Windy".to_string(),
            "Play".to_string(),
        ];
        
        let class_attribute = "Play".to_string();
        
        let data = vec![
            create_hashmap(vec![
                ("Outlook".to_string(), "Sunny".to_string()),
                ("Temperature".to_string(), "Hot".to_string()),
                ("Humidity".to_string(), "High".to_string()),
                ("Windy".to_string(), "True".to_string()),
                ("Play".to_string(), "No".to_string())
            ]),
            create_hashmap(vec![
                ("Outlook".to_string(), "Rainy".to_string()),
                ("Temperature".to_string(), "Cool".to_string()),
                ("Humidity".to_string(), "Normal".to_string()),
                ("Windy".to_string(), "True".to_string()),
                ("Play".to_string(), "No".to_string())
            ]),
            create_hashmap(vec![
                ("Outlook".to_string(), "Sunny".to_string()),
                ("Temperature".to_string(), "Cool".to_string()),
                ("Humidity".to_string(), "Normal".to_string()),
                ("Windy".to_string(), "False".to_string()),
                ("Play".to_string(), "Yes".to_string())
            ]),
            create_hashmap(vec![
                ("Outlook".to_string(), "Rainy".to_string()),
                ("Temperature".to_string(), "Mild".to_string()),
                ("Humidity".to_string(), "Normal".to_string()),
                ("Windy".to_string(), "False".to_string()),
                ("Play".to_string(), "Yes".to_string())
            ]),
            create_hashmap(vec![
                ("Outlook".to_string(), "Overcast".to_string()),
                ("Temperature".to_string(), "Hot".to_string()),
                ("Humidity".to_string(), "Normal".to_string()),
                ("Windy".to_string(), "False".to_string()),
                ("Play".to_string(), "Yes".to_string())
            ]),
        ];
        
        NominalDataset::new(objects, attributes, class_attribute, data)
    }

    fn cnc_verbose(dataset : &NominalDataset) -> CncResult {

        dataset.display_summary();
        // Run CNC algorithm
        let results = cnc(&dataset);
        println!();
        display_cnc_chosen_attribute(&dataset, &results);
        display_cnc_results(&dataset, &results.concepts);

        results
    }
    
    #[test]
    fn cnc_foo() {

        let dataset = create_foo_dataset();
        let results = cnc_verbose(&dataset);

        assert_eq!(8, results.concepts.len());
        //TODO: to add more assert_eq.
    }

    #[test]
    fn cnc_animal() {

        let dataset = create_animal_dataset();
        let results = cnc_verbose(&dataset);
    
        assert_eq!(2, results.concepts.len());
        //TODO: to add more assert_eq.
    }

    #[test]
    fn cnc_weather() {

        let dataset = create_weather_dataset();
        let results = cnc_verbose(&dataset);

        assert_eq!(1, results.concepts.len());
        //TODO: to add more assert_eq.
    }

    #[test]
    fn cnc_bp_foo() {
        let dataset = create_foo_dataset();
        
        // Test cnc_bp with all classes (should behave like regular CNC)
        let bp_all_results = cnc_bp(&dataset, 3);
        println!("Keeping {} most minority classes: {:?}", 3, bp_all_results.minority_classes);
        println!("Filtered dataset: {} objects (was {})", bp_all_results.filtered_size, bp_all_results.original_size);
        assert_eq!(8, bp_all_results.cnc_result.concepts.len());
        
        // Test cnc_bp with n > total classes (should behave like all classes)
        let bp_overflow_results = cnc_bp(&dataset, 5);
        println!("Keeping {} most minority classes: {:?}", 5, bp_overflow_results.minority_classes);
        println!("Filtered dataset: {} objects (was {})", bp_overflow_results.filtered_size, bp_overflow_results.original_size);
        assert_eq!(8, bp_overflow_results.cnc_result.concepts.len());
        
        // Test cnc_bp with 2 classes (two most minority)
        let bp_2_results = cnc_bp(&dataset, 2);
        println!("Keeping {} most minority classes: {:?}", 2, bp_2_results.minority_classes);
        println!("Filtered dataset: {} objects (was {})", bp_2_results.filtered_size, bp_2_results.original_size);
        assert_eq!(8, bp_2_results.cnc_result.concepts.len());
        
        // Test cnc_bp with 1 class (most minority)
        let bp_1_results = cnc_bp(&dataset, 1);
        println!("Keeping {} most minority classes: {:?}", 1, bp_1_results.minority_classes);
        println!("Filtered dataset: {} objects (was {})", bp_1_results.filtered_size, bp_1_results.original_size);
        assert!(bp_1_results.cnc_result.concepts.len() < 8);
    }
    
    #[test]
    fn cnc_bp_animal() {
        let dataset = create_animal_dataset();
        
        // Test cnc_bp with all classes
        let bp_all_results = cnc_bp(&dataset, 3);
        println!("Keeping {} most minority classes: {:?}", 3, bp_all_results.minority_classes);
        println!("Filtered dataset: {} objects (was {})", bp_all_results.filtered_size, bp_all_results.original_size);
        assert_eq!(2, bp_all_results.cnc_result.concepts.len());
        
        // Test cnc_bp with 2 classes (two most minority)
        let bp_2_results = cnc_bp(&dataset, 2);
        println!("Keeping {} most minority classes: {:?}", 2, bp_2_results.minority_classes);
        println!("Filtered dataset: {} objects (was {})", bp_2_results.filtered_size, bp_2_results.original_size);
        assert_eq!(2, bp_2_results.cnc_result.concepts.len());
        
        // Test cnc_bp with 1 class (most minority)
        let bp_1_results = cnc_bp(&dataset, 1);
        println!("Keeping {} most minority classes: {:?}", 1, bp_1_results.minority_classes);
        println!("Filtered dataset: {} objects (was {})", bp_1_results.filtered_size, bp_1_results.original_size);
        assert_eq!(2, bp_1_results.cnc_result.concepts.len());
    }
    
    #[test]
    fn cnc_bp_weather() {
        let dataset = create_weather_dataset();
        
        // Test cnc_bp with all classes (should behave like regular CNC)
        println!("Keeping {} most minority classes.", 2);
        let bp_all_results = cnc_bp(&dataset, 2);
        println!("The {} most minority classes found are: {:?}", 2, bp_all_results.minority_classes);
        println!("Filtered dataset: {} objects (was {})", bp_all_results.filtered_size, bp_all_results.original_size);
        assert_eq!(1, bp_all_results.cnc_result.concepts.len());
        
        // Test cnc_bp with 1 class (most minority)
        let bp_1_results = cnc_bp(&dataset, 1);
        println!("Keeping {} most minority classes: {:?}", 1, bp_1_results.minority_classes);
        println!("Filtered dataset: {} objects (was {})", bp_1_results.filtered_size, bp_1_results.original_size);
        assert_eq!(1, bp_1_results.cnc_result.concepts.len());
    }
}
