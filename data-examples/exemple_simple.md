# Exemple simple : Lancer CNC-BP sur un fichier .arff

## Résumé

J'ai ajouté dans `src/lib.rs` (module tests) :
1. Une fonction `load_arff_as_nominal()` qui parse les fichiers .arff
2. Deux tests d'exemple qui montrent comment utiliser CNC-BP avec des fichiers .arff

## Exécuter les exemples

```bash
# Exemple 1 : Weather dataset
cd /home/gustav/fcars
cargo test test_arff_weather_nominal -- --ignored --nocapture

# Exemple 2 : Contact lenses dataset
cargo test test_arff_contact_lenses -- --ignored --nocapture
```

## Code minimal pour utiliser CNC-BP avec un fichier .arff

Voici le code essentiel (voir `src/lib.rs` lignes 356-428 pour la version complète) :

```rust
use fcars::cnc::{NominalDataset, cnc_bp, display_cnc_results};
use std::collections::HashMap;
use std::fs;

// 1. Parser le fichier ARFF (version simplifiée)
fn load_arff(file_path: &str, class_attr: &str) -> Result<NominalDataset, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;

    let mut attributes = Vec::new();
    let mut in_data_section = false;
    let mut objects = Vec::new();
    let mut data = Vec::new();
    let mut obj_counter = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('%') { continue; }

        if line.to_lowercase().starts_with("@data") {
            in_data_section = true;
            continue;
        }

        if !in_data_section {
            if line.to_lowercase().starts_with("@attribute") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    attributes.push(parts[1].to_string());
                }
            }
        } else {
            let values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if values.len() == attributes.len() {
                objects.push(format!("obj_{}", obj_counter));
                obj_counter += 1;

                let mut obj_data = HashMap::new();
                for (i, value) in values.iter().enumerate() {
                    obj_data.insert(attributes[i].clone(), value.clone());
                }
                data.push(obj_data);
            }
        }
    }

    Ok(NominalDataset::new(objects, attributes, class_attr.to_string(), data))
}

// 2. Utiliser CNC-BP
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Charger le fichier
    let dataset = load_arff("data-examples/weather.nominal.arff", "play")?;

    // Afficher le résumé
    dataset.display_summary();

    // Exécuter CNC-BP avec n=1 (garder seulement la classe la plus minoritaire)
    let result = cnc_bp(&dataset, 1);

    println!("\nCNC-BP Results:");
    println!("- Minority classes: {:?}", result.minority_classes);
    println!("- Filtered: {}/{} objects", result.filtered_size, result.original_size);

    display_cnc_results(&dataset, &result.cnc_result.concepts);

    Ok(())
}
```

## Fichiers .arff testés et qui fonctionnent

**Fichiers nominaux** (prêts à l'emploi) :
- ✅ `weather.nominal.arff` - 14 objets, 5 attributs, 2 classes
- ✅ `contact-lenses.arff` - 24 objets, 5 attributs, 3 classes
- `vote.arff`, `labor.arff`, `breast-cancer.arff`, etc.

**Fichiers avec attributs numériques** (fonctionnent mais à discrétiser pour de meilleurs résultats) :
- `iris.arff`, `diabetes.arff`, `cpu.arff`, etc.

## Paramètres importants

### Paramètre `n` de `cnc_bp()`

```rust
cnc_bp(&dataset, 1)  // Garde la classe la plus minoritaire
cnc_bp(&dataset, 2)  // Garde les 2 classes les plus minoritaires
cnc_bp(&dataset, 3)  // Garde les 3 classes les plus minoritaires
```

Si des classes ont la même fréquence, toutes sont gardées (gestion des égalités).

### Attribut classe

Le nom de l'attribut classe doit correspondre exactement au nom dans le fichier .arff:
- `weather.nominal.arff` → `"play"`
- `contact-lenses.arff` → `"contact-lenses"`
- `iris.arff` → `"class"`

Vous pouvez le trouver en regardant la dernière ligne `@attribute` du fichier.

## Résultats attendus

Pour `weather.nominal.arff` avec `n=1`:
```
Minority classes kept: {"no"}
Filtered: 5 objects (was 14)
CNC Results (1 concept(s) found):
  Pertinent attribute: 'outlook' with value 'sunny'
  Majority class: 'no' (2/3, 66.7%)
```

Pour `contact-lenses.arff` avec `n=2`:
```
Minority classes kept: {"soft", "hard"}
Filtered: 9 objects (was 24)
CNC Results (1 concept(s) found):
  Pertinent attribute: 'tear-prod-rate' with value 'normal'
```
