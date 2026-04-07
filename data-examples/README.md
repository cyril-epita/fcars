# Utilisation de CNC-BP avec des fichiers ARFF

Ce dossier contient des exemples de fichiers ARFF pour tester l'algorithme CNC-BP.

## Fichiers ARFF disponibles

Des fichiers ARFF nominaux (entièrement catégoriels):
- `weather.nominal.arff` - Petit dataset météo avec attributs nominaux
- `contact-lenses.arff` - Dataset sur les lentilles de contact
- `vote.arff` - Votes du congrès américain
- Et plusieurs autres...

## Exécuter CNC-BP sur des fichiers ARFF

### Option 1: Via les tests unitaires

Des tests ont été ajoutés dans `src/lib.rs` qui montrent comment charger et utiliser des fichiers ARFF.

Pour exécuter les tests (qui sont ignorés par défaut):

```bash
# Exécuter un test spécifique
cargo test test_arff_weather_nominal -- --ignored --nocapture

# Exécuter tous les tests ARFF
cargo test test_arff -- --ignored --nocapture
```

### Option 2: Fonction helper `load_arff_as_nominal`

La fonction `load_arff_as_nominal` dans `src/lib.rs` (module tests) peut être utilisée comme référence pour créer votre propre code:

```rust
use arff::{ArffReader, Value};
use fcars::cnc::{NominalDataset, cnc_bp};
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

// Charger un fichier ARFF
let file = File::open("data-examples/weather.nominal.arff")?;
let reader = BufReader::new(file);
let arff_data = ArffReader::new(reader)?;

// Extraire les attributs
let attributes: Vec<String> = arff_data
    .attributes()
    .iter()
    .map(|attr| attr.name().to_string())
    .collect();

// Construire le NominalDataset
let mut objects = Vec::new();
let mut data = Vec::new();

for (idx, instance) in arff_data.instances().enumerate() {
    objects.push(format!("obj_{}", idx));
    let mut obj_data = HashMap::new();

    for (attr_idx, value) in instance.iter().enumerate() {
        let attr_name = &attributes[attr_idx];
        let value_str = match value {
            Value::Nominal(v) => v.to_string(),
            Value::Numeric(v) => v.to_string(), // Attention: considérer la discrétisation
            Value::Missing => "?".to_string(),
            _ => "?".to_string(),
        };
        obj_data.insert(attr_name.clone(), value_str);
    }
    data.push(obj_data);
}

let dataset = NominalDataset::new(
    objects,
    attributes,
    "play".to_string(), // nom de l'attribut classe
    data,
);

// Exécuter CNC-BP
let result = cnc_bp(&dataset, 1); // n=1 (garder la classe la plus minoritaire)
```

## Notes importantes

### Attributs numériques vs nominaux

CNC-BP fonctionne sur des données **nominales** (catégorielles). Si votre fichier ARFF contient des attributs numériques:

1. **Option recommandée**: Discrétisez-les d'abord (convertissez-les en catégories)
2. **Option rapide**: La fonction les convertit en strings, mais les résultats peuvent être moins pertinents

### Exemples de datasets

Fichiers **nominaux** (prêts à l'emploi):
- `weather.nominal.arff` ✓
- `contact-lenses.arff` ✓
- `vote.arff` ✓
- `labor.arff` ✓

Fichiers avec **attributs numériques** (nécessitent discrétisation):
- `iris.arff` - 4 attributs numériques
- `diabetes.arff` - attributs numériques
- `cpu.arff` - attributs numériques

## Paramètre n de CNC-BP

Le paramètre `n` de `cnc_bp(&dataset, n)` indique combien de classes minoritaires conserver:

- `n=1`: Garde seulement la classe la plus minoritaire
- `n=2`: Garde les 2 classes les plus minoritaires
- `n=k`: Garde les k classes les plus minoritaires

En cas d'égalité de fréquence, toutes les classes à ce niveau sont conservées.

## Exemple complet

```bash
# Se placer dans le répertoire racine du projet
cd /home/gustav/fcars

# Exécuter le test avec le dataset weather
cargo test test_arff_weather_nominal -- --ignored --nocapture

# Exécuter le test avec le dataset contact-lenses
cargo test test_arff_contact_lenses -- --ignored --nocapture
```
