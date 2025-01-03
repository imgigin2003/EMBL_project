use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Node {
    id: String,
    labels: Vec<String>,
    properties: serde_json::Value,
    #[serde(rename = "type")]
    node_type: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Relationship {
    id: String,
    label: String,
    start: Node,
    end: Node,
    #[serde(rename = "type")]
    relationship_type: String,
}

#[derive(Serialize, Debug)]
#[allow(dead_code)]
struct EmblEntry {
    locus_tag: String,
    protein_id: String,
    product: String,
    translation: String,
}

pub fn convert_json(input_json: &str, output_embl: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parse JSON data
    let data: Vec<serde_json::Value> = serde_json::from_str(input_json)?;

    // 2. Extract relevant information
    let mut embl_entries: Vec<EmblEntry> = Vec::new();
    for entry in data {
        if let Some(obj) = entry.as_object() {
            if let Some(properties) = obj.get("properties") {
                if let Some(locus_tag) = properties.get("locus_tag") {
                    if let Some(locus_tag_str) = locus_tag.as_str() {
                        if let Some(protein_id) = properties.get("protein_id") {
                            if let Some(protein_id_str) = protein_id.as_str() {
                                if let Some(product) = properties.get("product") {
                                    if let Some(product_str) = product.as_str() {
                                        if let Some(translation) = properties.get("translation") {
                                            if let Some(translation_str) = translation.as_str() {
                                                embl_entries.push(EmblEntry {
                                                    locus_tag: locus_tag_str.to_string(),
                                                    protein_id: protein_id_str.to_string(),
                                                    product: product_str.to_string(),
                                                    translation: translation_str.to_string(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Generate EMBL output
    let mut output_str = String::new();
    for entry in embl_entries {
        output_str.push_str(&format!(
            "LOCUS       {}\t{}\t{}\n",
            entry.locus_tag, "DNA", "linear\n"
        ));
        output_str.push_str(&format!("DEFINITION  {}\n", entry.product));
        output_str.push_str(&format!("ACCESSION   {}\n", entry.protein_id));
        output_str.push_str(&format!("VERSION     {}\n", entry.protein_id));
        output_str.push_str("KEYWORDS    \n");
        output_str.push_str("SOURCE      \n");
        output_str.push_str("  ORGANISM  \n");
        output_str.push_str("FEATURES             Location/Qualifiers\n");
        output_str.push_str(&format!("     CDS             join(..)\n"));
        output_str.push_str(&format!("                     /product=\"{}\"\n", entry.product)); // Fix 1: Remove extra argument
        output_str.push_str(&format!("                     /protein_id=\"{}\"\n", entry.protein_id)); // Fix 2: Remove extra argument
        output_str.push_str(&format!("                     /translation=\"{}\"\n", entry.translation)); // Fix 3: Remove extra argument
        output_str.push_str("ORIGIN\n");
        // ... (Add DNA sequence here, if available)
        output_str.push_str("//\n");
    }

    let output_file = File::create(output_embl)?;
    let mut writer = std::io::BufWriter::new(output_file);
    writer.write_all(output_str.as_bytes())?;

    Ok(())
}