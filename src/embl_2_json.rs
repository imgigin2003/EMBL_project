use std::fs::File; // Import File for file operations
use std::io::{BufRead, BufReader, Write}; // Import BufRead, BufReader for reading files line-by-line, and Write for writing
use serde_json::{json, Value}; // Import JSON manipulation tools from serde_json
use std::collections::HashMap; // Import HashMap for key-value data storage

// Main function to process EMBL file and generate JSON output
pub fn process_embl(input_embl: &str, output_json: &str) {
    // Open the EMBL input file
    let file = File::open(input_embl).expect("Failed to open EMBL file");
    // Create a buffered reader for efficient line-by-line reading
    let reader = BufReader::new(file);

    // String to store all JSON output content
    let mut content = String::new();
    // Counter for assigning unique IDs to relationships
    let mut rid: u64 = 0;

    // Variables for EMBL parsing
    let mut seq_record_id = String::new(); // Stores the ID of the sequence
    let mut annotation: HashMap<String, String> = HashMap::new(); // Stores annotations
    let mut organism = String::new(); // Stores organism name
    let mut in_feature = false; // Flag for feature parsing
    let mut current_feature: HashMap<String, Value> = HashMap::new(); // Temporary storage for current feature
    let mut features: Vec<HashMap<String, Value>> = Vec::new(); // List of all parsed features

    // Read each line from the input file
    for line in reader.lines() {
        let line = line.expect("Failed to read lines from EMBL file");

        if line.starts_with("ID") {
            // Extract and store the sequence record ID from the line
            seq_record_id = line.split_whitespace().nth(1).unwrap_or_default().replace(";", "").to_string();
        } else if line.starts_with("OC") {
            // Extract and store the organism name
            organism = line.replace("OC   ", "").trim().to_string();
        } else if line.starts_with("FT") {
            // Handle features parsing
            if in_feature {
                // Store the current feature before starting a new one
                features.push(current_feature.clone());
                // Clear the current feature storage
                current_feature.clear();
            }
            in_feature = true; // Mark as inside a feature

            // Extract feature type from the line
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() > 1 {
                // Extract feature type
                let feature_type = parts[1].trim().split_whitespace().next().unwrap_or("").to_string();
                // Store the feature type in the current feature
                current_feature.insert("type".to_string(), Value::String(feature_type));
            }
        } else if line.starts_with("XX") && in_feature {
            // Add organism information to annotations when features are finalized
            annotation.insert("organism".to_string(), organism.clone());
        } else if line.starts_with("//") {
            // At the end of a record, store any remaining features
            if in_feature {
                features.push(current_feature.clone());
                current_feature.clear();
                in_feature = false;
            }
        }
    }

    // Create a JSON node for the contig (main record)
    let contig_node = json!({
        "type": "node",
        "id": seq_record_id,
        "labels": ["Contig"],
        "properties": {
            "id": seq_record_id,
            "name": organism,
            "organism": organism,
            "annotations": annotation
        }
    });

    // Append the contig node to the content
    content.push_str(&serde_json::to_string(&contig_node).unwrap());
    // Append a newline character to separate nodes and relationships
    content.push('\n');

    // Iterate over each feature to generate nodes and relationships
    for (index, feature) in features.iter().enumerate() {
        // Determine node type (Lead for the first feature, Gene for the rest)
        let node_type = if index == 0 { "Lead" } else { "Gene" };
        // Extract node name from feature type
        let node_name = feature.get("type").and_then(|v| v.as_str()).unwrap_or("Unnamed");

        // Create a feature node
        let feature_node = json!({
            "type": "node",
            "id": format!("{}_{}", seq_record_id, index),
            "labels": [node_type],
            "properties": {
                "type": feature.get("type").unwrap_or(&Value::String("Unknown".to_string())),
                "organism": organism,
                "name": node_name
            }
        });

        // Append the feature node to the content
        content.push_str(&serde_json::to_string(&feature_node).unwrap());
        content.push('\n');

        // Determine relationship label (OWNS for the first, NEXT for others)
        let relationship_label = if index == 0 { "OWNS" } else { "NEXT" };
        // Define start and end node IDs for the relationship
        let start_id = if index == 0 { seq_record_id.clone() } else { format!("{}_{}", seq_record_id, index - 1) };
        let end_id = format!("{}_{}", seq_record_id, index);

        // Create a relationship
        let relationship = json!({
            "id": format!("{}_r_{}", seq_record_id, rid),
            "type": "relationship",
            "label": relationship_label,
            "start": { "id": start_id, "labels": [if index == 0 { "Contig" } else { "Gene" }] },
            "end": { "id": end_id, "labels": [node_type] }
        });

        // Increment the relationship ID counter
        rid += 1;
        // Append the relationship to the content
        content.push_str(&serde_json::to_string(&relationship).unwrap());
        content.push('\n');
    }

    // Write all generated JSON content to the output file
    let mut output_file = File::create(output_json).expect("Failed to create JSON file");
    output_file.write_all(content.as_bytes()).expect("Failed to write JSON");

    // Generate a DOT graph from the JSON output
    generate_graph(output_json, "graph.dot");
}

// Function to generate a DOT file from the JSON input file
fn generate_graph(input_json: &str, output_dot: &str) {
    // Open the JSON file for reading
    let file = File::open(input_json).expect("Failed to open JSON file.");
    let reader = BufReader::new(file);

    // Initialize the DOT file content with the graph declaration
    let mut dot_content = String::from("digraph G {\n");

    // Process each line from the JSON file
    for line in reader.lines() {
        let line = line.expect("Failed to read lines from JSON file");
        let value: Value = serde_json::from_str(&line).expect("Failed to parse JSON line.");

        // Match the type of the JSON object (node or relationship)
        match value.get("type").and_then(|v| v.as_str()) {
            // Handle node representation
            Some("node") => {
                // Extract node ID and name for DOT representation
                let id = value["id"].as_str().unwrap();
                // Replace double quotes with escaped double quotes for DOT compatibility
                let name = value["properties"]["name"].as_str().unwrap_or("unknown").replace("\"", "\\\"");
                // Append the node representation to the DOT content
                dot_content.push_str(&format!("    \"{}\" [label=\"{}\"]\n", id, name));
            }
            // Handle relationship representation
            Some("relationship") => {
                // Extract relationship start ID, end ID, and label for DOT representation
                let start_id = value["start"]["id"].as_str().unwrap();
                // Replace double quotes with escaped double quotes for DOT compatibility
                let end_id = value["end"]["id"].as_str().unwrap();
                // Replace double quotes with escaped double quotes for DOT compatibility
                let label = value["label"].as_str().unwrap_or("unknown").replace("\"", "\\\"");
                // Append the relationship representation to the DOT content
                dot_content.push_str(&format!("    \"{}\" -> \"{}\" [label=\"{}\"]\n", start_id, end_id, label));
            }
            // Ignore other types of JSON objects
            _ => {}
        }
    }

    // Close the graph declaration in the DOT file
    dot_content.push_str("}\n");

    // Write the DOT content to the output file
    let mut output_file = File::create(output_dot).expect("Failed to create DOT file.");
    output_file.write_all(dot_content.as_bytes()).expect("Failed to write to DOT file.");
}
